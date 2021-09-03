//! Big integer type definition.

#![doc(hidden)]

use crate::math::*;

/// Storage for a big integer type.
#[derive(Clone, PartialEq, Eq)]
pub struct Bigint {
    /// Internal storage for the Bigint, in little-endian order.
    pub data: LimbVecType,
}

impl Default for Bigint {
    fn default() -> Self {
        // We want to repeated reallocations at smaller volumes.
        let mut bigint = Bigint {
            data: LimbVecType::default(),
        };
        reserve(&mut bigint.data, 20);
        bigint
    }
}

impl Math for Bigint {
    #[inline(always)]
    fn data(&self) -> &LimbVecType {
        &self.data
    }

    #[inline(always)]
    fn data_mut(&mut self) -> &mut LimbVecType {
        &mut self.data
    }
}
