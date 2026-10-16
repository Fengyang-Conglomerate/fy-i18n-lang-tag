//! Storage traits and capacity error definitions.

use core::fmt;

/// An error indicating that the requested content exceeds the storage capacity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CapacityError {
    /// The required byte length to store the content.
    pub required: usize,
    /// The maximum capacity supported by the storage backend.
    pub max_capacity: usize,
}

impl CapacityError {
    /// Creates a new capacity error.
    #[must_use]
    pub const fn new(required: usize, max_capacity: usize) -> Self {
        Self {
            required,
            max_capacity,
        }
    }
}

impl fmt::Display for CapacityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "storage capacity exceeded: required length is {} bytes, but maximum capacity is {} bytes",
            self.required, self.max_capacity
        )
    }
}

impl core::error::Error for CapacityError {}

/// An abstraction over underlying byte storage backends.
///
/// Stores raw bytes without character set checks, case conversion, or string exposure.
/// Implemented exclusively by in-crate `Fixed` and `Grow` backends to ensure tag bytes
/// cannot be altered by external implementations or shared mutable state.
///
/// ```compile_fail
/// use fy_i18n_lang_tag::{Fixed, Storage};
/// let bytes = Fixed::<4>::new();
/// let _ = bytes.as_str();
/// ```
pub trait Storage: super::sealed::Sealed + Sized + Default {
    /// Borrows the currently stored valid byte slice.
    fn as_bytes(&self) -> &[u8];

    /// Returns the current valid byte length.
    #[inline]
    fn len(&self) -> usize {
        self.as_bytes().len()
    }

    /// Returns `true` if the storage is empty.
    #[inline]
    fn is_empty(&self) -> bool {
        self.as_bytes().is_empty()
    }

    /// Returns the maximum capacity supported by this storage backend.
    fn max_capacity(&self) -> usize;

    /// Replaces the entire content; returns an error and retains existing content if capacity is insufficient.
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), CapacityError> {
        if bytes.len() > self.max_capacity() {
            return Err(CapacityError::new(bytes.len(), self.max_capacity()));
        }
        self.clear();
        self.push_bytes(bytes)
    }

    /// Clears the storage content.
    fn clear(&mut self);

    /// Appends bytes as-is; returns an error and retains existing content if capacity is insufficient.
    fn push_bytes(&mut self, bytes: &[u8]) -> Result<(), CapacityError>;

    /// Appends a single byte as-is; returns an error and retains existing content if capacity is insufficient.
    fn push_byte(&mut self, byte: u8) -> Result<(), CapacityError> {
        self.push_bytes(&[byte])
    }
}
