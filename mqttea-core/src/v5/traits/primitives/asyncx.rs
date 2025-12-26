use futures::{AsyncReadExt, AsyncWriteExt};

use crate::v5::{
    commons::error::MQTTError,
    traits::primitives::{
        adapters::async_io::{AsyncReader, AsyncWriter},
        codec::BinaryCodec,
    },
};

pub trait ReadAsync<S>: Sized {
    fn read(stream: &mut S) -> impl futures::Future<Output = Result<Self, MQTTError>>;
}

impl<T, S> ReadAsync<S> for T
where
    T: BinaryCodec,
    S: AsyncReadExt + Unpin,
{
    async fn read(stream: &mut S) -> Result<Self, MQTTError> {
        let mut reader = AsyncReader { inner: stream };
        T::read_from(&mut reader)
    }
}

pub trait WriteAsync<S> {
    fn write(&self, stream: &mut S) -> impl std::future::Future<Output = Result<(), MQTTError>>;
}

impl<T, S> WriteAsync<S> for T
where
    T: BinaryCodec,
    S: AsyncWriteExt + Unpin,
{
    async fn write(&self, stream: &mut S) -> Result<(), MQTTError> {
        let mut writer = AsyncWriter { inner: stream };
        self.write_to(&mut writer)
    }
}
