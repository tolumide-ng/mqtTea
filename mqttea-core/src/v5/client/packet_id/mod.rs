use std::sync::atomic::{AtomicU16, Ordering};

mod shard;
use shard::PacketIdShard;

use crate::v5::{
    commons::error::MQTTError,
    traits::pkid_mgr::{PacketIdAlloc, PacketIdRelease},
};

#[derive(Debug)]
pub struct PacketIdManager {
    shards: Vec<PacketIdShard>,
    allocated: AtomicU16,
    max_packets: u16,
}

impl PacketIdManager {
    const BITS: usize = usize::BITS as usize;

    pub(crate) fn new(max_packets: u16) -> Self {
        let num_shards = (max_packets as usize + Self::BITS - 1) / Self::BITS;
        let shards = (0..num_shards)
            .map(|_| PacketIdShard::default())
            .collect::<Vec<_>>();
        Self {
            shards,
            allocated: AtomicU16::new(0),
            max_packets,
        }
    }
}

impl PacketIdAlloc for PacketIdManager {
    fn allocate(&self) -> Result<u16, MQTTError> {
        let allocated = self.allocated.fetch_add(1, Ordering::AcqRel);
        if allocated >= self.max_packets {
            // rollback
            self.allocated.fetch_sub(1, Ordering::Release);
            return Err(MQTTError::PacketIdGenerationError);
        }

        for (shard_index, shard) in self.shards.iter().enumerate() {
            if let Some(id) = shard.allocate() {
                // packet must always be non-zero
                let packet_id = (shard_index * (Self::BITS) + id as usize) + 1;
                return Ok(packet_id as u16);
            }
        }

        // It should be almost impossible for this to occur, but its better taken care of than not!
        self.allocated.fetch_sub(1, Ordering::Release);
        return Err(MQTTError::PacketIdGenerationError);
    }
}

impl PacketIdRelease for PacketIdManager {
    /// Returns whether the packetId is in use or free (Not yet tested)
    fn is_occupied(&self, id: u16) -> bool {
        let id = (id - 1) as usize;
        let shard_index = id / Self::BITS;
        let actual_index_in_shard = (id % Self::BITS) as u8;
        let result = self
            .shards
            .get(shard_index)
            .and_then(|shard| Some(shard.release(actual_index_in_shard)));
        return result.is_some();
    }

    fn release(&self, id: u16) {
        let id = (id - 1) as usize;
        let shard_index = id / Self::BITS;
        let actual_index_in_shard = (id % Self::BITS) as u8;
        let result = self
            .shards
            .get(shard_index)
            .and_then(|shard| Some(shard.release(actual_index_in_shard)));
        if result.is_some_and(|r| r) {
            let _ = self
                .allocated
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_sub(1));
        }
    }
}

// use futures::execut

#[cfg(test)]
mod test {
    use std::sync::atomic::Ordering;

    use super::*;

    #[test]
    fn creates_a_packetid_manager() {
        let mgr = PacketIdManager::new(2);
        assert_eq!(mgr.shards.len(), 1);
        assert_eq!(mgr.allocated.load(Ordering::Relaxed), 0);

        let mgr = PacketIdManager::new(123);
        assert_eq!(mgr.shards.len(), 2);
        assert_eq!(mgr.max_packets, 123);
        assert_eq!(mgr.allocated.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn can_allocate_and_release_packet_id() {
        let mgr = PacketIdManager::new(2);
        assert_eq!(mgr.allocated.load(Ordering::Relaxed), 0);

        let packet_id_1 = mgr.allocate();
        assert_eq!(packet_id_1, Ok(1));
        assert_eq!(mgr.allocated.load(Ordering::Relaxed), 1);

        let packet_id_2 = mgr.allocate();
        assert_eq!(packet_id_2, Ok(2));
        assert_eq!(mgr.allocated.load(Ordering::Relaxed), 2);

        assert_eq!(mgr.allocate(), Err(MQTTError::PacketIdGenerationError));
        assert_eq!(mgr.allocated.load(Ordering::Relaxed), 2);

        mgr.release(packet_id_1.unwrap());
        assert_eq!(mgr.allocated.load(Ordering::Relaxed), 1);

        mgr.release(packet_id_2.unwrap());
        assert_eq!(mgr.allocated.load(Ordering::Relaxed), 0);
    }

    #[test]
    // release
    fn must_not_panick_if_packet_id_is_out_of_bounds_or_does_not_exist() {
        let mgr = PacketIdManager::new(36);
        assert_eq!(mgr.allocated.load(Ordering::Relaxed), 0);

        mgr.release(37);
        assert_eq!(mgr.allocated.load(Ordering::Relaxed), 0);

        mgr.release(67);
        assert_eq!(mgr.allocated.load(Ordering::Relaxed), 0);
    }
}
