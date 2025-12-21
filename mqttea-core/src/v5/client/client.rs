use std::sync::Arc;

use async_channel::Sender;

use crate::v5::{commons::packet::Packet, traits::pkid_mgr::PacketIdAlloc};

#[derive(Debug)]
pub struct MqttClient<T> {
    /// sends packets to the channel
    tx: Sender<Packet>,
    pkid_alloc: Arc<T>,
    max_size: usize,
}

impl<T> MqttClient<T>
where
    T: PacketIdAlloc,
{
    pub(crate) fn new(tx: Sender<Packet>, pkid_alloc: Arc<T>, max_size: usize) -> Self {
        Self {
            tx,
            pkid_alloc,
            max_size,
        }
    }
}

mod asyncx {
    use super::MqttClient;

    use bytes::Bytes;

    use crate::v5::{
        commons::{error::MQTTError, packet::Packet, qos::QoS},
        packet::{
            disconnect::Disconnect,
            publish::{Publish, PublishProperties},
            subscribe::{Subscribe, SubscribeProperties, SubscriptionOptions},
            unsubscribe::{UnSubscribe, UnSubscribeProperties},
        },
        traits::{pkid_mgr::PacketIdAlloc, streamio::StreamIO, utils::Utils},
    };

    impl<T> MqttClient<T>
    where
        T: PacketIdAlloc,
    {
        pub async fn publish<U, V>(
            &self,
            topic: U,
            qos: QoS,
            retain: bool,
            payload: V,
            properties: Option<PublishProperties>,
        ) -> Result<(), MQTTError>
        where
            U: Into<String>,
            V: Into<Bytes>,
        {
            let pkid = match qos {
                QoS::Zero => None,
                _ => Some(self.pkid_alloc.allocate()?),
            };

            let properties = properties.unwrap_or(Default::default());

            let packet = Publish {
                dup: false,
                retain,
                qos,
                topic: topic.into(),
                pkid,
                payload: payload.into(),
                properties,
            };

            packet.is_valid(self.max_size)?;
            packet.validate_topic(&packet.topic)?;

            self.tx.send(Packet::Publish(packet)).await?;

            Ok(())
        }

        pub async fn subscribe(
            &self,
            payload: Vec<(String, SubscriptionOptions)>,
            properties: Option<SubscribeProperties>,
        ) -> Result<(), MQTTError> {
            let pkid = self.pkid_alloc.allocate()?;
            let properties = properties.unwrap_or(Default::default());

            let packet = Subscribe {
                pkid,
                payload,
                properties,
            };

            packet.is_valid(self.max_size)?;

            self.tx.send(Packet::Subscribe(packet)).await?;

            Ok(())
        }

        pub async fn unsubscribe<P>(
            &self,
            payload: P,
            properties: Option<UnSubscribeProperties>,
        ) -> Result<(), MQTTError>
        where
            P: Into<Vec<String>>,
        {
            let pkid = self.pkid_alloc.allocate()?;
            let properties = properties.unwrap_or(Default::default());

            let packet = UnSubscribe {
                pkid,
                properties,
                payload: payload.into(),
            };

            packet.is_valid(self.max_size)?;
            self.tx.send(Packet::UnSubscribe(packet)).await?;

            Ok(())
        }

        pub async fn disconnect(&self) -> Result<(), MQTTError> {
            let packet = Disconnect::default();

            self.tx.send(Packet::Disconnect(packet)).await?;
            Ok(())
        }
    }
}

mod syncx {}
