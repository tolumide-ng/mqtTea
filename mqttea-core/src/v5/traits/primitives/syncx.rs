use bytes::{Bytes, BytesMut};

use crate::v5::{commons::error::MQTTError, traits::primitives::codec::BinaryCodec};

pub trait Read: Sized {
    fn read(buf: &mut Bytes) -> Result<Self, MQTTError>;
}

impl<T: BinaryCodec> Read for T {
    fn read(buf: &mut Bytes) -> Result<Self, MQTTError> {
        T::read_from(buf)
    }
}

pub trait Write {
    fn write(&self, buf: &mut BytesMut);
}

impl<T: BinaryCodec> Write for T {
    fn write(&self, buf: &mut BytesMut) {
        let _ = self.write_to(buf);
    }
}
