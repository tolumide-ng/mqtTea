use crate::v5::{
    commons::error::MQTTError,
    traits::primitives::io::{ByteRead, ByteWrite},
};

pub(crate) trait VarInt {
    /// Allows a struct specify what it's length is to it's external users
    /// Normally this is obtainable using the .len() method (internally on structs implementing Length(formerly DataSize)),
    /// However, this method allows the struct customize what its actual length is.
    /// NOTE: The eventual plan is to make this the only property accessible externally and
    ///     make `.len()` internal while probably enforcing that all struct's implementing this method/trait
    ///     must also implement `DataSize` proc. So that there is a default accurate length property
    fn length(&self) -> usize {
        0
    }

    fn encode<W: ByteWrite>(&self, w: &mut W) -> Result<i32, MQTTError> {
        let mut len = self.length();

        if len > 0x0FFF_FFFF {
            return Err(MQTTError::PayloadTooLong);
        }

        let mut written = 0;

        for _ in 0..4 {
            let mut byte = (len % 128) as u8;
            len /= 128;

            if len > 0 {
                byte |= 0x80;
            }

            w.write_all(&[byte]);
            written += 1;

            if len == 0 {
                break;
            }
        }

        Ok(written)
    }

    fn decode<R: ByteRead>(r: &mut R) -> Result<(usize, usize), MQTTError> {
        let mut value = 0usize;

        for i in 0..4 {
            let mut buf = [0u8; 1];
            r.read_exact(&mut buf)?;

            let byte = buf[0];
            value |= ((byte & 0x7F) as usize) << (7 * i);

            if (byte & 0x80) == 0 {
                return Ok((value, i + 1));
            }
        }

        Err(MQTTError::MalformedPacket)
    }

    fn encoded_len(len: usize) -> usize {
        match len {
            0..=127 => 1,
            128..=16_383 => 2,
            16_384..=2_097_151 => 3,
            _ => 4,
        }
    }

    // fn read_with_fixedheader(_buf: &mut Bytes, _header: FixedHeader) -> Result<Self, MQTTError> {
    //     Err(MQTTError::MalformedPacket)
    // }
}
