use bytes::Bytes;
use mqttea_macros::{FromU8, Length};

use crate::v5::{
    commons::{error::MQTTError, property::Property},
    traits::utils::Utils,
};

use super::ReadData;

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, FromU8, Default)]
pub enum PubRecReasonCode {
    #[default]
    Success = 0,
    NoMatchingSubscribers = 16,
    UnspecifiedError = 128,
    ImplementationSpecificError = 131,
    NotAuthorized = 135,
    TopicNameInvalid = 144,
    PacketIdentifierInUse = 145,
    QuotaExceeded = 151,
    PayloadFormatIndicator = 153,
}

#[derive(Debug, Length, Default, PartialEq, Eq)]
pub struct PubRecProperties {
    pub reason_string: Option<String>,
    pub user_property: Vec<(String, String)>,
}

#[cfg(feature = "asyncx")]
pub(crate) use asyncx::*;
#[cfg(feature = "syncx")]
pub(crate) use syncx::*;

impl ReadData for PubRecProperties {
    fn read_data(data: &mut Bytes) -> Result<Self, MQTTError> {
        let mut props = Self::default();

        loop {
            let property = Property::read(data)?;

            match property {
                Property::ReasonString(ref v) => Self::try_update(
                    &mut props.reason_string,
                    v.as_deref().map(String::from),
                )(property)?,
                Property::UserProperty(value) => props.user_property.push(value.into_owned()),
                p => return Err(MQTTError::UnexpectedProperty(p.to_string(), "".to_string())),
            };

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

    use super::PubRecProperties;

    impl BufferIO for PubRecProperties {
        fn length(&self) -> usize {
            self.len()
        }

        fn write(&self, buf: &mut bytes::BytesMut) -> Result<(), MQTTError> {
            self.encode(buf)?;

            Property::ReasonString(self.reason_string.as_deref().map(Cow::Borrowed)).write(buf)?;
            self.user_property
                .iter()
                .try_for_each(|kv| Property::UserProperty(Cow::Borrowed(kv)).write(buf))?;

            Ok(())
        }
    }
}

mod asyncx {
    use std::borrow::Cow;

    use crate::v5::{commons::property::Property, traits::streamio::StreamIO};

    use super::PubRecProperties;

    impl StreamIO for PubRecProperties {
        fn length(&self) -> usize {
            self.len()
        }

        async fn write<W>(&self, stream: &mut W) -> Result<(), crate::v5::commons::error::MQTTError>
        where
            W: futures::AsyncWriteExt + Unpin,
        {
            self.encode(stream).await?;

            Property::ReasonString(self.reason_string.as_deref().map(Cow::Borrowed))
                .write(stream)
                .await?;
            for kv in &self.user_property {
                Property::UserProperty(Cow::Borrowed(kv))
                    .write(stream)
                    .await?;
            }

            Ok(())
        }
    }
}
