use crate::v5::commons::error::MQTTError;

pub trait ByteRead {
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), MQTTError>;
}

pub trait ByteWrite {
    fn write_all(&mut self, buf: &[u8]) -> Result<(), MQTTError>;
}
