use std::borrow::Cow;

use bytes::Bytes;
use mqttea_macros::Length;

use crate::v5::{
    commons::error::MQTTError,
    traits::{read_data::ReadData, utils::Utils},
};

use super::Property;

/// CONNECT Properties (3.1.2.11)
#[derive(Debug, Clone, Length, Default, PartialEq, Eq)]
pub(crate) struct ConnectProperties {
    pub(crate) session_expiry_interval: Option<u32>,
    pub(crate) receive_maximum: Option<u16>,
    pub(crate) maximum_packet_size: Option<u32>,
    pub(crate) topic_alias_maximum: Option<u16>,
    pub(crate) request_response_information: Option<u8>,
    pub(crate) request_problem_information: Option<u8>,
    pub(crate) user_property: Vec<(String, String)>,
    pub(crate) authentication_method: Option<String>,
    pub(crate) authentication_data: Option<Bytes>,
}

#[cfg(feature = "asyncx")]
pub(crate) use asyncx::*;
#[cfg(feature = "syncx")]
pub(crate) use syncx::*;

impl ReadData for ConnectProperties {
    fn read_data(data: &mut Bytes) -> Result<Self, MQTTError> {
        let mut properties = Self::default();

        loop {
            let property = Property::read(data)?;
            match property {
                Property::SessionExpiryInterval(value) => {
                    Self::try_update(&mut properties.session_expiry_interval, value)(property)?
                }
                Property::ReceiveMaximum(value) => {
                    Self::try_update(&mut properties.receive_maximum, value)(property)?
                }
                Property::MaximumPacketSize(value) => {
                    Self::try_update(&mut properties.maximum_packet_size, value)(property)?;
                }
                Property::TopicAliasMaximum(value) => {
                    Self::try_update(&mut properties.topic_alias_maximum, value)(property)?;
                }
                Property::RequestResponseInformation(value) => {
                    Self::try_update(&mut properties.request_response_information, value)(property)?
                }
                Property::RequestProblemInformation(value) => {
                    Self::try_update(&mut properties.request_problem_information, value)(property)?
                }
                Property::UserProperty(value) => {
                    properties.user_property.push(value.into_owned());
                }
                Property::AuthenticationMethod(ref value) => Self::try_update(
                    &mut properties.authentication_method,
                    value.as_deref().map(|x| String::from(x)),
                )(property)?,
                Property::AuthenticationData(ref value) => Self::try_update(
                    &mut properties.authentication_data,
                    value.to_owned().map(|x| Bytes::from_iter(x.into_owned())),
                )(property)?,
                p => return Err(MQTTError::UnexpectedProperty(p.to_string(), "".to_string())),
            }
            if data.is_empty() {
                break;
            }
        }

        Ok(properties)
    }
}

mod syncx {
    use std::borrow::Cow;

    use crate::v5::{
        commons::{error::MQTTError, property::Property},
        traits::bufferio::BufferIO,
    };

    use super::ConnectProperties;

    impl BufferIO for ConnectProperties {
        /// The length of the Properties in the CONNECT packet Variable Header encoded as a Variable Byte Integer 3.1.2.11.1
        fn length(&self) -> usize {
            self.len()
        }

        fn write(&self, buf: &mut bytes::BytesMut) -> Result<(), MQTTError> {
            self.encode(buf)?; // 3.1.2.11.1 (Property Length)
            Property::SessionExpiryInterval(self.session_expiry_interval).write(buf)?;
            Property::ReceiveMaximum(self.receive_maximum).write(buf)?;
            Property::MaximumPacketSize(self.maximum_packet_size).write(buf)?;
            Property::TopicAliasMaximum(self.topic_alias_maximum).write(buf)?;
            Property::RequestResponseInformation(self.request_response_information).write(buf)?;
            Property::RequestProblemInformation(self.request_problem_information).write(buf)?;
            self.user_property
                .iter()
                .try_for_each(|kv| Property::UserProperty(Cow::Borrowed(kv)).write(buf))?;
            Property::AuthenticationMethod(
                self.authentication_method.as_deref().map(Cow::Borrowed),
            )
            .write(buf)?;
            Property::AuthenticationData(self.authentication_data.as_deref().map(Cow::Borrowed))
                .write(buf)?;

            Ok(())
        }
    }
}

mod asyncx {

    use std::borrow::Cow;

    use crate::v5::{commons::property::Property, traits::streamio::StreamIO};

    use super::ConnectProperties;

    impl StreamIO for ConnectProperties {
        /// The length of the Properties in the CONNECT packet Variable Header encoded as a Variable Byte Integer 3.1.2.11.1
        fn length(&self) -> usize {
            self.len()
        }

        async fn write<W>(&self, stream: &mut W) -> Result<(), crate::v5::commons::error::MQTTError>
        where
            W: futures::AsyncWriteExt + Unpin,
        {
            self.encode(stream).await?; // 3.1.2.11.1 (Property Length)
            Property::SessionExpiryInterval(self.session_expiry_interval)
                .write(stream)
                .await?;
            Property::ReceiveMaximum(self.receive_maximum)
                .write(stream)
                .await?;
            Property::MaximumPacketSize(self.maximum_packet_size)
                .write(stream)
                .await?;
            Property::TopicAliasMaximum(self.topic_alias_maximum)
                .write(stream)
                .await?;
            Property::RequestResponseInformation(self.request_response_information)
                .write(stream)
                .await?;
            Property::RequestProblemInformation(self.request_problem_information)
                .write(stream)
                .await?;
            for kv in &self.user_property {
                Property::UserProperty(Cow::Borrowed(kv))
                    .write(stream)
                    .await?;
            }
            Property::AuthenticationMethod(
                self.authentication_method.as_deref().map(Cow::Borrowed),
            )
            .write(stream)
            .await?;
            Property::AuthenticationData(self.authentication_data.as_deref().map(Cow::Borrowed))
                .write(stream)
                .await?;

            Ok(())
        }
    }
}
