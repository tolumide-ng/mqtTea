use std::future::Future;

use futures::AsyncWriteExt;

use crate::v5::commons::error::MQTTError;

pub(crate) trait Write<S>: Sized {
    fn write(&self, stream: &mut S) -> impl Future<Output = Result<(), MQTTError>>;
}

impl<S> Write<S> for u8
where
    S: AsyncWriteExt + Unpin,
{
    async fn write(&self, stream: &mut S) -> Result<(), MQTTError> {
        stream.write_all(&self.to_be_bytes()).await?;
        Ok(())
    }
}

impl<S> Write<S> for u16
where
    S: AsyncWriteExt + Unpin,
{
    async fn write(&self, stream: &mut S) -> Result<(), MQTTError> {
        stream.write_all(&self.to_be_bytes()).await?;
        Ok(())
    }
}

impl<S> Write<S> for u32
where
    S: AsyncWriteExt + Unpin,
{
    async fn write(&self, stream: &mut S) -> Result<(), MQTTError> {
        stream.write_all(&self.to_be_bytes()).await?;
        Ok(())
    }
}

impl<S> Write<S> for Vec<u8>
where
    S: AsyncWriteExt + Unpin,
{
    async fn write(&self, stream: &mut S) -> Result<(), MQTTError> {
        stream.write_all(&(self.len() as u16).to_be_bytes()).await?;
        stream.write_all(&self).await?;
        Ok(())
    }
}

impl<S> Write<S> for String
where
    S: AsyncWriteExt + Unpin,
{
    async fn write(&self, stream: &mut S) -> Result<(), MQTTError> {
        stream.write_all(&(self.len() as u16).to_be_bytes()).await?;
        stream.write_all(&self.as_bytes()).await?;
        Ok(())
    }
}

impl<S> Write<S> for (String, String)
where
    S: AsyncWriteExt + Unpin,
{
    async fn write(&self, stream: &mut S) -> Result<(), MQTTError> {
        self.0.write(stream).await?;
        self.1.write(stream).await
    }
}
