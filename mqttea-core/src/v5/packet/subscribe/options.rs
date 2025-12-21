use crate::v5::commons::error::MQTTError;
use crate::v5::commons::qos::QoS;

use super::ReadData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SubscriptionOptions {
    pub qos: QoS,
    pub no_local: bool,
    pub retain_as_published: bool,
    pub retain_handling: RetainHandling,
}

#[cfg(feature = "asyncx")]
pub(crate) use asyncx::*;
#[cfg(feature = "syncx")]
pub(crate) use syncx::*;

impl ReadData for SubscriptionOptions {}

impl From<SubscriptionOptions> for u8 {
    fn from(v: SubscriptionOptions) -> Self {
        u8::from(v.qos)
            | u8::from(v.no_local) << 2
            | u8::from(v.retain_as_published) << 3
            | (v.retain_handling as u8) << 4
    }
}

impl TryFrom<u8> for SubscriptionOptions {
    type Error = MQTTError;

    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        let qos = byte & 0b0000_0011;
        let qos = QoS::try_from(qos).map_err(|_| MQTTError::UnsupportedQoS(qos))?;
        let no_local = (byte & 0b0000_0100) != 0;
        let retain_as_published = (byte & 0b0000_1000) != 0;
        let retain_handling = RetainHandling::try_from(byte & 0b0011_0000)?;

        Ok(Self {
            qos,
            no_local,
            retain_as_published,
            retain_handling,
        })
    }
}

mod syncx {
    use crate::v5::{
        commons::error::MQTTError,
        traits::{
            bufferio::BufferIO,
            syncx::{read::Read, write::Write},
        },
    };

    use super::SubscriptionOptions;

    impl BufferIO for SubscriptionOptions {
        fn length(&self) -> usize {
            1
        }

        fn write(&self, buf: &mut bytes::BytesMut) -> Result<(), MQTTError> {
            u8::from(*self).write(buf);
            Ok(())
        }

        fn read(buf: &mut bytes::Bytes) -> Result<Self, MQTTError> {
            let byte = u8::read(buf)?;

            Self::try_from(byte)
        }
    }
}

mod asyncx {
    use crate::v5::traits::{
        asyncx::{read::Read, write::Write},
        streamio::StreamIO,
    };

    use super::SubscriptionOptions;

    impl StreamIO for SubscriptionOptions {
        fn length(&self) -> usize {
            1
        }

        async fn write<W>(&self, stream: &mut W) -> Result<(), crate::v5::commons::error::MQTTError>
        where
            W: futures::AsyncWriteExt + Unpin,
        {
            u8::from(*self).write(stream).await
        }

        async fn read<R>(stream: &mut R) -> Result<Self, crate::v5::commons::error::MQTTError>
        where
            R: futures::AsyncReadExt + Unpin,
            Self: Default,
        {
            let byte = u8::read(stream).await?;
            Self::try_from(byte)
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RetainHandling {
    /// Send the retained messages at the time of the subscribe
    #[default]
    Zero = 0,
    /// Send retained messages at subscribe only if subscription does not currently exist
    One = 1,
    /// Do not send retained messages at the time of the subscription
    Two = 2,
}

impl TryFrom<u8> for RetainHandling {
    type Error = MQTTError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Zero),
            1 => Ok(Self::One),
            2 => Ok(Self::Two),
            _ => Err(MQTTError::MalformedPacket),
        }
    }
}
