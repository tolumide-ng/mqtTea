mod properties;
mod reason_code;
use properties::SubAckProperties;
pub use reason_code::SubAckReasonCode;

use crate::v5::{
    commons::{fixed_header::FixedHeader, packet_type::PacketType},
    traits::read_data::ReadData,
};

/// 3.9: Sent by the Server to the Client to confirm receipt and processing of a SUBSCRIBE packet.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct SubAck {
    pub pkid: u16,
    pub payload: Vec<SubAckReasonCode>,
    pub properties: SubAckProperties,
}

#[cfg(feature = "asyncx")]
pub(crate) use asyncx::*;
#[cfg(feature = "syncx")]
pub(crate) use syncx::*;

impl ReadData for SubAck {}

mod syncx {
    use crate::v5::{
        commons::error::MQTTError,
        traits::{
            bufferio::BufferIO,
            syncx::{read::Read, write::Write},
        },
    };

    use super::{properties::SubAckProperties, FixedHeader, PacketType, SubAck, SubAckReasonCode};

    impl BufferIO for SubAck {
        // Length of the Variable Header plus the length of the Payload
        fn length(&self) -> usize {
            2 + self.payload.len() + self.properties.length() + self.properties.variable_length()
        }

        fn write(&self, buf: &mut bytes::BytesMut) -> Result<(), MQTTError> {
            FixedHeader::new(PacketType::SubAck, 0, self.length()).write(buf)?;

            self.pkid.write(buf);
            self.properties.write(buf)?;
            self.payload.iter().for_each(|x| u8::from(*x).write(buf));
            Ok(())
        }

        fn read(buf: &mut bytes::Bytes) -> Result<Self, MQTTError> {
            let mut packet = Self::default();
            packet.pkid = u16::read(buf)?;
            packet.properties = SubAckProperties::read(buf)?;

            // there is no specific rule outright forbidding a suback with an empty reason code although MQTT-3.9.3-2
            // advices in this line, it doesn't mention that an Error should be returned (NEED to ask around or rethink) - !todo
            // if buf.is_empty() { return Err(MQTTError::MalformedPacket)}

            loop {
                if buf.is_empty() {
                    break;
                }
                packet.payload.push(
                    SubAckReasonCode::try_from(u8::read(buf)?).map_err(MQTTError::UnknownData)?,
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

    use super::{properties::SubAckProperties, FixedHeader, PacketType, SubAck, SubAckReasonCode};

    impl StreamIO for SubAck {
        // Length of the Variable Header plus the length of the Payload
        fn length(&self) -> usize {
            2 + self.payload.len() + self.properties.length() + self.properties.variable_length()
        }

        async fn write<W>(&self, stream: &mut W) -> Result<(), crate::v5::commons::error::MQTTError>
        where
            W: futures::AsyncWriteExt + Unpin,
        {
            FixedHeader::new(PacketType::SubAck, 0, self.length())
                .write(stream)
                .await?;

            self.pkid.write(stream).await?;
            self.properties.write(stream).await?;

            for payload in &self.payload {
                u8::from(*payload).write(stream).await?;
            }

            Ok(())
        }

        async fn read<R>(stream: &mut R) -> Result<Self, crate::v5::commons::error::MQTTError>
        where
            R: futures::AsyncReadExt + Unpin,
            Self: Default,
        {
            let mut packet = Self::default();
            packet.pkid = u16::read(stream).await?;
            packet.properties = SubAckProperties::read(stream).await?;

            while let Ok(value) = u8::read(stream).await {
                SubAckReasonCode::try_from(value).map_err(MQTTError::UnknownData)?;
            }

            Ok(packet)
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use bytes::{Bytes, BytesMut};

//     use super::*;

//     #[test]
//     fn read_write_suback() {
//         let packet = SubAck::default();
//         let mut buf = BytesMut::with_capacity(5);
//         packet.write(&mut buf).unwrap();

//         assert_eq!(buf.to_vec(), b"\x90\x03\0\0\0".to_vec());

//         let mut read_buf = Bytes::from_iter(buf.to_vec());
//         let fixed_header = FixedHeader::read(&mut read_buf).unwrap();
//         assert_eq!(fixed_header.flags, None);
//         assert_eq!(fixed_header.packet_type, PacketType::SubAck);
//         assert_eq!(fixed_header.remaining_length, 3);
//     }

//     #[test]
//     fn read_write_suback_with_properties_and_payload() {
//         let mut packet = SubAck::default();
//         packet.pkid = 0x3F;
//         packet.payload = vec![
//             SubAckReasonCode::GrantedQoS2,
//             SubAckReasonCode::QuotaExceeded,
//             SubAckReasonCode::UnspecifiedError,
//             SubAckReasonCode::NotAuhtorized,
//         ];
//         packet.properties = SubAckProperties {
//             reason_string: Some("googoogReason".into()),
//             user_property: vec![("keyAbc".into(), "valueAbc".into())],
//         };

//         let mut buf = BytesMut::with_capacity(5);
//         packet.write(&mut buf).unwrap();

//         let mut read_buf = Bytes::from_iter(buf.to_vec());
//         let fixed_header = FixedHeader::read(&mut read_buf).unwrap();
//         let read_packet = SubAck::read(&mut read_buf).unwrap();

//         println!("xddddddd {:?}", buf);
//         assert_eq!(
//             buf.to_vec(),
//             b"\x90*\0?#\x1f\0\rgoogoogReason&\0\x06keyAbc\0\x08valueAbc\x02\x97\x80\x87".to_vec()
//         );
//         assert_eq!(fixed_header.flags, None);
//         assert_eq!(fixed_header.packet_type, PacketType::SubAck);
//         assert_eq!(fixed_header.remaining_length, 42);
//         assert_eq!(packet, read_packet);
//     }
// }
