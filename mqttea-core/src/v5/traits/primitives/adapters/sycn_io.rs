use bytes::{Buf, BufMut, Bytes, BytesMut};

use crate::v5::{
    commons::error::MQTTError,
    traits::primitives::io::{ByteRead, ByteWrite},
};

impl ByteRead for Bytes {
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), crate::v5::commons::error::MQTTError> {
        if self.len() < buf.len() {
            return Err(MQTTError::IncompleteData("buffer", buf.len(), self.len()));
        }
        self.copy_to_slice(buf);
        Ok(())
    }
}

impl ByteWrite for BytesMut {
    fn write_all(&mut self, buf: &[u8]) -> Result<(), MQTTError> {
        self.put_slice(buf);
        Ok(())
    }
}
