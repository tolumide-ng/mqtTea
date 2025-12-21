use bytes::{Buf, Bytes};

use crate::v5::commons::error::MQTTError;

pub(crate) trait Read: Sized {
    fn read(buf: &mut Bytes) -> Result<Self, MQTTError>;
}


impl Read for u8 {
    fn read(buf: &mut Bytes) -> Result<Self, MQTTError> {
        let len = std::mem::size_of::<u8>();
        if buf.is_empty() { return Err(MQTTError::IncompleteData("u8", len, buf.len()))}
        
        Ok(buf.get_u8())
    }
}

impl Read for u16 {
    fn read(buf: &mut Bytes) -> Result<Self, MQTTError> {
        let len = std::mem::size_of::<u16>();

        if buf.len() < len { return Err(MQTTError::IncompleteData("u16", len, buf.len()))}
        
        Ok(buf.get_u16())
    }
}

impl Read for u32 {
    fn read(buf: &mut Bytes) -> Result<Self, MQTTError> {
        let len = std::mem::size_of::<u32>();
        if buf.len() < len { return Err(MQTTError::IncompleteData("u32", len, buf.len()))}
        
        Ok(buf.get_u32())
    }
}


impl Read for String {
    fn read(buf: &mut Bytes) -> Result<Self, MQTTError> {
        let data = Bytes::read(buf)?;

        match String::from_utf8(data.to_vec()) {
            Ok(d) => Ok(d),
            Err(e) => Err(MQTTError::Utf8Error(e)), // should be Malformed packet see 1.5.4 (UTF-8 Encoded String)
        }
    }
}


impl Read for Bytes {
    fn read(buf: &mut Bytes) -> Result<Self, MQTTError> {
        let len = buf.get_u16() as usize;

        if len > buf.len() {
            return Err(MQTTError::IncompleteData("Bytes", len, buf.len()))
        }
        Ok(buf.split_to(len))
    }
}