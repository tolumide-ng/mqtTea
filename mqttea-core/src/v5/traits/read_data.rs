use crate::v5::commons::error::MQTTError;
use bytes::Bytes;

pub(crate) trait ReadData: Sized {
    fn read_data(data: &mut Bytes) -> Result<Self, MQTTError> {
        Err(MQTTError::UnImplemented)
    }
}
