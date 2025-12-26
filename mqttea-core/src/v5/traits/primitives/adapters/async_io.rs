use futures::{AsyncReadExt, AsyncWriteExt};

use crate::v5::traits::primitives::io::{ByteRead, ByteWrite};

pub struct AsyncReader<'a, R: AsyncReadExt + Unpin> {
    pub inner: &'a mut R,
}

pub struct AsyncWriter<'a, W: AsyncWriteExt + Unpin> {
    pub inner: &'a mut W,
}

impl<'a, R: AsyncReadExt + Unpin> ByteRead for AsyncReader<'a, R> {
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), crate::v5::commons::error::MQTTError> {
        futures::executor::block_on(self.inner.read_exact(buf))
    }
}

impl<'a, W: AsyncWriteExt + Unpin> ByteWrite for AsyncWriter<'a, W> {
    fn write_all(&mut self, buf: [u8]) -> Result<(), crate::v5::commons::error::MQTTError> {
        futures::executor::block_on(self.inner.write_all(buf))
    }
}
