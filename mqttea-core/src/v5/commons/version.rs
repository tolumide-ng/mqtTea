use super::error::MQTTError;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum Version {
    V4 = 0b0000_0100,
    #[default]
    V5 = 0b0000_0101
}

impl TryFrom<u8> for Version {
    type Error = MQTTError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0b0000_0100 => Ok(Self::V4),
            0b0000_0101 => Ok(Self::V5),
            v => Err(MQTTError::VersionNotSupported(v)),
        }
    }
}