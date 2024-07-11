use anyhow::{bail, Result};

/// A very simple hash set implementation that uses
/// linear probing to handle collisions. This is intended
/// to be extremely lightweight to improve performance.
#[derive(Debug, Clone)]
pub struct Hash {
    pub(crate) key: Vec<usize>,
    pub(crate) value: Vec<f32>,
    capacity: usize,
}

impl Hash {
    pub fn new(capacity: usize) -> Hash {
        Hash {
            key: vec![0; capacity],
            value: vec![0.; capacity],
            capacity,
        }
    }

    ///// Reset all the nonzero values to 0
    //pub fn reset_values(&mut self) {
    //    for i in 0..self.value.len() {
    //        if self.value[i] > 0. {
    //            self.value[i] = 0.;
    //        }
    //    }
    //}

    /// Add the given value to the set
    /// NOTE: Assumes target is > 0!
    pub fn add(&mut self, target: usize) -> Result<usize> {
        // We may now cast hv to a usize because we're sure
        // that it is < self.size and will therefore fit.
        let start = target % self.capacity as usize;
        let mut probed_index = start;

        if self.key[probed_index] == target {
            return Ok(probed_index);
        }

        // Find the next empty slot (this is the linear probing bit).
        loop {
            if self.key[probed_index] == 0 {
                self.key[probed_index] = target;
                break;
            }

            probed_index += 1;

            // We've reached the end of the hash, so go back to the beginning
            if probed_index >= self.capacity as usize {
                probed_index = 0;
            }

            // We've wrapped around to the start w/o finding a slot
            if probed_index == start {
                bail!("hash full");
            }
        }

        Ok(probed_index)
    }
}

#[cfg(test)]
mod hash_tests {
    use crate::hash::Hash;

    #[test]
    fn hash_create() {
        let hash = Hash::new(10);
        assert_eq!(hash.capacity, 10);
        assert_eq!(hash.key.len(), 10);
        assert_eq!(hash.value.len(), 10);

        for i in 0..10 {
            assert_eq!(hash.key[i], 0);
            assert_eq!(hash.value[i], 0.);
        }
    }

    #[test]
    fn hash_add() {
        let mut hash = Hash::new(10);
        let res = hash.add(10);
        assert!(res.is_ok());

        assert_eq!(hash.key[0], 10);
        assert_eq!(res.unwrap(), 0);

        let res = hash.add(11);
        assert!(res.is_ok());

        assert_eq!(hash.key[1], 11);
        assert_eq!(res.unwrap(), 1);

        // 1 will collide with 10 and should
        // be pushed over 2 places after 11
        let res = hash.add(1);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 2);
        assert_eq!(hash.key[2], 1);

        //// 19 should be placed at the end
        let res = hash.add(19);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 9);
        assert_eq!(hash.key[9], 19);

        //// 9 will conflict with 19 and will wrap around
        let res = hash.add(9);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 3);
        assert_eq!(hash.key[3], 9);

        //// 20 will conflict with 10 and will wrap around
        let res = hash.add(20);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 4);
        assert_eq!(hash.key[4], 20);
    }
}
