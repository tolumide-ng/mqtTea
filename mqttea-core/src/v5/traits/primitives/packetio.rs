use crate::v5::{
    commons::error::MQTTError,
    traits::primitives::{
        io::{ByteRead, ByteWrite},
        varint::VarInt,
    },
};

pub trait PacketIO: Sized + VarInt {
    /// Remaining length (MQTT spec)
    fn rem_len(&self) -> usize;

    /// Write packet body
    fn write_body<W: ByteWrite>(&self, w: &mut W) -> Result<(), MQTTError>;

    /// Read packet body
    fn read_body<R: ByteRead>(r: &mut R, len: usize) -> Result<Self, MQTTError>;

    // /// Encode Remaining Length + body (NO length argument needed)
    // fn encode<W: ByteWrite>(&self, w: &mut W) -> Result<(), MQTTError> {
    //     // self.
    // }

    fn variable_length(&self) -> usize {
        self.length()
    }
}
