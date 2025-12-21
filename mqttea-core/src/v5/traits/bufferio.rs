use crate::v5::{
    commons::fixed_header::FixedHeader,
    traits::syncx::{read::Read, write::Write},
};
use bytes::{Bytes, BytesMut};

use crate::v5::commons::error::MQTTError;

use super::read_data::ReadData;

pub(crate) trait BufferIO: Sized + ReadData {
    /// Encodes a non-negative Integer into the Variable Byte Integer encoding
    fn encode(&self, buf: &mut BytesMut) -> Result<(), MQTTError> {
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

            (byte as u8).write(buf); // writes the encoded byte into the buffer
            if len == 0 {
                break;
            }
        }
        Ok(())
    }

    /// Decodes a Variable byte Inetger
    fn decode(buf: &mut Bytes) -> Result<(usize, usize), MQTTError> {
        let mut result = 0;

        for i in 0..4 {
            if buf.is_empty() {
                return Err(MQTTError::MalformedPacket);
            }
            let byte = u8::read(buf)?;

            result += ((byte as usize) & 0x7F) << (7 * i);

            if (byte & 0x80) == 0 {
                return Ok((result, i + 1));
            }
        }

        return Err(MQTTError::MalformedPacket);
    }

    /// Allows a struct specify what it's length is to it's external users
    /// Normally this is obtainable using the .len() method (internally on structs implementing Length(formerly DataSize)),
    /// However, this method allows the struct customize what its actual length is.
    /// NOTE: The eventual plan is to make this the only property accessible externally and
    ///     make `.len()` internal while probably enforcing that all struct's implementing this method/trait
    ///     must also implement `DataSize` proc. So that there is a default accurate length property
    fn length(&self) -> usize {
        0
    }

    fn write(&self, buf: &mut BytesMut) -> Result<(), MQTTError> {
        Ok(())
    }

    fn read(buf: &mut Bytes) -> Result<Self, MQTTError>
    where
        Self: Default,
    {
        let Some(len) = Self::parse_len(buf)? else {
            return Ok(Self::default());
        };

        let mut data = buf.split_to(len);
        Self::read_data(&mut data)
    }

    fn read_with_fixedheader(_buf: &mut Bytes, _header: FixedHeader) -> Result<Self, MQTTError> {
        Err(MQTTError::MalformedPacket)
    }

    fn parse_len(buf: &mut Bytes) -> Result<Option<usize>, MQTTError>
    where
        Self: Default,
    {
        let (len, _) = Self::decode(buf)?;

        if len == 0 {
            return Ok(None);
        }
        if len > buf.len() {
            return Err(MQTTError::IncompleteData("", len, buf.len()));
        };

        Ok(Some(len))
    }

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
}
