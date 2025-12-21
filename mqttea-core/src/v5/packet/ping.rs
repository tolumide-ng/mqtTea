use crate::v5::{
    commons::{fixed_header::FixedHeader, packet_type::PacketType},
    traits::read_data::ReadData,
};

#[derive(Debug, Default, PartialEq, Eq)]
pub struct PingReq {}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct PingResp;

#[cfg(feature = "asyncx")]
pub(crate) use asyncx::*;
#[cfg(feature = "syncx")]
pub(crate) use syncx::*;

impl ReadData for PingReq {}
impl ReadData for PingResp {}

mod syncx {
    use crate::v5::{commons::error::MQTTError, traits::bufferio::BufferIO};

    use super::{FixedHeader, PacketType, PingReq, PingResp};

    impl BufferIO for PingReq {
        fn write(&self, buf: &mut bytes::BytesMut) -> Result<(), MQTTError> {
            FixedHeader::new(PacketType::PingReq, 0, 0).write(buf)?;
            Ok(())
        }

        fn read(_buf: &mut bytes::Bytes) -> Result<Self, MQTTError> {
            Ok(Self::default())
        }
    }

    impl BufferIO for PingResp {
        fn write(&self, buf: &mut bytes::BytesMut) -> Result<(), MQTTError> {
            FixedHeader::new(PacketType::PingResp, 0, 0).write(buf)?;
            Ok(())
        }

        fn read(_buf: &mut bytes::Bytes) -> Result<Self, MQTTError> {
            Ok(Self::default())
        }
    }
}

mod asyncx {
    use crate::v5::traits::streamio::StreamIO;

    use super::{FixedHeader, PacketType, PingReq, PingResp};

    impl StreamIO for PingReq {
        async fn write<W>(&self, stream: &mut W) -> Result<(), crate::v5::commons::error::MQTTError>
        where
            W: futures::AsyncWriteExt + Unpin,
        {
            FixedHeader::new(PacketType::PingReq, 0, 0)
                .write(stream)
                .await
        }

        async fn read<R>(stream: &mut R) -> Result<Self, crate::v5::commons::error::MQTTError>
        where
            R: futures::AsyncReadExt + Unpin,
            Self: Default,
        {
            Ok(Self::default())
        }
    }

    impl StreamIO for PingResp {
        async fn write<W>(&self, stream: &mut W) -> Result<(), crate::v5::commons::error::MQTTError>
        where
            W: futures::AsyncWriteExt + Unpin,
        {
            FixedHeader::new(PacketType::PingResp, 0, 0)
                .write(stream)
                .await
        }

        async fn read<R>(stream: &mut R) -> Result<Self, crate::v5::commons::error::MQTTError>
        where
            R: futures::AsyncReadExt + Unpin,
            Self: Default,
        {
            Ok(Self::default())
        }
    }
}
