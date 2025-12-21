mod properties;
pub use properties::UnSubscribeProperties;

use crate::v5::traits::read_data::ReadData;

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct UnSubscribe {
    pub pkid: u16,
    pub properties: UnSubscribeProperties,
    pub payload: Vec<String>,
}

#[cfg(feature = "asyncx")]
pub(crate) use asyncx::*;
#[cfg(feature = "syncx")]
pub(crate) use syncx::*;

impl ReadData for UnSubscribe {}

mod syncx {
    use crate::v5::{
        commons::{error::MQTTError, fixed_header::FixedHeader, packet_type::PacketType},
        traits::{
            bufferio::BufferIO,
            syncx::{read::Read, write::Write},
        },
    };

    use super::{UnSubscribe, UnSubscribeProperties};

    impl BufferIO for UnSubscribe {
        /// Length of the Variable Header (2 bytes) plus the length of the Payload
        fn length(&self) -> usize {
            // packet identidier + string len
            2 + self.payload.iter().fold(0, |acc, x| acc + x.len() + 2)
                + self.properties.length()
                + self.properties.variable_length()
        }

        fn write(&self, buf: &mut bytes::BytesMut) -> Result<(), MQTTError> {
            if self.payload.is_empty() {
                return Err(MQTTError::ProtocolError(
                    "The Payload of an UNSUBSCRIBE packet MUST contain at least one Topic Filter",
                ));
            };
            FixedHeader::new(PacketType::UnSubscribe, 0b10, self.length()).write(buf)?;

            self.pkid.write(buf);
            self.properties.write(buf)?;
            self.payload.iter().for_each(|p| p.write(buf));
            Ok(())
        }

        fn read(buf: &mut bytes::Bytes) -> Result<Self, MQTTError> {
            // the assumption here is that the provided buffer has already been advanced by the Fixed Header length
            let mut packet = Self::default();

            packet.pkid = u16::read(buf)?;
            packet.properties = UnSubscribeProperties::read(buf)?;
            loop {
                if buf.is_empty() {
                    break;
                }
                packet.payload.push(String::read(buf)?);
            }

            if packet.payload.is_empty() {
                return Err(MQTTError::ProtocolError(
                    "The Payload of an UNSUBSCRIBE packet MUST contain at least one Topic Filter",
                ));
            };

            Ok(packet)
        }
    }
}

mod asyncx {
    use crate::v5::{
        commons::{error::MQTTError, fixed_header::FixedHeader, packet_type::PacketType},
        traits::{
            asyncx::{read::Read, write::Write},
            streamio::StreamIO,
        },
    };

    use super::{UnSubscribe, UnSubscribeProperties};

    impl StreamIO for UnSubscribe {
        /// Length of the Variable Header (2 bytes) plus the length of the Payload
        fn length(&self) -> usize {
            // packet identidier + string len
            2 + self.payload.iter().fold(0, |acc, x| acc + x.len() + 2)
                + self.properties.length()
                + self.properties.variable_length()
        }

        async fn write<W>(&self, stream: &mut W) -> Result<(), crate::v5::commons::error::MQTTError>
        where
            W: futures::AsyncWriteExt + Unpin,
        {
            if self.payload.is_empty() {
                return Err(MQTTError::ProtocolError(
                    "The Payload of an UNSUBSCRIBE packet MUST contain at least one Topic Filter",
                ));
            };
            FixedHeader::new(PacketType::UnSubscribe, 0b10, self.length())
                .write(stream)
                .await?;

            self.pkid.write(stream).await?;
            self.properties.write(stream).await?;
            for p in &self.payload {
                p.write(stream).await?;
            }

            Ok(())
        }

        async fn read<R>(stream: &mut R) -> Result<Self, MQTTError>
        where
            R: futures::AsyncReadExt + Unpin,
            Self: Default,
        {
            // the assumption here is that the provided buffer has already been advanced by the Fixed Header length
            let mut packet = Self::default();

            packet.pkid = u16::read(stream).await?;
            packet.properties = UnSubscribeProperties::read(stream).await?;

            while let Ok(value) = String::read(stream).await {
                packet.payload.push(value);
            }

            if packet.payload.is_empty() {
                return Err(MQTTError::ProtocolError(
                    "The Payload of an UNSUBSCRIBE packet MUST contain at least one Topic Filter",
                ));
            };

            Ok(packet)
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use bytes::{Bytes, BytesMut};

//     use crate::v5::{
//         commons::{error::MQTTError, fixed_header::FixedHeader, packet_type::PacketType},
//         packet::unsubscribe::UnSubscribe,
//         traits::syncx::{read::Read, write::Write},
//     };

//     #[test]
//     fn should_fail_to_read_a_unsubscribe_buf_without_atleast_one_topic() {
//         let mut bytes = Bytes::from_iter(b"\xa2\x10\0.\r&\0\x03key\0\x05value".to_vec());
//         let fixed_header = FixedHeader::read(&mut bytes).unwrap();

//         assert_eq!(fixed_header.flags, Some(0b10));
//         assert_eq!(fixed_header.packet_type, PacketType::UnSubscribe);

//         let packet = UnSubscribe::read(&mut bytes);
//         assert_eq!(
//             packet.unwrap_err(),
//             MQTTError::ProtocolError(
//                 "The Payload of an UNSUBSCRIBE packet MUST contain at least one Topic Filter"
//             )
//         );
//     }

//     #[test]
//     fn should_fail_to_write_a_unsubscribe_packet_without_a_topic() {
//         let packet = UnSubscribe::default();

//         let mut buf = BytesMut::with_capacity(5);
//         let result = packet.write(&mut buf);

//         assert_eq!(
//             result,
//             Err(MQTTError::ProtocolError(
//                 "The Payload of an UNSUBSCRIBE packet MUST contain at least one Topic Filter"
//             ))
//         )
//     }

//     #[test]
//     fn read_write_unsubscribe() {
//         let mut packet = UnSubscribe::default();
//         packet.pkid = 0x2E;
//         packet.payload = vec!["topicA".into(), "topicB".into()];
//         packet.properties = UnSubscribeProperties {
//             user_property: vec![("key".into(), "value".into())],
//         };

//         let mut buf = BytesMut::with_capacity(30);
//         packet.write(&mut buf).unwrap();

//         let mut read_buf = Bytes::from_iter(buf.to_vec());
//         let fixed_header = FixedHeader::read(&mut read_buf).unwrap();

//         assert_eq!(fixed_header.flags, Some(0b10));
//         assert_eq!(fixed_header.packet_type, PacketType::UnSubscribe);
//         assert_eq!(fixed_header.remaining_length, 32);

//         let read_packet = UnSubscribe::read(&mut read_buf).unwrap();
//         assert_eq!(read_packet, packet);
//     }
// }
