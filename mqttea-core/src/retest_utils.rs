use std::sync::{Arc, Mutex};

use crate::{constants::PID, v5::client::packet_id::PacketIdManager};

pub(crate) fn initialize_pid() {
    PID.get_or_init(|| Arc::new(Mutex::new(PacketIdManager::new(200))));
}