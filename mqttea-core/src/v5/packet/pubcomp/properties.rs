use bytes::Bytes;
use mqttea_macros::{FromU8, Length};

use crate::v5::{commons::error::MQTTError, traits::utils::Utils};

use super::{Property, ReadData};

#[derive(Debug, PartialEq, Eq, Default, FromU8, Clone, Copy)]
pub(crate) enum PubCompReasonCode {
    #[default]
    Success = 0,
    PacketIdentifierNotFound = 146,
}

#[derive(Debug, Length, Default, PartialEq, Eq, Clone)]
pub struct PubCompProperties {
    pub reason_string: Option<String>,
    pub user_property: Vec<(String, String)>,
}

impl ReadData for PubCompProperties {
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

    use super::PubCompProperties;

    impl BufferIO for PubCompProperties {
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

    use crate::v5::{
        commons::{error::MQTTError, property::Property},
        traits::streamio::StreamIO,
    };

    use super::PubCompProperties;

    impl StreamIO for PubCompProperties {
        fn length(&self) -> usize {
            self.len()
        }

        async fn write<W>(&self, stream: &mut W) -> Result<(), MQTTError>
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
