use std::ops::Deref;

use bytes::Bytes;
use mqttea_macros::Length;

use crate::v5::{commons::error::MQTTError, traits::utils::Utils};

use super::{Property, ReadData};

#[derive(Debug, Length, Default, PartialEq, Eq)]
pub struct SubscribeProperties {
    pub subscription_id: Option<usize>,
    pub user_property: Vec<(String, String)>,
}

#[cfg(feature = "asyncx")]
pub(crate) use asyncx::*;
#[cfg(feature = "syncx")]
pub(crate) use syncx::*;

impl ReadData for SubscribeProperties {
    fn read_data(data: &mut Bytes) -> Result<Self, MQTTError> {
        let mut props = Self::default();

        loop {
            let property = Property::read(data)?;

            match property {
                Property::SubscriptionIdentifier(ref v) => {
                    Self::try_update(&mut props.subscription_id, Some(*v.deref()))(property)?
                }
                Property::UserProperty(v) => props.user_property.push(v.into_owned()),
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

    use super::SubscribeProperties;

    impl BufferIO for SubscribeProperties {
        fn length(&self) -> usize {
            self.len()
        }

        fn write(&self, buf: &mut bytes::BytesMut) -> Result<(), MQTTError> {
            self.encode(buf)?;

            if let Some(id) = &self.subscription_id {
                Property::SubscriptionIdentifier(Cow::Borrowed(id)).write(buf)?;
            }
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

    use super::SubscribeProperties;

    impl StreamIO for SubscribeProperties {
        fn length(&self) -> usize {
            self.len()
        }

        async fn write<W>(&self, stream: &mut W) -> Result<(), crate::v5::commons::error::MQTTError>
        where
            W: futures::AsyncWriteExt + Unpin,
        {
            self.encode(stream).await?;

            if let Some(id) = &self.subscription_id {
                Property::SubscriptionIdentifier(Cow::Borrowed(id))
                    .write(stream)
                    .await?;
            }

            for kv in &self.user_property {
                Property::UserProperty(Cow::Borrowed(kv))
                    .write(stream)
                    .await?;
            }

            Ok(())
        }
    }
}
