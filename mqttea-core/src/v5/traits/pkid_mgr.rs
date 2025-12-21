use crate::v5::commons::error::MQTTError;

pub(crate) trait PacketIdRelease: Sized {
    fn release(&self, id: u16);

    fn is_occupied(&self, id: u16) -> bool;
}

pub(crate) trait PacketIdAlloc: Sized {
    fn allocate(&self) -> Result<u16, MQTTError>;
}
