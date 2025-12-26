use bytes::Bytes;

use crate::v5::{
    commons::error::MQTTError,
    traits::primitives::io::{ByteRead, ByteWrite},
};

pub trait BinaryCodec: Sized {
    fn read_from<R: ByteRead>(r: &mut R) -> Result<Self, MQTTError>;
    fn write_to<W: ByteWrite>(&self, w: &mut W) -> Result<(), MQTTError>;
}

impl BinaryCodec for u8 {
    fn read_from<R: ByteRead>(r: &mut R) -> Result<Self, MQTTError> {
        let mut buf = [0u8; 1];
        r.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    fn write_to<W: ByteWrite>(&self, w: &mut W) -> Result<(), MQTTError> {
        w.write_all(&[*self])
    }
}

impl BinaryCodec for u16 {
    fn read_from<R: ByteRead>(r: &mut R) -> Result<Self, MQTTError> {
        let mut buf = [0u8; 2];
        r.read_exact(&mut buf)?;
        Ok(u16::from_be_bytes(buf))
    }

    fn write_to<W: ByteWrite>(&self, w: &mut W) -> Result<(), MQTTError> {
        w.write_all(&self.to_be_bytes())
    }
}

impl BinaryCodec for u32 {
    fn read_from<R: ByteRead>(r: &mut R) -> Result<Self, MQTTError> {
        let mut buf = [0u8; 4];
        r.read_exact(&mut buf)?;
        Ok(u32::from_be_bytes(buf))
    }

    fn write_to<W: ByteWrite>(&self, w: &mut W) -> Result<(), MQTTError> {
        w.write_all(&self.to_be_bytes())
    }
}

impl BinaryCodec for String {
    fn read_from<R: ByteRead>(r: &mut R) -> Result<Self, MQTTError> {
        let len = u16::read_from(r)? as usize;
        let mut buf = vec![0u8; len];
        r.read_exact(&mut buf)?;
        String::from_utf8(buf).map_err(MQTTError::Utf8Error)
    }

    fn write_to<W: ByteWrite>(&self, w: &mut W) -> Result<(), MQTTError> {
        let bytes = self.as_bytes();
        (bytes.len() as u16).write_to(w)?;
        w.write_all(bytes)
    }
}

impl BinaryCodec for Bytes {
    fn read_from<R: ByteRead>(r: &mut R) -> Result<Self, MQTTError> {
        let len = u16::read_from(r)? as usize;
        let mut buf = vec![0u8; len];
        r.read_exact(&mut buf)?;
        Ok(Bytes::from(buf))
    }

    fn write_to<W: ByteWrite>(&self, w: &mut W) -> Result<(), MQTTError> {
        (self.len() as u16).write_to(w)?;
        w.write_all(self)
    }
}
