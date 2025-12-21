mod properties;
pub mod will;

use mqttea_macros::Length;
use properties::ConnectProperties;
use will::Will;

use crate::{
    constants::PROTOCOL_NAME,
    v5::{
        client::ConnectOptions,
        commons::{
            error::MQTTError, fixed_header::FixedHeader, packet_type::PacketType,
            property::Property, qos::QoS, version::Version,
        },
        traits::read_data::ReadData,
    },
};

#[derive(Debug, Length, PartialEq, Eq)]
pub struct Connect {
    #[bytes(no_id)]
    pub(crate) client_id: String,
    #[bytes(no_id)]
    pub(crate) username: Option<String>,
    #[bytes(no_id)]
    pub(crate) password: Option<String>,
    #[bytes(ignore)]
    pub(crate) version: Version,
    #[bytes(ignore)]
    pub(crate) will: Option<Will>,
    #[bytes(ignore)]
    pub(crate) clean_start: bool,
    #[bytes(ignore)]
    pub(crate) keep_alive: u16,
    #[bytes(ignore)] // Connection properties
    pub(crate) properties: ConnectProperties,
}

#[cfg(feature = "asyncx")]
pub(crate) use asyncx::*;
#[cfg(feature = "syncx")]
pub(crate) use syncx::*;

impl Default for Connect {
    fn default() -> Self {
        Self {
            client_id: "uniqueId".into(),
            username: None,
            password: None,
            version: Version::V5,
            will: None,
            clean_start: true,
            keep_alive: 0,
            properties: ConnectProperties::default(),
        }
    }
}

impl From<&ConnectOptions> for Connect {
    fn from(value: &ConnectOptions) -> Self {
        Self {
            client_id: value.client_id.clone(),
            username: value.username.clone(),
            password: value.password.clone(),
            version: Version::V5,
            will: value.will.clone(),
            clean_start: value.clean_start,
            keep_alive: value.keep_alive,
            properties: ConnectProperties {
                session_expiry_interval: value.session_expiry_interval,
                receive_maximum: Some(value.client_receive_max.get()),
                maximum_packet_size: Some(value.client_max_size.get()),
                topic_alias_maximum: Some(value.inbound_topic_alias_max),
                request_response_information: value.request_response_information,
                request_problem_information: value.request_problem_information,
                user_property: value.user_property.clone(),
                authentication_method: value.authentication_method.clone(),
                authentication_data: value.authentication_data.clone(),
            },
        }
    }
}

impl ReadData for Connect {}

mod syncx {
    use bytes::Bytes;

    use crate::v5::{
        commons::error::MQTTError,
        traits::{
            bufferio::BufferIO,
            syncx::{read::Read, write::Write},
        },
    };

    use super::{
        will::Will, Connect, ConnectFlags, ConnectProperties, FixedHeader, PacketType, Version,
        PROTOCOL_NAME,
    };

    impl BufferIO for Connect {
        /// Length of the Variable Header + the length of the Payload
        fn length(&self) -> usize {
            let mut len: usize = (2 + PROTOCOL_NAME.len()) + 1 + 1 + 2; // version + connect flags + keep alive

            len += self.properties.length();
            len += self.properties.variable_length();
            if let Some(will) = &self.will {
                len += will.length()
            }
            len += self.len(); // client id + username + password

            len
        }

        fn write(&self, buf: &mut bytes::BytesMut) -> Result<(), MQTTError> {
            FixedHeader::new(PacketType::Connect, 0, self.length()).write(buf)?;

            (PROTOCOL_NAME.to_string()).write(buf);
            (self.version as u8).write(buf);

            let mut flags = ConnectFlags {
                clean_start: self.clean_start,
                password: self.password.is_some(),
                username: self.username.is_some(),
                ..Default::default()
            };

            if let Some(will) = &self.will {
                flags.will_retain = will.retain;
                flags.will_flag = true;
                flags.will_qos = will.qos;
            }

            u8::from(flags).write(buf); // 3.1.2.3
            self.keep_alive.write(buf); // 3.1.2.10
            self.properties.write(buf)?; // 3.1.2.11
                                         // CONNECT Payload: length-prefixed fields
            self.client_id.write(buf); // ClientId, willProperties, willTopic, willPayload, userName, password
            if let Some(will) = &self.will {
                will.write(buf)?;
            }
            if let Some(username) = &self.username {
                username.write(buf);
            } // 3.1.3.5
            if let Some(password) = &self.password {
                password.write(buf);
            } // 3.1.3.6

            Ok(())
        }

        fn read(buf: &mut Bytes) -> Result<Self, MQTTError> {
            // Assumption is that the fixed header as been read already
            String::read(buf).and_then(|x| {
                if x == "MQTT".to_string() {
                    return Ok(x);
                }
                return Err(MQTTError::MalformedPacket);
            })?;

            let mut packet = Self::default();
            packet.version = Version::try_from(u8::read(buf)?)?;

            let flags = ConnectFlags::try_from(u8::read(buf)?)?;
            packet.keep_alive = u16::read(buf)?;
            packet.properties = ConnectProperties::read(buf)?;
            packet.client_id = String::read(buf)?;

            if flags.will_flag {
                let mut will = Will::read(buf)?;
                will.retain = flags.will_retain;
                will.qos = flags.will_qos;
                packet.clean_start = flags.will_retain;
                packet.will = Some(will);
            }

            if flags.username {
                packet.username = Some(String::read(buf)?)
            };
            if flags.password {
                packet.password = Some(String::read(buf)?)
            };

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
        will::Will, Connect, ConnectFlags, ConnectProperties, FixedHeader, PacketType, Version,
        PROTOCOL_NAME,
    };

    impl StreamIO for Connect {
        /// Length of the Variable Header + the length of the Payload
        fn length(&self) -> usize {
            let mut len: usize = (2 + PROTOCOL_NAME.len()) + 1 + 1 + 2; // version + connect flags + keep alive

            len += self.properties.length();
            len += self.properties.variable_length();
            if let Some(will) = &self.will {
                len += will.length()
            }
            len += self.len(); // client id + username + password

            len
        }

        async fn write<W>(&self, stream: &mut W) -> Result<(), MQTTError>
        where
            W: futures::AsyncWriteExt + Unpin,
        {
            FixedHeader::new(PacketType::Connect, 0, self.length())
                .write(stream)
                .await?;

            (PROTOCOL_NAME.to_string()).write(stream).await?;
            (self.version as u8).write(stream).await?;

            let mut flags = ConnectFlags {
                clean_start: self.clean_start,
                password: self.password.is_some(),
                username: self.username.is_some(),
                ..Default::default()
            };

            if let Some(will) = &self.will {
                flags.will_retain = will.retain;
                flags.will_flag = true;
                flags.will_qos = will.qos;
            }

            u8::from(flags).write(stream).await?; // 3.1.2.3
            self.keep_alive.write(stream).await?; // 3.1.2.10
            self.properties.write(stream).await?; // 3.1.2.11
                                                  // CONNECT Payload: length-prefixed fields
            self.client_id.write(stream).await?; // ClientId, willProperties, willTopic, willPayload, userName, password
            if let Some(will) = &self.will {
                will.write(stream).await?;
            }
            if let Some(username) = &self.username {
                username.write(stream).await?;
            } // 3.1.3.5
            if let Some(password) = &self.password {
                password.write(stream).await?;
            } // 3.1.3.6

            Ok(())
        }

        async fn read<R>(stream: &mut R) -> Result<Self, crate::v5::commons::error::MQTTError>
        where
            R: futures::AsyncReadExt + Unpin,
        {
            // Assumption is that the fixed header as been read already
            String::read(stream).await.and_then(|x| {
                if x == "MQTT".to_string() {
                    return Ok(x);
                }
                return Err(MQTTError::MalformedPacket);
            })?;

            let mut packet = Self::default();
            packet.version = Version::try_from(u8::read(stream).await?)?;

            let flags = ConnectFlags::try_from(u8::read(stream).await?)?;
            packet.keep_alive = u16::read(stream).await?;
            packet.properties = ConnectProperties::read(stream).await?;
            packet.client_id = String::read(stream).await?;

            if flags.will_flag {
                let mut will = Will::read(stream).await?;
                will.retain = flags.will_retain;
                will.qos = flags.will_qos;
                packet.clean_start = flags.will_retain;
                packet.will = Some(will);
            }

            if flags.username {
                packet.username = Some(String::read(stream).await?)
            };
            if flags.password {
                packet.password = Some(String::read(stream).await?)
            };

            Ok(packet)
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ConnectFlags {
    pub username: bool,
    pub password: bool,
    pub will_retain: bool,
    pub will_qos: QoS,
    pub will_flag: bool,
    pub clean_start: bool,
}

impl From<ConnectFlags> for u8 {
    fn from(value: ConnectFlags) -> Self {
        let flags = u8::from(value.username) << 7
            | u8::from(value.password) << 6
            | u8::from(value.will_retain) << 5
            | u8::from(value.will_qos) << 3
            | u8::from(value.will_flag) << 2
            | u8::from(value.clean_start) << 1;
        flags
    }
}

impl TryFrom<u8> for ConnectFlags {
    type Error = MQTTError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let username = (value & (1 << 7)) != 0;
        let password = (value & (1 << 6)) != 0;
        let will_retain = (value & (1 << 5)) != 0;
        let qos = (value & (0b11 << 3)) >> 3;
        let will_qos = QoS::try_from(qos).map_err(|_| MQTTError::UnsupportedQoS(qos))?;
        let will_flag = (value & (1 << 2)) != 0;
        let clean_start = (value & (1 << 1)) != 0;

        Ok(Self {
            username,
            password,
            will_retain,
            will_qos,
            will_flag,
            clean_start,
        })
    }
}

// #[cfg(test)]
// mod connect_packet {
//     use super::*;
//     use crate::v5::commons::qos::QoS;
//     use bytes::BytesMut;
//     use std::io::Read;

//     #[cfg(test)]
//     mod write {
//         use will::WillProperties;

//         use super::*;
//         #[test]
//         fn create_connect_packet() -> Result<(), MQTTError> {
//             let mut buf = BytesMut::new();

//             Connect::default().write(&mut buf)?;
//             let expected = b"\x10\x15\0\x04MQTT\x05\x02\0\0\0\0\x08uniqueId"
//                 .as_ref()
//                 .to_vec();
//             let received = buf.bytes().flatten().collect::<Vec<u8>>();
//             assert_eq!(expected, received);

//             let mut connect = Connect::default();
//             connect.username = Some("username".into());
//             connect.password = Some("password".into());
//             connect.keep_alive = 170;
//             connect.will = Some(Will::default());
//             let will = connect.will.as_mut().unwrap();
//             will.topic = String::from("auto_warmup");
//             will.qos = QoS::Two;
//             will.properties = WillProperties::default();
//             will.payload = b"will payload".to_vec().into();

//             let mut buf = BytesMut::new();
//             connect.write(&mut buf)?;

//             let expected = b"\x10E\0\x04MQTT\x05\xd6\0\xaa\0\0\x08uniqueId\0\0\x0bauto_warmup\0\x0cwill payload\0\x08username\0\x08password".to_vec();

//             assert_eq!(expected, buf.to_vec());
//             Ok(())
//         }
//     }

//     #[cfg(test)]
//     mod read {
//         use super::*;

//         #[test]
//         fn read_connect_packet() {
//             let mut input = b"\0\x04MQTT\x05\xd6\0\xaa\0\0\x08uniqueId\0\0\x0bauto_warmup\0\x0cwill payload\0\x08username\0\x08password".as_ref().into();
//             let packet = Connect::read(&mut input).unwrap();

//             assert_eq!(packet.username.unwrap(), "username".to_string());
//             assert_eq!(packet.password.unwrap(), "password".to_string());
//             let will = packet.will.unwrap();
//             assert_eq!(will.qos, QoS::Two);
//             assert_eq!(will.retain, false);

//             assert_eq!(will.topic, "auto_warmup");
//             assert_eq!(packet.keep_alive, 170);
//             assert_eq!(will.payload, b"will payload".to_vec());
//             assert_eq!(will.retain, false);

//             assert_eq!(packet.version, Version::V5);
//             assert_eq!(packet.client_id, "uniqueId");
//             assert_eq!(packet.clean_start, false);
//             assert_eq!(packet.properties.authentication_data, None);
//             assert_eq!(packet.properties.authentication_method, None);
//             assert_eq!(packet.properties.receive_maximum, None);
//         }
//     }
// }
