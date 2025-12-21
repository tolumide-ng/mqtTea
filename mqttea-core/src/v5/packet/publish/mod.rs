mod properties;
pub use properties::PublishProperties;

use bytes::Bytes;

use crate::v5::{
    commons::{fixed_header::FixedHeader, packet_type::PacketType, qos::QoS},
    traits::read_data::ReadData,
};

#[derive(Debug, Default, PartialEq, Clone, Eq)]
pub struct Publish {
    pub dup: bool,
    pub retain: bool,
    pub qos: QoS,
    pub topic: String,
    pub pkid: Option<u16>,
    pub properties: PublishProperties,
    pub payload: Bytes,
}

#[cfg(feature = "asyncx")]
pub(crate) use asyncx::*;
#[cfg(feature = "syncx")]
pub(crate) use syncx::*;

impl ReadData for Publish {}

mod syncx {
    use bytes::Bytes;

    use crate::v5::{
        commons::{error::MQTTError, qos::QoS},
        traits::{
            bufferio::BufferIO,
            syncx::{read::Read, write::Write},
        },
    };

    use super::{FixedHeader, PacketType, Publish, PublishProperties};

    impl BufferIO for Publish {
        /// variable header, length of the payload, encoded as Variable Byte Integer
        fn length(&self) -> usize {
            let mut len = if self.qos != QoS::Zero {
                self.topic.len() + 2
            } else {
                0
            };
            len +=
                self.properties.length() + self.properties.variable_length() + self.payload.len();
            len
        }

        fn write(&self, buf: &mut bytes::BytesMut) -> Result<(), MQTTError> {
            FixedHeader::new(
                PacketType::Publish,
                (self.dup as u8) << 3 | (self.qos as u8) << 1 | (self.retain as u8),
                self.length(),
            )
            .write(buf)?;

            self.topic.write(buf);
            if self.qos != QoS::Zero {
                // assignment of a packet id should be done from the client level after user provides us with the publish data
                self.pkid
                    .ok_or_else(|| MQTTError::PacketIdRequired)?
                    .write(buf);
            }

            self.properties.write(buf)?;
            self.payload.write(buf);
            Ok(())
        }

        /// Publish does not implement `read` only read_with_flag
        fn read_with_fixedheader(buf: &mut Bytes, header: FixedHeader) -> Result<Self, MQTTError> {
            let mut packet = Self::default();
            let flag = header.flags.unwrap_or(0);

            packet.topic = String::read(buf)?;
            packet.dup = (flag & 0b1000) != 0;
            let qos = (flag & 0b0110) >> 1;
            packet.qos = QoS::try_from(qos).map_err(|_| MQTTError::UnsupportedQoS(qos))?;
            packet.retain = (flag & 0b1) != 0;

            if packet.qos != QoS::Zero {
                packet.pkid = Some(u16::read(buf).map_err(|_| MQTTError::PacketIdRequired)?);
            }

            packet.properties = PublishProperties::read(buf)?;
            packet.payload = Bytes::read(buf)?;
            Ok(packet)
        }
    }
}

mod asyncx {
    use bytes::Bytes;

    use crate::v5::{
        commons::{error::MQTTError, qos::QoS},
        traits::{
            asyncx::{read::Read, write::Write},
            streamio::StreamIO,
        },
    };

    use super::{FixedHeader, PacketType, Publish, PublishProperties};
    impl StreamIO for Publish {
        /// variable header, length of the payload, encoded as Variable Byte Integer
        fn length(&self) -> usize {
            let mut len = if self.qos != QoS::Zero {
                self.topic.len() + 2
            } else {
                0
            };
            len +=
                self.properties.length() + self.properties.variable_length() + self.payload.len();
            len
        }

        async fn write<W>(&self, stream: &mut W) -> Result<(), crate::v5::commons::error::MQTTError>
        where
            W: futures::AsyncWriteExt + Unpin,
        {
            FixedHeader::new(
                PacketType::Publish,
                (self.dup as u8) << 3 | (self.qos as u8) << 1 | (self.retain as u8),
                self.length(),
            )
            .write(stream)
            .await?;

            self.topic.write(stream).await?;
            if self.qos != QoS::Zero {
                // assignment of a packet id should be done from the client level after user provides us with the publish data
                self.pkid
                    .ok_or_else(|| MQTTError::PacketIdRequired)?
                    .write(stream)
                    .await?;
            }

            self.properties.write(stream).await?;
            self.payload.to_vec().write(stream).await?;
            Ok(())
        }

        async fn read_with_fixedheader<R>(
            stream: &mut R,
            header: &FixedHeader,
        ) -> Result<Self, MQTTError>
        where
            R: futures::AsyncReadExt + Unpin,
            Self: Default,
        {
            let mut packet = Self::default();
            let flag = header.flags.unwrap_or(0);

            packet.topic = String::read(stream).await?;
            packet.dup = (flag & 0b1000) != 0;
            let qos = (flag & 0b0110) >> 1;
            packet.qos = QoS::try_from(qos).map_err(|_| MQTTError::UnsupportedQoS(qos))?;
            packet.retain = (flag & 0b1) != 0;

            if packet.qos != QoS::Zero {
                packet.pkid = Some(
                    u16::read(stream)
                        .await
                        .map_err(|_| MQTTError::PacketIdRequired)?,
                );
            }

            packet.properties = PublishProperties::read(stream).await?;
            packet.payload = Bytes::from_iter(Vec::read(stream).await?);

            Ok(packet)
        }
    }
}

#[cfg(test)]
mod syncx_tests {
    use super::*;
    use crate::retest_utils::initialize_pid;
    use crate::v5::traits::bufferio::BufferIO;
    use bytes::BytesMut;

    #[test]
    fn read_write_publish() {
        initialize_pid();
        let packet = Publish {
            dup: true,
            retain: true,
            qos: QoS::One,
            topic: String::from("packagin_plant/#"),
            pkid: Some(8930),
            payload: b"veryLarge payload".to_vec().into(),
            properties: PublishProperties {
                payload_format_indicator: Some(13),
                topic_alias: Some(02),
                ..Default::default()
            },
        };

        let mut buf = BytesMut::new();
        packet.write(&mut buf).unwrap();

        let expected =
            b";)\0\x10packagin_plant/#\"\xe2\x05\x01\r#\0\x02\0\x11veryLarge payload".to_vec();
        assert_eq!(buf.to_vec(), expected);

        let mut expected = Bytes::from_iter(
            b";)\0\x10packagin_plant/#\"\xe2\x05\x01\r#\0\x02\0\x11veryLarge payload".to_vec()[2..]
                .to_vec(),
        );
        let created_packed = Publish::read_with_fixedheader(
            &mut expected,
            FixedHeader::new(PacketType::Publish, 0b1011, 9),
        )
        .unwrap();
        assert_eq!(created_packed, packet);
    }
}
