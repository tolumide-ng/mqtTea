use crate::v5::{
    commons::error::MQTTError,
    traits::{
        primitives::{
            codec::BinaryCodec,
            io::{ByteRead, ByteWrite},
            varint::VarInt,
        },
        read_data::ReadData,
    },
};

use super::packet_type::PacketType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct FixedHeader {
    pub(crate) packet_type: PacketType,
    pub(crate) flags: Option<u8>,
    /// Variable Byte Integer representing the number of bytes in the Variable Header and the Payload.
    pub(crate) remaining_length: usize,
    pub(crate) header_len: usize,
}

impl ReadData for FixedHeader {}

impl FixedHeader {
    pub(crate) fn new(packet_type: PacketType, flags: u8, remaining_length: usize) -> Self {
        Self {
            packet_type,
            flags: Some(flags).filter(|f| *f != 0),
            remaining_length,
            header_len: 0,
        }
    }

    pub(crate) fn with_len(&mut self, header_len: usize) -> Self {
        Self {
            header_len,
            ..*self
        }
    }
}

impl BinaryCodec for FixedHeader {
    fn write_to<W: ByteWrite>(&self, w: &mut W) -> Result<(), MQTTError> {
        let flags = self.flags.unwrap_or(0);
        let first_byte = self.packet_type as u8 | (flags);
        first_byte.write_to(w)?;
        self.remaining_length.encode(w)?;
        Ok(())
    }

    fn read_from<R: ByteRead>(r: &mut R) -> Result<Self, MQTTError> {
        Ok(())
    }
}

pub(crate) mod syncx {
    use crate::v5::commons::fixed_header::FixedHeader;
    use crate::v5::commons::packet_type::PacketType;
    use crate::v5::traits::bufferio::BufferIO;
    use crate::v5::{
        commons::error::MQTTError,
        traits::syncx::{read::Read, write::Write},
    };
    use bytes::{Bytes, BytesMut};

    impl BufferIO for FixedHeader {
        fn length(&self) -> usize {
            self.remaining_length
        }

        fn write(&self, buf: &mut BytesMut) -> Result<(), MQTTError> {
            // let f = self.flags.unwrap_or(0);
            ((self.packet_type as u8) | &self.flags.unwrap_or(0)).write(buf);
            self.encode(buf)?;

            Ok(())
        }

        fn read(buf: &mut Bytes) -> Result<Self, MQTTError> {
            if buf.len() < 2 {
                return Err(MQTTError::InsufficientBytes);
            }

            let byte0 = u8::read(buf)?;
            let packet = byte0 & 0b11110000;
            let packet_type = PacketType::try_from(packet).map_err(|_| {
                MQTTError::UnknownData(format!("Unexpected packet type: {}", packet))
            })?;

            let (remaining_length, header_len) = Self::decode(buf)?;

            Ok(Self {
                packet_type,
                flags: Some(byte0 & 0b00001111).filter(|n| *n != 0),
                remaining_length,
                header_len,
            })
        }
    }
}

pub(crate) mod asyncx {
    use crate::v5::commons::fixed_header::FixedHeader;
    use crate::v5::commons::packet_type::PacketType;
    use crate::v5::traits::streamio::StreamIO;
    use crate::v5::{
        commons::error::MQTTError,
        traits::asyncx::{read::Read, write::Write},
    };
    use futures::{AsyncReadExt, AsyncWriteExt};

    impl StreamIO for FixedHeader {
        fn length(&self) -> usize {
            0
        }

        async fn read<R>(stream: &mut R) -> Result<Self, MQTTError>
        where
            R: AsyncReadExt + Unpin,
        {
            let byte0 = u8::read(stream).await?;
            let packet = byte0 & 0b11110000;
            let packet_type = PacketType::try_from(packet).map_err(|_| {
                MQTTError::UnknownData(format!("Unexpected packet type: {}", packet))
            })?;

            let (remaining_length, header_len) = Self::decode(stream).await?;

            Ok(Self {
                packet_type,
                flags: Some(byte0 & 0b00001111).filter(|n| *n != 0),
                remaining_length,
                header_len,
            })
        }

        async fn write<W>(&self, stream: &mut W) -> Result<(), MQTTError>
        where
            W: AsyncWriteExt + Unpin,
        {
            let byte0 = (self.packet_type as u8) | &self.flags.unwrap_or(0);
            (byte0 as u8).write(stream).await?;

            self.encode(stream).await?;

            Ok(())
        }
    }
}
