mod properties;
mod reason_code;

use properties::UnSubAckProperties;
pub use reason_code::UnSubAckReasonCode;

use crate::v5::{
    commons::{fixed_header::FixedHeader, packet_type::PacketType, property::Property},
    traits::read_data::ReadData,
};

// #[cfg(any(not(feature = "asyncx"), feature = "asyncx"))]
/// Sent by the Server to the Client to confirm receipt of an UNSUBSCRIBE packet
#[derive(Debug, Default, PartialEq, Eq)]
pub struct UnSubAck {
    pub pkid: u16,
    pub properties: UnSubAckProperties,
    pub payload: Vec<UnSubAckReasonCode>,
}

#[cfg(feature = "asyncx")]
pub(crate) use asyncx::*;
#[cfg(feature = "syncx")]
pub(crate) use syncx::*;

impl ReadData for UnSubAck {}

mod syncx {
    use crate::v5::{
        commons::error::MQTTError,
        traits::{
            bufferio::BufferIO,
            syncx::{read::Read, write::Write},
        },
    };

    use super::{
        properties::UnSubAckProperties, FixedHeader, PacketType, UnSubAck, UnSubAckReasonCode,
    };

    impl BufferIO for UnSubAck {
        // Length of the Variable Header plus the length of the Payload
        fn length(&self) -> usize {
            2 + self.properties.length() + self.properties.variable_length() + self.payload.len()
        }

        fn write(&self, buf: &mut bytes::BytesMut) -> Result<(), MQTTError> {
            FixedHeader::new(PacketType::UnSubAck, 0, self.length()).write(buf)?;

            // packet identifier, properties
            self.pkid.write(buf);
            self.properties.write(buf)?;

            self.payload.iter().for_each(|p| u8::from(*p).write(buf));

            Ok(())
        }

        fn read(buf: &mut bytes::Bytes) -> Result<Self, MQTTError> {
            // the assumption here is that the provided buffer has already been advanced by the Fixed Header length
            let mut packet = Self::default();
            packet.pkid = u16::read(buf)?;
            packet.properties = UnSubAckProperties::read(buf)?;

            loop {
                if buf.is_empty() {
                    break;
                }
                packet.payload.push(
                    UnSubAckReasonCode::try_from(u8::read(buf)?).map_err(MQTTError::UnknownData)?,
                );
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

    use super::{
        properties::UnSubAckProperties, FixedHeader, PacketType, UnSubAck, UnSubAckReasonCode,
    };

    impl StreamIO for UnSubAck {
        // Length of the Variable Header plus the length of the Payload
        fn length(&self) -> usize {
            2 + self.properties.length() + self.properties.variable_length() + self.payload.len()
        }

        async fn write<W>(&self, stream: &mut W) -> Result<(), crate::v5::commons::error::MQTTError>
        where
            W: futures::AsyncWriteExt + Unpin,
        {
            FixedHeader::new(PacketType::UnSubAck, 0, self.length())
                .write(stream)
                .await?;

            // packet identifier, properties
            self.pkid.write(stream).await?;
            self.properties.write(stream).await?;

            for p in &self.payload {
                u8::from(*p).write(stream).await?;
            }

            Ok(())
        }

        async fn read<R>(stream: &mut R) -> Result<Self, crate::v5::commons::error::MQTTError>
        where
            R: futures::AsyncReadExt + Unpin,
            Self: Default,
        {
            // the assumption here is that the provided buffer has already been advanced by the Fixed Header length
            let mut packet = Self::default();
            packet.pkid = u16::read(stream).await?;
            packet.properties = UnSubAckProperties::read(stream).await?;

            while let Ok(value) = u8::read(stream).await {
                packet
                    .payload
                    .push(UnSubAckReasonCode::try_from(value).map_err(MQTTError::UnknownData)?);
            }

            Ok(packet)
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use bytes::{Bytes, BytesMut};

//     use crate::v5::packet::unsuback::UnSubAck;

//     use super::*;

//     #[test]
//     fn read_write_unsuback() {
//         let mut packet = UnSubAck::default();
//         packet.pkid = 0x2E;
//         packet.payload = vec![
//             UnSubAckReasonCode::Success,
//             UnSubAckReasonCode::TopicFilterInvalid,
//         ];
//         packet.properties = UnSubAckProperties {
//             reason_string: Some("reason_string here and there".into()),
//             user_property: vec![("key".into(), "value".into())],
//         };

//         let mut buf = BytesMut::with_capacity(50);
//         packet.write(&mut buf).unwrap();
//         let expected =
//             b"\xb01\0.,\x1f\0\x1creason_string here and there&\0\x03key\0\x05value\0\x8f".to_vec();

//         assert_eq!(buf.to_vec(), expected);

//         let mut read_buf = Bytes::from_iter(expected);
//         let fixed_header = FixedHeader::read(&mut read_buf).unwrap();

//         assert_eq!(fixed_header.flags, None);
//         assert_eq!(fixed_header.packet_type, PacketType::UnSubAck);
//         assert_eq!(fixed_header.remaining_length, 49);

//         let read_packet = UnSubAck::read(&mut read_buf).unwrap();
//         assert_eq!(read_packet, packet);
//     }
// }
