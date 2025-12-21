use std::sync::{Arc, Mutex, OnceLock};

use crate::v5::client::packet_id::PacketIdManager;

pub(crate) const MAX: usize = 65_535;
pub(crate) const PROTOCOL_NAME: &'static str = "MQTT";

/// this need to be updated in such a way that we can easily create newer instances for newer clients
/// this is not a good idea and should be moved instead to the network where it can be properly managed
pub(crate) static PID: OnceLock<Arc<Mutex<PacketIdManager>>> = OnceLock::new();
