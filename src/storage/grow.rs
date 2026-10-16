//! Dynamic heap storage backend (`Grow`).

extern crate alloc;

use super::traits::{CapacityError, Storage};
use alloc::vec::Vec;

/// A byte storage backend backed by a dynamically allocated heap buffer.
///
/// Manages bytes internally with [`Vec<u8>`], defaulting to maximum capacity `usize::MAX`.
/// Requires the `alloc` feature flag (gated by parent module).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Grow<const MAX: usize = { usize::MAX }> {
    buf: Vec<u8>,
}

impl<const MAX: usize> Grow<MAX> {
    /// Constructs an empty dynamic storage backend.
    #[must_use]
    pub const fn new() -> Self {
        Self { buf: Vec::new() }
    }

    /// Returns the maximum capacity upper bound in bytes.
    #[must_use]
    #[inline]
    pub const fn max_capacity(&self) -> usize {
        MAX
    }
}

impl<const MAX: usize> Default for Grow<MAX> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<const MAX: usize> Storage for Grow<MAX> {
    #[inline]
    fn as_bytes(&self) -> &[u8] {
        &self.buf
    }

    #[inline]
    fn max_capacity(&self) -> usize {
        MAX
    }

    #[inline]
    fn clear(&mut self) {
        self.buf.clear();
    }

    fn push_bytes(&mut self, bytes: &[u8]) -> Result<(), CapacityError> {
        let new_len = self
            .len()
            .checked_add(bytes.len())
            .ok_or(CapacityError::new(usize::MAX, MAX))?;
        if new_len > MAX {
            return Err(CapacityError::new(new_len, MAX));
        }
        self.buf.extend_from_slice(bytes);
        Ok(())
    }
}
