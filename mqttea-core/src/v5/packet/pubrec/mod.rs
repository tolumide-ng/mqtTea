pub mod properties;
use properties::{PubRecProperties, PubRecReasonCode};

use crate::v5::{
    commons::{fixed_header::FixedHeader, packet_type::PacketType},
    traits::read_data::ReadData,
};

#[derive(Debug, PartialEq, Eq, Default)]
pub struct PubRec {
    pub(crate) pkid: u16,
    pub(crate) reason_code: PubRecReasonCode,
    pub(crate) properties: PubRecProperties,
}

#[cfg(feature = "asyncx")]
pub(crate) use asyncx::*;
#[cfg(feature = "syncx")]
pub(crate) use syncx::*;

impl ReadData for PubRec {}

mod syncx {
    use crate::v5::traits::syncx::{read::Read, write::Write};
    use crate::v5::{commons::error::MQTTError, traits::bufferio::BufferIO};

    use super::properties::PubRecProperties;
    use super::{properties::PubRecReasonCode, FixedHeader, PacketType, PubRec};

    impl BufferIO for PubRec {
        // length of the variable header, encoded as a variable byte integer
        fn length(&self) -> usize {
            let mut len = std::mem::size_of::<u16>(); // packet identifier

            if self.reason_code == PubRecReasonCode::Success && self.properties.length() == 0 {
                return len;
            }

            len += 1 + self.properties.length() + self.properties.variable_length(); // reason code
            len
        }

        fn write(&self, buf: &mut bytes::BytesMut) -> Result<(), MQTTError> {
            FixedHeader::new(PacketType::PubRec, 0, self.length()).write(buf)?;

            self.pkid.write(buf);
            if self.properties.length() == 0 && self.reason_code == PubRecReasonCode::Success {
                return Ok(());
            }

            u8::from(self.reason_code).write(buf);
            self.properties.write(buf)?;
            Ok(())
        }

        fn read_with_fixedheader(
            buf: &mut bytes::Bytes,
            header: FixedHeader,
        ) -> Result<Self, MQTTError> {
            let mut packet = Self::default();
            packet.pkid = u16::read(buf)?;

            if header.remaining_length == 2 {
                packet.reason_code = PubRecReasonCode::Success;
                return Ok(packet);
            }

            packet.reason_code = PubRecReasonCode::try_from(u8::read(buf)?)
                .map_err(|e| MQTTError::UnknownData(e))?;
            packet.properties = PubRecProperties::read(buf)?;

            Ok(packet)
        }
    }
}

mod asyncx {
    use crate::v5::{
        commons::error::MQTTError,
        traits::{
            asyncx::{read::Read, write::Write},
            streamio::StreamIO,
        },
    };

    use super::{
        properties::{PubRecProperties, PubRecReasonCode},
        FixedHeader, PacketType, PubRec,
    };

    impl StreamIO for PubRec {
        // length of the variable header, encoded as a variable byte integer
        fn length(&self) -> usize {
            let mut len = std::mem::size_of::<u16>(); // packet identifier

            if self.reason_code == PubRecReasonCode::Success && self.properties.length() == 0 {
                return len;
            }

            len += 1 + self.properties.length() + self.properties.variable_length(); // reason code
            len
        }

        async fn write<W>(&self, stream: &mut W) -> Result<(), crate::v5::commons::error::MQTTError>
        where
            W: futures::AsyncWriteExt + Unpin,
        {
            FixedHeader::new(PacketType::PubRec, 0, self.length())
                .write(stream)
                .await?;

            self.pkid.write(stream).await?;
            if self.properties.length() == 0 && self.reason_code == PubRecReasonCode::Success {
                return Ok(());
            }

            u8::from(self.reason_code).write(stream).await?;
            self.properties.write(stream).await?;

            Ok(())
        }

        async fn read_with_fixedheader<R>(
            stream: &mut R,
            header: &FixedHeader,
        ) -> Result<Self, crate::v5::commons::error::MQTTError>
        where
            R: futures::AsyncReadExt + Unpin,
            Self: Default,
        {
            let mut packet = Self::default();
            packet.pkid = u16::read(stream).await?;

            if header.remaining_length == 2 {
                packet.reason_code = PubRecReasonCode::Success;
                return Ok(packet);
            }

            packet.reason_code = PubRecReasonCode::try_from(u8::read(stream).await?)
                .map_err(|e| MQTTError::UnknownData(e))?;
            packet.properties = PubRecProperties::read(stream).await?;

            Ok(packet)
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use bytes::{Bytes, BytesMut};

//     use crate::v5::packet::pubrec::{FixedHeader, PacketType};

//     use super::{properties::PubRecReasonCode, PubRec};

//     #[test]
//     fn read_write_with_no_properties() {
//         let mut packet = PubRec::default();
//         packet.reason_code = PubRecReasonCode::Success;

//         let mut buf = BytesMut::with_capacity(20);
//         packet.write(&mut buf).unwrap();

//         assert_eq!(buf.to_vec(), b"P\x02\0\0".to_vec());

//         let mut read_buf = Bytes::from_iter(buf.to_vec());
//         let header = FixedHeader::read(&mut read_buf).unwrap();

//         assert_eq!(header.remaining_length, 2);
//         assert_eq!(header.packet_type, PacketType::PubRec);

//         let received_packet = PubRec::read_with_fixedheader(&mut read_buf, header).unwrap();
//         assert_eq!(packet, received_packet);
//     }

//     #[test]
//     fn read_write_with_properties_and_reasoncode() {
//         let mut packet = PubRec::default();
//         packet.properties.reason_string =
//             Some(String::from("thisIsAReasonStriing--andMoreAndMore"));
//         packet.properties.user_property =
//             vec![(String::from("notAuthorized"), String::from("value"))];
//         packet.reason_code = PubRecReasonCode::NotAuthorized;

//         let mut buf = BytesMut::with_capacity(50);
//         packet.write(&mut buf).unwrap();

//         let expected =
//             b"PB\0\0\x87>\x1f\0$thisIsAReasonStriing--andMoreAndMore&\0\rnotAuthorized\0\x05value"
//                 .to_vec();

//         assert_eq!(buf.to_vec(), expected);

//         let mut read_buf = Bytes::from_iter(expected);
//         let fixed_header = FixedHeader::read(&mut read_buf).unwrap();

//         assert_eq!(fixed_header.packet_type, PacketType::PubRec);

//         let received_packet = PubRec::read_with_fixedheader(&mut read_buf, fixed_header).unwrap();
//         assert_eq!(packet, received_packet);
//     }
// }
