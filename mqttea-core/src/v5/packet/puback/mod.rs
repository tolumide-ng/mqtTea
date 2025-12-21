mod properties;
pub use properties::{PubAckProperties, PubAckReasonCode};

use crate::v5::{
    commons::{fixed_header::FixedHeader, packet_type::PacketType, property::Property},
    traits::read_data::ReadData,
};

#[derive(Debug, Default, PartialEq, Eq)]
pub struct PubAck {
    pub(crate) pkid: u16,
    pub(crate) reason_code: PubAckReasonCode,
    pub(crate) properties: PubAckProperties,
}

#[cfg(feature = "asyncx")]
pub(crate) use asyncx::*;
#[cfg(feature = "syncx")]
pub(crate) use syncx::*;

impl ReadData for PubAck {}

mod syncx {
    use crate::v5::{
        commons::error::MQTTError,
        traits::{
            bufferio::BufferIO,
            syncx::{read::Read, write::Write},
        },
    };

    use super::{FixedHeader, PacketType, PubAck, PubAckProperties, PubAckReasonCode};

    impl BufferIO for PubAck {
        /// Length of the Variable Header, encoded as Variable Byte Integer
        fn length(&self) -> usize {
            let mut len = std::mem::size_of::<u16>(); // packet identifier

            // only add reason code if there's no properties
            if self.reason_code == PubAckReasonCode::Success && self.properties.length() == 0 {
                return len;
            }

            len += 1; // reason code
            len += self.properties.length() + self.properties.variable_length();
            len
        }

        fn write(&self, buf: &mut bytes::BytesMut) -> Result<(), MQTTError> {
            FixedHeader::new(PacketType::PubAck, 0, self.length()).write(buf)?;
            self.pkid.write(buf);
            if self.reason_code == PubAckReasonCode::Success && self.properties.length() == 0 {
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
                packet.reason_code = PubAckReasonCode::Success;
                return Ok(packet);
            }

            packet.reason_code = PubAckReasonCode::try_from(u8::read(buf)?)
                .map_err(|e| MQTTError::UnknownData(format!("Uknown reason code: {e}")))?;
            packet.properties = PubAckProperties::read(buf)?;

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

    use super::{FixedHeader, PacketType, PubAck, PubAckProperties, PubAckReasonCode};

    impl StreamIO for PubAck {
        /// Length of the Variable Header, encoded as Variable Byte Integer
        fn length(&self) -> usize {
            let mut len = std::mem::size_of::<u16>(); // packet identifier

            // only add reason code if there's no properties
            if self.reason_code == PubAckReasonCode::Success && self.properties.length() == 0 {
                return len;
            }

            len += 1; // reason code
            len += self.properties.length() + self.properties.variable_length();
            len
        }

        async fn write<W>(&self, stream: &mut W) -> Result<(), crate::v5::commons::error::MQTTError>
        where
            W: futures::AsyncWriteExt + Unpin,
        {
            FixedHeader::new(PacketType::PubAck, 0, self.length())
                .write(stream)
                .await?;
            self.pkid.write(stream).await?;
            if self.reason_code == PubAckReasonCode::Success && self.properties.length() == 0 {
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
                packet.reason_code = PubAckReasonCode::Success;
                return Ok(packet);
            }

            packet.reason_code = PubAckReasonCode::try_from(u8::read(stream).await?)
                .map_err(|e| MQTTError::UnknownData(format!("Uknown reason code: {e}")))?;
            packet.properties = PubAckProperties::read(stream).await?;

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
//         let mut packet = PubAck::default();
//         packet.reason_code = PubAckReasonCode::Success;

//         let mut buf = BytesMut::with_capacity(200);
//         packet.write(&mut buf).unwrap();

//         assert_eq!(buf.to_vec(), b"@\x02\0\0".to_vec());

//         let mut read_buf = Bytes::from_iter(buf.to_vec());
//         let fixed_header = FixedHeader::read(&mut read_buf).unwrap();

//         assert_eq!(fixed_header.remaining_length, 2);
//         assert_eq!(fixed_header.packet_type, PacketType::PubAck);

//         let received_packet = PubAck::read_with_fixedheader(&mut read_buf, fixed_header).unwrap();
//         assert_eq!(packet, received_packet);
//     }

//     #[test]
//     fn read_write_with_neither_properties_nor_reasoncode() {
//         let packet = PubAck::default();

//         let mut buf = BytesMut::with_capacity(200);
//         packet.write(&mut buf).unwrap();

//         assert_eq!(buf.to_vec(), b"@\x02\0\0".to_vec());

//         let mut read_buf = Bytes::from_iter(buf.to_vec());
//         let fixed_header = FixedHeader::read(&mut read_buf).unwrap();

//         assert_eq!(fixed_header.remaining_length, 2);
//         assert_eq!(fixed_header.packet_type, PacketType::PubAck);

//         let received_packet = PubAck::read_with_fixedheader(&mut read_buf, fixed_header).unwrap();
//         assert_eq!(packet, received_packet);
//         assert_eq!(packet.reason_code, PubAckReasonCode::Success);
//     }

//     #[test]
//     fn read_write_with_reasoncode_and_properties() {
//         let mut packet = PubAck::default();
//         packet.reason_code = PubAckReasonCode::Success;
//         packet.properties.reason_string =
//             Some(String::from("thisIsAReasonStriing--andMoreAndMore"));
//         packet.properties.user_property = vec![(String::from("keyKey"), String::from("value"))];

//         let mut buf = BytesMut::with_capacity(200);
//         packet.write(&mut buf).unwrap();

//         let expected =
//             b"@;\0\0\07\x1f\0$thisIsAReasonStriing--andMoreAndMore&\0\x06keyKey\0\x05value"
//                 .to_vec();

//         let mut read_buf = Bytes::from_iter(expected);
//         let fixed_header = FixedHeader::read(&mut read_buf).unwrap();

//         assert_eq!(fixed_header.packet_type, PacketType::PubAck);

//         let received_packet = PubAck::read_with_fixedheader(&mut read_buf, fixed_header).unwrap();
//         assert_eq!(packet, received_packet);
//         assert_eq!(packet.reason_code, PubAckReasonCode::Success);
//     }
// }
