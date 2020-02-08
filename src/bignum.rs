//! Big integer type definition.

use super::math::*;

/// Storage for a big integer type.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Bigint {
    /// Internal storage for the Bigint, in little-endian order.
    pub(crate) data: LimbVecType,
}

impl Default for Bigint {
    fn default() -> Self {
        Bigint { data: LimbVecType::default() }
    }
}

impl Math for Bigint {
    #[inline(always)]
    fn data<'a>(&'a self) -> &'a LimbVecType {
        &self.data
    }

    #[inline(always)]
    fn data_mut<'a>(&'a mut self) -> &'a mut LimbVecType {
        &mut self.data
    }
}
