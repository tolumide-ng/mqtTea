// The #[cfg(target_has_atomic)]

use std::sync::atomic::{AtomicUsize, Ordering};

/// Depending on the target architecture, this can either be 64 bits or 32 bits
#[derive(Debug, Default)]
pub(super) struct PacketIdShard(pub(super) AtomicUsize); // Each bit manages 64 OR 32 packet id's (Usize::BITS = 64 OR 32)

impl PacketIdShard {
    // Allocate an available packet ID
    pub(super) fn allocate(&self) -> Option<u16> {
        let mut bitmap = self.0.load(Ordering::Relaxed);
        loop {
            let free_index = (!bitmap).trailing_zeros();
            if free_index >= usize::BITS { return None; } // break here

            let new_bitmap = bitmap | (1 << free_index);
            // Try to reserve the packetId
            let result = self.0.compare_exchange(bitmap, new_bitmap, Ordering::Acquire, Ordering::Relaxed);

            match result {
                Ok(_) => { return Some(free_index as u16) }
                Err(current_bitmap) => { bitmap = current_bitmap }
            }
        }
    }

    /// Release a packet ID
    /// Returns `true` if the release was successful, otherwise, it returns false
    pub(super) fn release(&self, id: u8) -> bool {
        if id >= usize::BITS as u8 { return false }
        let result= self.0.fetch_and(!(1 << id), Ordering::Release);
        let new = self.0.load(Ordering::Relaxed);
        new != result
    }

    /// Returns the number of already allocated packet ids in this shard
    pub(super) fn count(&self) -> u8 {
        self.0.load(Ordering::Relaxed).count_ones() as u8
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod allocate_packet_id {
        use super::*;
        
        #[test]
        fn should_return_the_nearest_id() {
            let shard = PacketIdShard(0x1DF.into());
            let next_index = shard.allocate();
            let expected = 0b000111011111usize;
            assert_eq!(next_index, Some((!expected).trailing_zeros() as u16));
        }
        
        
        #[test]
        fn should_return_none_if_there_is_no_more_space() {
            // 18_446_744_073_709_551_615 (usize::MAX on 64-bit platform)
            if cfg!(target_pointer_width = "64") {
                let value = 0xFFFFFFFFFFFFFFFF;
                let shard = PacketIdShard(value.into());
                let next_index = shard.allocate();
                assert_eq!(next_index, None);
            } else if cfg!(target_pointer_width = "32") {
                let value = 0xFFFFFFFF;
                let shard = PacketIdShard(value.into());
                let next_index = shard.allocate();
                assert_eq!(next_index, None);
            } else {
                assert!(false, "Unknown architecture")
            }
        }
    }


    #[cfg(test)]
    mod relax_packet_id {
        use super::*;


        #[test]
        fn should_release_an_allocated_packet_id() {
            let shard = PacketIdShard(0x1DF.into());
            let expected = 0b000111011111usize;
            assert_eq!(shard.0.load(Ordering::Relaxed), expected);
            
            shard.release(3);
            let new_value = shard.0.load(Ordering::Relaxed);
            assert_eq!(new_value, 0b000111010111);
    
            shard.release(0);
            let new_value = shard.0.load(Ordering::Relaxed);
            assert_eq!(new_value, 0b000111010110);
    
            shard.release(63);
            let new_value = shard.0.load(Ordering::Relaxed);
            assert_eq!(new_value, 0b000111010110);
        }
    
        #[test]
        fn should_do_nothing_if_the_release_index_is_oob() {
            //oob -> Out ot bounds
    
            let shard = PacketIdShard(0x1DF.into());
            let expected = 0b000111011111usize;
            assert_eq!(shard.0.load(Ordering::Relaxed), expected);
    
            shard.release(64);
            let new_value = shard.0.load(Ordering::Relaxed);
            assert_eq!(new_value, expected);
        }
    
    
        #[test]
        fn should_do_nothing_if_the_index_was_not_previously_allocated() {
            let shard = PacketIdShard(0x1DF.into());
            let expected = 0b000111011111usize;
            assert_eq!(shard.0.load(Ordering::Relaxed), expected);
            
            shard.release(5);
            let new_value = shard.0.load(Ordering::Relaxed);
            assert_eq!(new_value, expected);
        }
    }
}