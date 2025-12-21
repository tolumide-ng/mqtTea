use crate::v5::commons::error::MQTTError;
use crate::v5::commons::fixed_header::FixedHeader;
use crate::v5::traits::asyncx::read::Read;
use crate::v5::traits::asyncx::write::Write;

use bytes::Bytes;
use futures::{AsyncReadExt, AsyncWriteExt};

use super::read_data::ReadData;

pub(crate) trait StreamIO: Sized + ReadData {
    fn variable_length(&self) -> usize {
        let len = self.length();
        if len >= 2_097_152 {
            4
        } else if len >= 16_384 {
            3
        } else if len >= 128 {
            2
        } else {
            1
        }
    }
    /// Encodes a non-negative Integer into the Variable Byte Integer encoding
    async fn encode<T>(&self, stream: &mut T) -> Result<(), MQTTError>
    where
        T: AsyncWriteExt + Unpin,
    {
        let mut len = self.length();

        // 268_435_455
        if len > 0xFFFFFFF {
            return Err(MQTTError::PayloadTooLong);
        }

        for _ in 0..4 {
            let mut byte = len % 128;
            len /= 128;

            if len > 0 {
                byte |= 128;
            }

            (byte as u8).write(stream).await?;

            if len == 0 {
                break;
            }
        }

        Ok(())
    }

    /// Decodes a Variable byte Integer
    async fn decode<R>(stream: &mut R) -> Result<(usize, usize), MQTTError>
    where
        R: AsyncReadExt + Unpin,
    {
        let mut result = 0;

        for i in 0..4 {
            let byte = u8::read(stream).await?;

            result += ((byte as usize) & 0x7F) << (7 * i);

            if (byte & 0x80) == 0 {
                return Ok((result, i + 1));
            }
        }

        return Err(MQTTError::MalformedPacket);
    }

    fn length(&self) -> usize {
        0
    }

    async fn write<W>(&self, stream: &mut W) -> Result<(), MQTTError>
    where
        W: AsyncWriteExt + Unpin,
    {
        Ok(())
    }

    async fn read<R>(stream: &mut R) -> Result<Self, MQTTError>
    where
        R: futures::AsyncReadExt + Unpin,
        Self: Default,
    {
        let Some(len) = Self::parse_len(stream).await? else {
            return Ok(Self::default());
        };

        let mut data = Vec::with_capacity(len);
        stream.read_exact(&mut data).await?;
        let mut data = Bytes::copy_from_slice(&data);

        Self::read_data(&mut data)
    }

    async fn read_with_fixedheader<R>(
        stream: &mut R,
        header: &FixedHeader,
    ) -> Result<Self, MQTTError>
    where
        R: AsyncReadExt + Unpin,
        Self: Default,
    {
        Ok(Self::default())
    }

    async fn parse_len<R>(stream: &mut R) -> Result<Option<usize>, MQTTError>
    where
        Self: Default,
        R: AsyncReadExt + Unpin,
    {
        let (len, _) = Self::decode(stream).await?;
        if len == 0 {
            return Ok(None);
        }
        Ok(Some(len))
    }

    fn is_valid(&self, max_size: usize) -> Result<(), MQTTError> {
        let len = self.length();
        if len > max_size {
            return Err(MQTTError::MaxPacketSizeExceed(len));
        }

        Ok(())
    }
}
