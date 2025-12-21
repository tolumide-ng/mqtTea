use bytes::Bytes;
use mqttea_macros::Length;

use crate::v5::commons::{error::MQTTError, qos::QoS};
use crate::v5::traits::utils::Utils;

use super::{Property, ReadData};

#[derive(Length, Debug, Clone, Default, PartialEq, Eq)]
pub struct WillProperties {
    pub delay_interval: Option<u32>,
    pub payload_format_indicator: Option<u8>,
    pub message_expiry_interval: Option<u32>,
    pub content_type: Option<String>,
    pub response_topic: Option<String>,
    pub correlation_data: Option<Bytes>,
    pub user_property: Vec<(String, String)>,
}

#[derive(Debug, Clone, Length, Default, PartialEq, Eq)]
pub struct Will {
    #[bytes(ignore)]
    pub properties: WillProperties,
    #[bytes(no_id)]
    pub topic: String,
    #[bytes(no_id)]
    pub payload: Bytes,
    #[bytes(ignore)]
    pub(super) qos: QoS,
    #[bytes(ignore)]
    pub(super) retain: bool,
}

impl ReadData for WillProperties {
    fn read_data(data: &mut Bytes) -> Result<Self, MQTTError> {
        let mut properties = Self::default();

        loop {
            let property = Property::read(data)?;
            match property {
                Property::WillDelayInterval(value) => {
                    Self::try_update(&mut properties.delay_interval, value)(property)?
                }
                Property::PayloadFormatIndicator(value) => {
                    Self::try_update(&mut properties.payload_format_indicator, value)(property)?
                }
                Property::MessageExpiryInterval(value) => {
                    Self::try_update(&mut properties.message_expiry_interval, value)(property)?
                }
                Property::ContentType(ref value) => Self::try_update(
                    &mut properties.content_type,
                    value.as_deref().map(|x| String::from(x)),
                )(property)?,
                Property::ResponseTopic(ref value) => Self::try_update(
                    &mut properties.response_topic,
                    value.as_deref().map(|x| String::from(x)),
                )(property)?,
                Property::CorrelationData(ref value) => Self::try_update(
                    &mut properties.correlation_data,
                    value.as_deref().map(|x| Bytes::from_iter(x.to_vec())),
                )(property)?,
                Property::UserProperty(value) => {
                    properties.user_property.push(value.into_owned());
                }
                p => return Err(MQTTError::UnexpectedProperty(p.to_string(), "".to_string())),
            }

            if data.is_empty() {
                break;
            }
        }

        Ok(properties)
    }
}

impl ReadData for Will {}

#[cfg(feature = "asyncx")]
pub(crate) use asyncx::*;
#[cfg(feature = "syncx")]
pub(crate) use syncx::*;

mod syncx {
    use std::borrow::Cow;

    use bytes::Bytes;

    use crate::v5::{
        commons::{error::MQTTError, property::Property},
        traits::{
            bufferio::BufferIO,
            read_data::ReadData,
            syncx::{read::Read, write::Write},
        },
    };

    use super::{Will, WillProperties};

    impl BufferIO for WillProperties {
        fn length(&self) -> usize {
            self.len()
        }

        fn write(&self, buf: &mut bytes::BytesMut) -> Result<(), MQTTError> {
            self.encode(buf)?; // 3.1.3.2.1

            Property::WillDelayInterval(self.delay_interval).write(buf)?; // 3.1.3.2.2
            Property::PayloadFormatIndicator(self.payload_format_indicator).write(buf)?; // 3.1.3.2.3
            Property::MessageExpiryInterval(self.message_expiry_interval).write(buf)?; // 3.1.3.2.4
            Property::ContentType(self.content_type.as_deref().map(Cow::Borrowed)).write(buf)?; // 3.1.3.2.5
            Property::ResponseTopic(self.response_topic.as_deref().map(Cow::Borrowed))
                .write(buf)?; // 3.1.3.2.6
            Property::CorrelationData(self.correlation_data.as_deref().map(Cow::Borrowed))
                .write(buf)?; // 3.1.3.2.7
            self.user_property
                .iter()
                .try_for_each(|kv| Property::UserProperty(Cow::Borrowed(kv)).write(buf))?; // 3.1.3.2.8

            Ok(())
        }

        fn read(buf: &mut Bytes) -> Result<Self, MQTTError> {
            let (length, _) = Self::decode(buf)?;
            let properties = Self::default();

            if length == 0 {
                return Ok(properties);
            } else if length > buf.len() {
                return Err(MQTTError::IncompleteData(
                    "WillProperties",
                    length,
                    buf.len(),
                ));
            };

            let mut data = buf.split_to(length);

            Self::read_data(&mut data)
        }
    }

    impl BufferIO for Will {
        fn write(&self, buf: &mut bytes::BytesMut) -> Result<(), MQTTError> {
            self.properties.write(buf)?;
            self.topic.write(buf); // 3.1.3.3
            self.payload.write(buf); // 3.1.3.4
            Ok(())
        }

        fn length(&self) -> usize {
            self.len() + self.properties.variable_length() + self.properties.length()
        }

        fn read(buf: &mut Bytes) -> Result<Self, MQTTError> {
            let mut will = Self::default();

            will.properties = WillProperties::read(buf)?;
            will.topic = String::read(buf)?;
            will.payload = Bytes::read(buf)?;
            Ok(will)
        }
    }
}

mod asyncx {
    use std::borrow::Cow;

    use bytes::Bytes;

    use crate::v5::commons::property::Property;
    use crate::v5::{
        commons::error::MQTTError,
        traits::{
            asyncx::{read::Read, write::Write},
            {read_data::ReadData, streamio::StreamIO},
        },
    };

    use super::{Will, WillProperties};

    impl StreamIO for WillProperties {
        fn length(&self) -> usize {
            self.len()
        }

        async fn read<R>(stream: &mut R) -> Result<Self, MQTTError>
        where
            R: futures::AsyncReadExt + Unpin,
        {
            let Some(len) = Self::parse_len(stream).await? else {
                return Ok(Self::default());
            };

            let mut data = Vec::with_capacity(len);
            stream.read_exact(&mut data).await?;

            let mut data = Bytes::copy_from_slice(&data);

            Self::read_data(&mut data)
        }

        async fn write<W>(&self, stream: &mut W) -> Result<(), MQTTError>
        where
            W: futures::AsyncWriteExt + Unpin,
        {
            self.encode(stream).await?;

            Property::WillDelayInterval(self.delay_interval)
                .write(stream)
                .await?; // 3.1.3.2.2
            Property::PayloadFormatIndicator(self.payload_format_indicator)
                .write(stream)
                .await?; // 3.1.3.2.3
            Property::MessageExpiryInterval(self.message_expiry_interval)
                .write(stream)
                .await?; // 3.1.3.2.4
            Property::ContentType(self.content_type.as_deref().map(Cow::Borrowed))
                .write(stream)
                .await?; // 3.1.3.2.5
            Property::ResponseTopic(self.response_topic.as_deref().map(Cow::Borrowed))
                .write(stream)
                .await?; // 3.1.3.2.6
            Property::CorrelationData(self.correlation_data.as_deref().map(Cow::Borrowed))
                .write(stream)
                .await?; // 3.1.3.2.7
            for kv in &self.user_property {
                Property::UserProperty(Cow::Borrowed(kv))
                    .write(stream)
                    .await?; // 3.1.3.2.8
            }

            Ok(())
        }
    }

    impl StreamIO for Will {
        fn length(&self) -> usize {
            self.len() + self.properties.variable_length() + self.properties.length()
        }

        async fn write<W>(&self, stream: &mut W) -> Result<(), MQTTError>
        where
            W: futures::AsyncWriteExt + Unpin,
        {
            self.properties.write(stream).await?;
            self.topic.write(stream).await?;
            self.payload.to_vec().write(stream).await?;

            Ok(())
        }

        async fn read<R>(stream: &mut R) -> Result<Self, MQTTError>
        where
            R: futures::AsyncReadExt + Unpin,
        {
            let mut will = Self::default();

            will.properties = WillProperties::read(stream).await?;
            will.topic = String::read(stream).await?;
            will.payload = Bytes::from_iter(Vec::read(stream).await?);

            Ok(will)
        }
    }
}
