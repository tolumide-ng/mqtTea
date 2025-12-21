use std::{sync::Arc, time::Instant};

use async_channel::Receiver;
use futures::{select, AsyncReadExt, AsyncWriteExt, FutureExt};

use crate::v5::{
    client::{client::MqttClient, handler::AsyncHandler, state::State, ConnectOptions},
    commons::{error::MQTTError, packet::Packet, packet_type::PacketType},
    packet::{
        connack::{reason_code::ConnAckReasonCode, ConnAck},
        connect::Connect,
        ping::PingReq,
    },
    traits::streamio::StreamIO,
};

use super::PacketIdManager;

pub enum NetworkStatus {
    IncomingDisconnect,
    OutgoingDisconnect,
    Timeout,
}

#[derive(Debug)]
pub struct Network<S> {
    stream: S,
    options: ConnectOptions,
    state: State<PacketIdManager>,
    rx: Receiver<Packet>,
}

impl<S> Network<S>
where
    S: AsyncReadExt + AsyncWriteExt + Unpin,
{
    pub async fn new(
        options: ConnectOptions,
        stream: S,
    ) -> Result<(Self, MqttClient<PacketIdManager>), MQTTError> {
        let max_size = options.server_max_size.get() as usize;
        let state = State::from(&options);

        let (tx, rx) = async_channel::bounded::<Packet>(100); // receive_max + send_max

        let mut network = Self {
            stream,
            options,
            // pkids,
            state,
            rx,
        };

        let connack = network.connect().await?;
        let server_receive_max = connack.properties.receive_maximum.unwrap_or(100);
        let pkids = Arc::new(PacketIdManager::new(server_receive_max));

        network.state.pkid_mgr = Some(pkids.clone());

        Ok((network, MqttClient::new(tx, pkids, max_size)))
    }

    async fn connect(&mut self) -> Result<ConnAck, MQTTError> {
        Packet::Connect(Connect::from(&self.options))
            .write(&mut self.stream)
            .await?;

        let packet = Packet::read(&mut self.stream).await?;

        let Packet::ConnAck(connack) = packet else {
            return Err(MQTTError::ConnectionError); // this needs to be return an Error that contains the packet received
        };

        if connack.reason == ConnAckReasonCode::Success {
            return Ok(connack);
        }

        Err(MQTTError::ConnectionRefused(connack.reason.into()))
    }

    pub async fn run<H>(&mut self, handler: &mut H) -> Result<NetworkStatus, MQTTError>
    where
        H: AsyncHandler,
    {
        let mut last_ping = if self.options.keep_alive > 0 {
            Some(Instant::now())
        } else {
            None
        };

        let keep_alive = self.options.keep_alive;
        let mut expecting_pingresp = false;
        let max_timeout = keep_alive as u64 * (3 / 2);

        // let result = self.state.handle_incoming_packet(&mut data);
        // result.unwrap().unwrap().write(&mut self.stream);

        loop {
            select! {
                // receiving incoming packets
                incoming = Packet::read(&mut self.stream).fuse() => {
                    let mut packet = incoming?;

                    match packet {
                        Packet::PingResp(_) => {
                            handler.handle(packet).await;
                            expecting_pingresp = false;
                            last_ping = Some(Instant::now());
                        }
                        Packet::Disconnect(_) => {
                            handler.handle(packet).await;
                            return Ok(NetworkStatus::IncomingDisconnect)
                        }
                        _ => {
                            let result = self.state.handle_incoming_packet(&mut packet)?;
                            last_ping = Some(Instant::now());
                            handler.handle(packet).await;

                            if let Some(response) = result {
                                response.write(&mut self.stream).await?;
                            }
                        }
                    }
                },
                outgoing = self.rx.recv().fuse() => {
                    let packet = outgoing?;

                    let disconnect = packet.packet_type() == PacketType::Disconnect;

                    packet.write(&mut self.stream).await?;
                    self.state.handle_outgoing_packet(packet)?;
                    last_ping = Some(Instant::now());

                    if disconnect {
                        return Ok(NetworkStatus::OutgoingDisconnect)
                    }
                },
                 default => {
                     let Some(last_time) = last_ping else { continue; };
                     let since = last_time.elapsed().as_secs();
                    if expecting_pingresp {
                        if since >= max_timeout {
                            return Ok(NetworkStatus::Timeout);
                        }
                        continue;
                    }

                    if last_ping.is_some_and(|t| t.elapsed().as_secs() >= keep_alive as u64) {
                        Packet::PingReq(PingReq::default()).write(&mut self.stream).await?;
                        last_ping = Some(Instant::now());
                        expecting_pingresp = true;
                    }
                },
            };
        }
    }
}
