mod properties;
pub use properties::{PubRelProperties, PubRelReasonCode};

use crate::v5::{
    commons::{fixed_header::FixedHeader, packet_type::PacketType, property::Property},
    traits::read_data::ReadData,
};

#[derive(Debug, Default, PartialEq, Eq)]
pub struct PubRel {
    pub(crate) pkid: u16,
    pub(crate) reason_code: PubRelReasonCode,
    pub(crate) properties: PubRelProperties,
}

#[cfg(feature = "asyncx")]
pub(crate) use asyncx::*;
#[cfg(feature = "syncx")]
pub(crate) use syncx::*;

impl ReadData for PubRel {}
mod syncx {
    use crate::v5::{
        commons::error::MQTTError,
        traits::{
            bufferio::BufferIO,
            syncx::{read::Read, write::Write},
        },
    };

    use super::{FixedHeader, PacketType, PubRel, PubRelProperties, PubRelReasonCode};

    impl BufferIO for PubRel {
        fn length(&self) -> usize {
            let mut len = std::mem::size_of::<u16>(); // packet identifier
            if self.reason_code == PubRelReasonCode::Success && self.properties.length() == 0 {
                return len;
            }
            len += 1 + self.properties.length() + self.properties.variable_length(); // reason code
            len
        }

        fn write(&self, buf: &mut bytes::BytesMut) -> Result<(), MQTTError> {
            FixedHeader::new(PacketType::PubRel, 0b10, self.length()).write(buf)?;

            self.pkid.write(buf);
            if self.properties.length() == 0 && self.reason_code == PubRelReasonCode::Success {
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
                packet.reason_code = PubRelReasonCode::Success;
                return Ok(packet);
            }

            packet.reason_code = PubRelReasonCode::try_from(u8::read(buf)?)
                .map_err(|e| MQTTError::UnknownData(e))?;
            packet.properties = PubRelProperties::read(buf)?;

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

    use super::{FixedHeader, PacketType, PubRel, PubRelProperties, PubRelReasonCode};

    impl StreamIO for PubRel {
        fn length(&self) -> usize {
            let mut len = std::mem::size_of::<u16>(); // packet identifier
            if self.reason_code == PubRelReasonCode::Success && self.properties.length() == 0 {
                return len;
            }
            len += 1 + self.properties.length() + self.properties.variable_length(); // reason code
            len
        }

        async fn write<W>(&self, stream: &mut W) -> Result<(), crate::v5::commons::error::MQTTError>
        where
            W: futures::AsyncWriteExt + Unpin,
        {
            FixedHeader::new(PacketType::PubRel, 0b10, self.length())
                .write(stream)
                .await?;

            self.pkid.write(stream).await?;
            if self.properties.length() == 0 && self.reason_code == PubRelReasonCode::Success {
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
                packet.reason_code = PubRelReasonCode::Success;
                return Ok(packet);
            }

            packet.reason_code = PubRelReasonCode::try_from(u8::read(stream).await?)
                .map_err(|e| MQTTError::UnknownData(e))?;
            packet.properties = PubRelProperties::read(stream).await?;

            Ok(packet)
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use bytes::{Bytes, BytesMut};

//     use super::*;

//     #[test]
//     fn read_write_with_no_properties() {
//         let mut packet = PubRel::default();
//         packet.reason_code = PubRelReasonCode::Success;

//         let mut buf = BytesMut::with_capacity(20);
//         packet.write(&mut buf).unwrap();

//         assert_eq!(buf.to_vec(), b"b\x02\0\0".to_vec());

//         let mut read_buf = Bytes::from_iter(buf.to_vec());
//         let header = FixedHeader::read(&mut read_buf).unwrap();

//         assert_eq!(header.remaining_length, 2);
//         assert_eq!(header.packet_type, PacketType::PubRel);

//         let received_packet = PubRel::read_with_fixedheader(&mut read_buf, header).unwrap();
//         assert_eq!(packet, received_packet);
//     }

//     #[test]
//     fn read_write_with_neither_properties_nor_reasoncode() {
//         let packet = PubRel::default();

//         let mut buf = BytesMut::with_capacity(10);
//         packet.write(&mut buf).unwrap();

//         assert_eq!(buf.to_vec(), b"b\x02\0\0".to_vec());

//         let mut read_buf = Bytes::from_iter(buf.to_vec());
//         let fixed_header = FixedHeader::read(&mut read_buf).unwrap();

//         assert_eq!(fixed_header.remaining_length, 2);
//         assert_eq!(fixed_header.packet_type, PacketType::PubRel);

//         let received_packet = PubRel::read_with_fixedheader(&mut read_buf, fixed_header).unwrap();
//         assert_eq!(packet, received_packet);
//         assert_eq!(packet.reason_code, PubRelReasonCode::Success);
//     }

//     #[test]
//     fn read_write_with_properties_and_reasoncode() {
//         let mut packet = PubRel::default();
//         packet.properties.reason_string =
//             Some(String::from("thisIsAReasonStriing--andMoreAndMore"));
//         packet.properties.user_property =
//             vec![(String::from("notAuthorized"), String::from("value"))];
//         packet.reason_code = PubRelReasonCode::PacketIdentifierNotFound;

//         let mut buf = BytesMut::with_capacity(50);
//         packet.write(&mut buf).unwrap();

//         let expected =
//             b"bB\0\0\x92>\x1f\0$thisIsAReasonStriing--andMoreAndMore&\0\rnotAuthorized\0\x05value"
//                 .to_vec();

//         assert_eq!(buf.to_vec(), expected);

//         let mut read_buf = Bytes::from_iter(expected);
//         let fixed_header = FixedHeader::read(&mut read_buf).unwrap();

//         assert_eq!(fixed_header.packet_type, PacketType::PubRel);

//         let received_packet = PubRel::read_with_fixedheader(&mut read_buf, fixed_header).unwrap();
//         assert_eq!(packet, received_packet);
//     }
// }
