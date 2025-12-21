mod options;
mod properties;

pub use options::SubscriptionOptions;
pub use properties::SubscribeProperties;

use crate::v5::{
    commons::{fixed_header::FixedHeader, packet_type::PacketType, property::Property},
    traits::read_data::ReadData,
};

#[cfg(feature = "asyncx")]
pub(crate) use asyncx::*;
#[cfg(not(feature = "asyncx"))]
pub(crate) use syncx::*;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Subscribe {
    pub pkid: u16,
    pub properties: SubscribeProperties,
    /// It is protocl error to have a subscribe packet that doesn't have atleast one payload (topic, subscriptionOptions)
    pub payload: Vec<(String, SubscriptionOptions)>,
}

impl ReadData for Subscribe {}

mod syncx {
    use bytes::Bytes;

    use crate::v5::{
        commons::error::MQTTError,
        traits::{
            bufferio::BufferIO,
            syncx::{read::Read, write::Write},
        },
    };

    use super::{FixedHeader, PacketType, Subscribe, SubscribeProperties, SubscriptionOptions};

    impl BufferIO for Subscribe {
        /// (Length of Variable Header + Length of the Payload)
        fn length(&self) -> usize {
            let mut len = 2 + self.properties.length() + self.properties.variable_length(); // packet identifier + properties
            len += self
                .payload
                .iter()
                .fold(0, |acc, x| acc + (1 + (2 + x.0.len()))); // u8(len) + (string(2) + topic.len())

            len
        }

        fn write(&self, buf: &mut bytes::BytesMut) -> Result<(), MQTTError> {
            if self.payload.len() == 0 {
                return Err(MQTTError::ProtocolError(
                    "Must contain at least one topic/subscription option pair",
                ));
            }

            FixedHeader::new(PacketType::Subscribe, 0b10, self.length()).write(buf)?;
            self.pkid.write(buf);
            self.properties.write(buf)?;
            for (topic, options) in &self.payload {
                topic.write(buf);
                options.write(buf)?;
            }

            Ok(())
        }

        fn read(buf: &mut Bytes) -> Result<Self, MQTTError> {
            let mut packet = Self::default();
            packet.pkid = u16::read(buf)?;

            packet.properties = SubscribeProperties::read(buf)?;

            loop {
                if buf.is_empty() {
                    break;
                }
                let topic = String::read(buf)?;
                let options = SubscriptionOptions::read(buf)?;
                packet.payload.push((topic, options));
            }

            if packet.payload.len() == 0 {
                return Err(MQTTError::ProtocolError(
                    "Must contain at least one topic/subscription option pair",
                ));
            }
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

    use super::{FixedHeader, PacketType, Subscribe, SubscribeProperties, SubscriptionOptions};

    impl StreamIO for Subscribe {
        /// (Length of Variable Header + Length of the Payload)
        fn length(&self) -> usize {
            let mut len = 2 + self.properties.length() + self.properties.variable_length(); // packet identifier + properties
            len += self
                .payload
                .iter()
                .fold(0, |acc, x| acc + (1 + (2 + x.0.len()))); // u8(len) + (string(2) + topic.len())

            len
        }

        async fn write<W>(&self, stream: &mut W) -> Result<(), crate::v5::commons::error::MQTTError>
        where
            W: futures::AsyncWriteExt + Unpin,
        {
            if self.payload.len() == 0 {
                return Err(MQTTError::ProtocolError(
                    "Must contain at least one topic/subscription option pair",
                ));
            }

            FixedHeader::new(PacketType::Subscribe, 0b10, self.length())
                .write(stream)
                .await?;
            self.pkid.write(stream).await?;
            self.properties.write(stream).await?;
            for (topic, options) in &self.payload {
                topic.write(stream).await?;
                options.write(stream).await?;
            }

            Ok(())
        }

        async fn read<R>(stream: &mut R) -> Result<Self, MQTTError>
        where
            R: futures::AsyncReadExt + Unpin,
            Self: Default,
        {
            let mut packet = Self::default();
            packet.pkid = u16::read(stream).await?;

            packet.properties = SubscribeProperties::read(stream).await?;

            while let Ok(topic) = String::read(stream).await {
                let options = SubscriptionOptions::read(stream).await?;
                packet.payload.push((topic, options));
            }

            if packet.payload.len() == 0 {
                return Err(MQTTError::ProtocolError(
                    "Must contain at least one topic/subscription option pair",
                ));
            }

            Ok(packet)
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use bytes::BytesMut;

//     use super::*;

//     #[test]
//     fn should_return_an_error_if_user_tries_to_write_subscribe_with_no_payload() {
//         let mut packet = Subscribe::default();
//         packet.properties = SubscribeProperties {
//             subscription_id: Some(28293),
//             user_property: vec![("key".into(), "value".into())],
//         };
//         packet.pkid = 0x3F;

//         let mut buf = BytesMut::with_capacity(5);
//         let result = packet.write(&mut buf);

//         assert_eq!(
//             result.unwrap_err(),
//             MQTTError::ProtocolError("Must contain at least one topic/subscription option pair")
//         );
//     }

//     #[test]
//     fn should_return_an_error_when_reading_a_subscribe_packet_with_no_payload() {
//         let mut received_bytes =
//             Bytes::from_iter(b"\0?\x11\x0b\x85\xdd\x01&\0\x03key\0\x05value".to_vec());

//         let body = Subscribe::read(&mut received_bytes);
//         assert_eq!(
//             body,
//             Err(MQTTError::ProtocolError(
//                 "Must contain at least one topic/subscription option pair"
//             ))
//         );
//     }

//     #[test]
//     fn default_read_write_subscribe_packet() {
//         let mut packet = Subscribe::default();
//         packet.pkid = 2378;
//         packet.payload = vec![("autos".into(), SubscriptionOptions::default())];

//         let mut buf = BytesMut::with_capacity(20);
//         packet.write(&mut buf).unwrap();

//         let mut read_buf = Bytes::from_iter(buf.to_vec());
//         let fixed_header = FixedHeader::read(&mut read_buf).unwrap();

//         assert_eq!(buf.to_vec(), b"\x82\x0b\tJ\0\0\x05autos\0".to_vec());
//         assert_eq!(fixed_header.flags, Some(0b10));
//         assert_eq!(fixed_header.packet_type, PacketType::Subscribe);
//         let read_packet = Subscribe::read(&mut read_buf).unwrap();
//         assert_eq!(packet, read_packet);
//     }
// }
