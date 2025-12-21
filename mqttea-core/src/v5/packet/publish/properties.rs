use bytes::Bytes;
use mqttea_macros::Length;

use crate::v5::{
    commons::{error::MQTTError, property::Property},
    traits::{read_data::ReadData, utils::Utils},
};

#[derive(Debug, Length, Default, Clone, PartialEq, Eq)]
pub struct PublishProperties {
    pub payload_format_indicator: Option<u8>,
    pub message_expiry_internal: Option<u32>,
    pub topic_alias: Option<u16>,
    /// the presence of a Response Topic identifies the Message as a Request
    pub response_topic: Option<String>,
    pub correlation_data: Option<Bytes>,
    pub user_property: Vec<(String, String)>,
    pub subscription_identifier: Vec<usize>,
    pub content_type: Option<String>,
}

#[cfg(feature = "asyncx")]
pub(crate) use asyncx::*;
#[cfg(feature = "syncx")]
pub(crate) use syncx::*;

impl ReadData for PublishProperties {
    fn read_data(data: &mut Bytes) -> Result<Self, MQTTError> {
        let mut props = Self::default();

        loop {
            let property = Property::read(data)?;

            match property {
                Property::PayloadFormatIndicator(value) => {
                    Self::try_update(&mut props.payload_format_indicator, value)(property)?
                }
                Property::MessageExpiryInterval(value) => {
                    Self::try_update(&mut props.message_expiry_internal, value)(property)?
                }
                Property::TopicAlias(value) => {
                    Self::try_update(&mut props.topic_alias, value)(property)?
                }
                Property::ResponseTopic(ref v) => Self::try_update(
                    &mut props.response_topic,
                    v.as_deref().map(String::from),
                )(property)?,
                // Property::CorrelationData(ref v) => Self::try_update(&mut props.correlation_data, v.to_owned().map(|x| Bytes::from_iter(x.into_owned())))(property)?,
                Property::CorrelationData(ref value) => Self::try_update(
                    &mut props.correlation_data,
                    value.as_deref().map(|x| Bytes::from_iter(x.to_vec())),
                )(property)?,
                Property::UserProperty(value) => props.user_property.push(value.into_owned()),
                Property::SubscriptionIdentifier(value) => {
                    props.subscription_identifier.push(value.into_owned())
                }
                Property::ContentType(ref v) => Self::try_update(
                    &mut props.content_type,
                    v.as_deref().map(String::from),
                )(property)?,
                p => return Err(MQTTError::UnexpectedProperty(p.to_string(), "".to_string())),
            }

            if data.is_empty() {
                break;
            }
        }

        Ok(props)
    }
}

mod syncx {
    use std::borrow::Cow;

    use crate::v5::{
        commons::{error::MQTTError, property::Property},
        traits::bufferio::BufferIO,
    };

    use super::PublishProperties;

    impl BufferIO for PublishProperties {
        fn length(&self) -> usize {
            self.len()
        }

        fn write(&self, buf: &mut bytes::BytesMut) -> Result<(), MQTTError> {
            self.encode(buf)?;

            Property::PayloadFormatIndicator(self.payload_format_indicator).write(buf)?;
            Property::MessageExpiryInterval(self.message_expiry_internal).write(buf)?;
            Property::TopicAlias(self.topic_alias).write(buf)?;
            Property::ResponseTopic(self.response_topic.as_deref().map(Cow::Borrowed))
                .write(buf)?;
            Property::CorrelationData(self.correlation_data.as_deref().map(Cow::Borrowed))
                .write(buf)?;
            self.user_property
                .iter()
                .try_for_each(|kv| Property::UserProperty(Cow::Borrowed(kv)).write(buf))?;
            self.subscription_identifier.iter().try_for_each(|si| {
                Property::SubscriptionIdentifier(Cow::Borrowed(&si)).write(buf)
            })?;
            Property::ContentType(self.content_type.as_deref().map(Cow::Borrowed)).write(buf)?;

            Ok(())
        }
    }
}

mod asyncx {
    use std::borrow::Cow;

    use crate::v5::{commons::property::Property, traits::streamio::StreamIO};

    use super::PublishProperties;

    impl StreamIO for PublishProperties {
        fn length(&self) -> usize {
            self.len()
        }

        async fn write<W>(&self, stream: &mut W) -> Result<(), crate::v5::commons::error::MQTTError>
        where
            W: futures::AsyncWriteExt + Unpin,
        {
            self.encode(stream).await?;

            Property::PayloadFormatIndicator(self.payload_format_indicator)
                .write(stream)
                .await?;
            Property::MessageExpiryInterval(self.message_expiry_internal)
                .write(stream)
                .await?;
            Property::TopicAlias(self.topic_alias).write(stream).await?;
            Property::ResponseTopic(self.response_topic.as_deref().map(Cow::Borrowed))
                .write(stream)
                .await?;
            Property::CorrelationData(self.correlation_data.as_deref().map(Cow::Borrowed))
                .write(stream)
                .await?;

            for kv in &self.user_property {
                Property::UserProperty(Cow::Borrowed(kv))
                    .write(stream)
                    .await?
            }
            for kv in &self.subscription_identifier {
                Property::SubscriptionIdentifier(Cow::Borrowed(&kv))
                    .write(stream)
                    .await?;
            }

            Property::ContentType(self.content_type.as_deref().map(Cow::Borrowed))
                .write(stream)
                .await?;

            Ok(())
        }
    }
}
