use std::{num::NonZero, u32};

use bytes::Bytes;

use super::packet::connect::will::Will;

pub(crate) mod client;
pub mod handler;
pub mod network;
pub(crate) mod packet_id;
pub(crate) mod state;

#[derive(Debug)]
pub struct ConnectOptions {
    /// Whether the user want's to handle all acks manually, or they want us to do this for them
    pub manual_ack: bool,
    pub clean_start: bool,
    /// 3.1.2.11.2
    pub session_expiry_interval: Option<u32>, // default value is 0
    /// 3.1.2.11.4 Maximum number of bytes in an MQTT Control Packet
    pub server_max_size: NonZero<u32>, // set by connack
    pub client_max_size: NonZero<u32>, // set by connect

    /// 3.1.2.11.3 The Client uses this value to limit the number of QoS 1 and QoS 2 publications that it is willing to process concurrently
    pub client_receive_max: NonZero<u16>, // this is us (the client)
    /// 3.2.2.3.3 The Server uses this value to limit the number of QoS 1 and QoS 2 publications that it is willing to process concurrently for the Client.
    /// this determines how much pkids we can make max (the server already told us how much it can handle, so that's the max we can generate)
    /// This is usually received on the CONNACK
    pub server_receive_max: NonZero<u16>,

    /// 3.1.2.11.5 Highest value a client will accept as a topic alias sent by the server
    pub inbound_topic_alias_max: u16, // this is sent in the connect packet
    /// 3.2.2.3.8 Topic Alias Maximum: Indicates the highest value that the Server will accept as a Topic Alias sent by the Client
    pub outbound_topic_alias_max: u16, // this is obtained from the connack packet

    // we use the server's keep_alive from CONNACK else we use the one in CONNECT. Must always be in seconds
    pub keep_alive: u16,
    pub will: Option<Will>,
    pub client_id: String,
    pub username: Option<String>,
    pub password: Option<String>,

    // host: Option<String>,
    // port: Option<u16>,
    pub request_response_information: Option<u8>,
    pub request_problem_information: Option<u8>,
    pub user_property: Vec<(String, String)>,
    pub authentication_method: Option<String>,
    pub authentication_data: Option<Bytes>,
}

impl Default for ConnectOptions {
    fn default() -> Self {
        Self {
            inbound_topic_alias_max: 0,
            outbound_topic_alias_max: 0,
            manual_ack: false,
            clean_start: true,
            session_expiry_interval: Some(0),
            client_receive_max: NonZero::<u16>::MAX, // connect
            server_receive_max: NonZero::<u16>::MAX, // connack

            keep_alive: 69,
            will: None,
            client_id: String::from("UniqueClientId"),
            username: None,
            password: None,
            // the client uses the size to inform the Server that it will not process packets exceeding this limit
            // WE must return a DISCONNECT with reasoncode 0x95 if we receive any packet larger than this size
            server_max_size: NonZero::<u32>::MAX,
            client_max_size: NonZero::<u32>::MAX,

            request_response_information: None,
            request_problem_information: None,
            user_property: Vec::with_capacity(0),
            authentication_method: None,
            authentication_data: None,
        }
    }
}
