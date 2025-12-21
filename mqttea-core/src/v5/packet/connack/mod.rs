mod properties;
pub(crate) mod reason_code;

use properties::ConnAckProperties;
use reason_code::ConnAckReasonCode;

use crate::v5::{
    commons::{fixed_header::FixedHeader, packet_type::PacketType},
    traits::read_data::ReadData,
};

#[derive(Debug, Default, PartialEq, Eq)]
pub struct ConnAck {
    /// 3.2.2.1.1 Connect Acknowledge flag
    pub session_present: bool, // bit 0 of the COnnect Acknowledge flag
    pub(crate) reason: ConnAckReasonCode,
    pub properties: ConnAckProperties,
}

impl ReadData for ConnAck {}

mod synx {
    use crate::v5::commons::{error::MQTTError, fixed_header::FixedHeader};
    use crate::v5::traits::{
        bufferio::BufferIO,
        syncx::{read::Read, write::Write},
    };

    use super::properties::ConnAckProperties;
    use super::{ConnAck, ConnAckReasonCode, PacketType};

    impl BufferIO for ConnAck {
        /// This is the length of the Variable Header
        fn length(&self) -> usize {
            let mut len = 1 + 1; // session present + reason code
            len += self.properties.length();
            len += self.properties.variable_length();
            len
        }

        fn write(&self, buf: &mut bytes::BytesMut) -> Result<(), MQTTError> {
            FixedHeader::new(PacketType::ConnAck, 0, self.length()).write(buf)?;

            u8::from(self.session_present).write(buf);
            (self.reason as u8).write(buf);
            self.properties.write(buf)?;

            Ok(())
        }

        fn read(buf: &mut bytes::Bytes) -> Result<Self, MQTTError> {
            // Assumption is that the fixed header as been read already
            let mut packet = Self::default();
            packet.session_present = u8::read(buf)? != 0;
            let reason = u8::read(buf)?;
            packet.reason = ConnAckReasonCode::try_from(reason).map_err(|_| {
                MQTTError::UnknownData(format!("Unrecognized reason code: {reason}"))
            })?;
            packet.properties = ConnAckProperties::read(buf)?;

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
        properties::ConnAckProperties, ConnAck, ConnAckReasonCode, FixedHeader, PacketType,
    };

    impl StreamIO for ConnAck {
        /// This is the length of the Variable Header
        fn length(&self) -> usize {
            let mut len = 1 + 1; // session present + reason code
            len += self.properties.length();
            len += self.properties.variable_length();
            len
        }

        async fn read<R>(stream: &mut R) -> Result<Self, MQTTError>
        where
            R: futures::AsyncReadExt + Unpin,
        {
            let mut packet = Self::default();
            packet.session_present = u8::read(stream).await? != 0;
            let reason = u8::read(stream).await?;
            packet.reason = ConnAckReasonCode::try_from(reason).map_err(|_| {
                MQTTError::UnknownData(format!("Unrecognized reason code: {reason}"))
            })?;
            packet.properties = ConnAckProperties::read(stream).await?;

            Ok(packet)
        }

        async fn write<W>(&self, stream: &mut W) -> Result<(), MQTTError>
        where
            W: futures::AsyncWriteExt + Unpin,
        {
            FixedHeader::new(PacketType::ConnAck, 0, self.length())
                .write(stream)
                .await?;

            u8::from(self.session_present).write(stream).await?;
            (self.reason as u8).write(stream).await?;
            self.properties.write(stream).await?;

            Ok(())
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use bytes::{Bytes, BytesMut};

//     use super::*;

//     #[test]
//     fn read_write_connack() {
//         let packet = ConnAck {
//             session_present: true,
//             reason: ConnAckReasonCode::Success,
//             properties: ConnAckProperties {
//                 session_expiry_interval: Some(3000),
//                 receive_maximum: Some(14000),
//                 maximum_qos: Some(false),
//                 retain_available: Some(true),
//                 maximum_packet_size: Some(65_536),
//                 assigned_client_id: Some(String::from("mqttea")),
//                 topic_alias_maximum: Some(3287),
//                 reason_string: None,
//                 user_property: vec![
//                     (String::from("key"), String::from("value")),
//                     (String::from("abcde"), String::from("fghij")),
//                 ],
//                 wildcard_subscription_available: Some(true),
//                 subscription_identifiers_available: None,
//                 shared_subscription_available: Some(true),
//                 server_keep_alive: Some(3_600),
//                 response_information: None,
//                 server_reference: None,
//                 authentication_method: Some(String::from("basic access authentication")),
//                 authentication_data: None,
//             },
//         };

//         let mut buf = BytesMut::with_capacity(200);
//         packet.write(&mut buf).unwrap();
//         let expected = b" c\x01\0`\x11\0\0\x0b\xb8!6\xb0$\0%\x01'\0\x01\0\0\x12\0\x08mqttea\"\x0c\xd7&\0\x03key\0\x05value&\0\x05abcde\0\x05fghij(\x01*\x01\x13\x0e\x10\x15\0\x1bbasic access authentication".to_vec();

//         let received = buf.to_vec();
//         assert_eq!(expected, received);

//         let mut read_buf = Bytes::from_iter(buf.to_vec());
//         let fixed_header = FixedHeader::read(&mut read_buf).unwrap();
//         assert_eq!(fixed_header.flags, None);
//         assert_eq!(fixed_header.packet_type, PacketType::ConnAck);
//         assert_eq!(fixed_header.remaining_length, packet.length());

//         let read_packet = ConnAck::read(&mut read_buf).unwrap();
//         assert_eq!(read_packet, packet);
//     }
// }
