use bytes::Bytes;

use crate::v5::commons::{error::MQTTError, property::Property};
use crate::v5::traits::syncx::read::Read;

use super::bufferio::BufferIO;
use super::streamio::StreamIO;

pub(crate) trait Utils: Sized {
    fn try_update<T>(
        field: &mut Option<T>,
        value: Option<T>,
    ) -> impl Fn(Property) -> Result<(), MQTTError> {
        let is_duplicate = field.is_some();
        *field = value;

        move |ppt| {
            if is_duplicate {
                return Err(MQTTError::DuplicateProperty(ppt.to_string()));
            }
            return Ok(());
        }
    }

    /// Decodes a Variable byte Integer
    fn decode(buf: &mut Bytes) -> Result<(usize, usize), MQTTError> {
        let mut result = 0;

        for i in 0..4 {
            if buf.is_empty() {
                return Err(MQTTError::MalformedPacket);
            }
            let byte = u8::read(buf)?;

            result += ((byte as usize) & 0x7F) << (7 * i);

            if (byte & 0x80) == 0 {
                return Ok((result, i + 1));
            }
        }

        return Err(MQTTError::MalformedPacket);
    }

    fn validate_topic(&self, topic: &String) -> Result<(), MQTTError> {
        if topic.contains("+") || topic.contains("#") {
            return Err(MQTTError::InvalidTopic("invalid character"));
        }

        Ok(())
    }
}

impl<T: StreamIO + BufferIO> Utils for T {}
