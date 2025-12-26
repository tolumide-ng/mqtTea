use crate::v5::traits::syncx::{read::Read as ByteRead, write::Write as ByteWrite};

pub(crate) struct VarInt;

impl VarInt {
    pub fn encode<W: ByteWrite>() {}
}
