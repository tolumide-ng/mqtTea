use bytes::Bytes;
use mqttea_macros::Length;

use crate::v5::{commons::error::MQTTError, traits::utils::Utils};

use super::{Property, ReadData};

#[derive(Debug, Length, Default, PartialEq, Eq)]
pub struct DisconnectProperties {
    pub session_expiry_interval: Option<u32>,
    pub reason_string: Option<String>,
    pub user_property: Vec<(String, String)>,
    pub server_reference: Option<String>,
}

#[cfg(feature = "asyncx")]
pub(crate) use asyncx::*;
#[cfg(feature = "syncx")]
pub(crate) use syncx::*;

impl ReadData for DisconnectProperties {
    fn read_data(data: &mut Bytes) -> Result<Self, MQTTError> {
        let mut props = Self::default();

        loop {
            let property = Property::read(data)?;
            match property {
                Property::ReasonString(ref v) => Self::try_update(
                    &mut props.reason_string,
                    v.as_deref().map(String::from),
                )(property)?,
                Property::UserProperty(v) => props.user_property.push(v.into_owned()),
                Property::SessionExpiryInterval(v) => {
                    Self::try_update(&mut props.session_expiry_interval, v)(property)?
                }
                Property::ServerReference(ref v) => Self::try_update(
                    &mut props.server_reference,
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

    use super::DisconnectProperties;

    impl BufferIO for DisconnectProperties {
        fn length(&self) -> usize {
            self.len()
        }

        fn write(&self, buf: &mut bytes::BytesMut) -> Result<(), MQTTError> {
            self.encode(buf)?;

            Property::SessionExpiryInterval(self.session_expiry_interval).write(buf)?;
            Property::ReasonString(self.reason_string.as_deref().map(Cow::Borrowed)).write(buf)?;
            self.user_property
                .iter()
                .try_for_each(|up| Property::UserProperty(Cow::Borrowed(&up)).write(buf))?;
            Property::ServerReference(self.server_reference.as_deref().map(Cow::Borrowed))
                .write(buf)?;

            Ok(())
        }
    }
}

mod asyncx {
    use std::borrow::Cow;

    use crate::v5::{commons::property::Property, traits::streamio::StreamIO};

    use super::DisconnectProperties;

    impl StreamIO for DisconnectProperties {
        fn length(&self) -> usize {
            self.len()
        }

        async fn write<W>(&self, stream: &mut W) -> Result<(), crate::v5::commons::error::MQTTError>
        where
            W: futures::AsyncWriteExt + Unpin,
        {
            self.encode(stream).await?;

            Property::SessionExpiryInterval(self.session_expiry_interval)
                .write(stream)
                .await?;

            Property::ReasonString(self.reason_string.as_deref().map(Cow::Borrowed))
                .write(stream)
                .await?;

            for kv in &self.user_property {
                Property::UserProperty(Cow::Borrowed(&kv))
                    .write(stream)
                    .await?
            }

            Property::ServerReference(self.server_reference.as_deref().map(Cow::Borrowed))
                .write(stream)
                .await?;

            Ok(())
        }
    }
}
