//! Fixed stack capacity storage backend (`Fixed`).

use super::traits::{CapacityError, Storage};

/// A byte storage backend backed by a fixed-capacity inline stack array.
///
/// Default capacity is 32 bytes (`MAX = 32`), requiring no global allocator (`no_std` bare-metal compatible).
#[derive(Debug, Clone, Copy)]
pub struct Fixed<const MAX: usize = 32> {
    buf: [u8; MAX],
    len: usize,
}

impl<const MAX: usize> PartialEq for Fixed<MAX> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.buf[..self.len] == other.buf[..other.len]
    }
}

impl<const MAX: usize> Eq for Fixed<MAX> {}

impl<const MAX: usize> PartialOrd for Fixed<MAX> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<const MAX: usize> Ord for Fixed<MAX> {
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.buf[..self.len].cmp(&other.buf[..other.len])
    }
}

impl<const MAX: usize> core::hash::Hash for Fixed<MAX> {
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.buf[..self.len].hash(state);
    }
}

impl<const MAX: usize> Fixed<MAX> {
    /// Constructs an empty fixed storage backend.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            buf: [0u8; MAX],
            len: 0,
        }
    }

    /// Returns the maximum capacity in bytes.
    #[must_use]
    #[inline]
    pub const fn max_capacity(&self) -> usize {
        MAX
    }
}

impl<const MAX: usize> Default for Fixed<MAX> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<const MAX: usize> Storage for Fixed<MAX> {
    #[inline]
    fn as_bytes(&self) -> &[u8] {
        &self.buf[..self.len]
    }

    #[inline]
    fn max_capacity(&self) -> usize {
        MAX
    }

    #[inline]
    fn clear(&mut self) {
        self.len = 0;
    }

    fn push_bytes(&mut self, bytes: &[u8]) -> Result<(), CapacityError> {
        let new_len = self
            .len()
            .checked_add(bytes.len())
            .ok_or(CapacityError::new(usize::MAX, MAX))?;
        if new_len > MAX {
            return Err(CapacityError::new(new_len, MAX));
        }
        self.buf[self.len..new_len].copy_from_slice(bytes);
        self.len = new_len;
        Ok(())
    }
}
