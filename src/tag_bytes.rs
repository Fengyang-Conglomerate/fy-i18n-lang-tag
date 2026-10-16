//! Canonical writing and trusted UTF-8 conversions for the tag layer.

use crate::storage::{CapacityError, Storage};

/// Appends validated ASCII content in normalized lowercase to private storage.
pub(crate) fn push_lowercase<S: Storage>(storage: &mut S, text: &str) -> Result<(), CapacityError> {
    let required = storage
        .len()
        .checked_add(text.len())
        .ok_or(CapacityError::new(usize::MAX, storage.max_capacity()))?;
    if required > storage.max_capacity() {
        return Err(CapacityError::new(required, storage.max_capacity()));
    }
    for byte in text.bytes() {
        storage.push_byte(byte.to_ascii_lowercase())?;
    }
    Ok(())
}

/// Converts validated, immutable in-tag bytes to a string slice.
pub(crate) fn as_str(bytes: &[u8]) -> &str {
    #[cfg(feature = "unsafe")]
    {
        // SAFETY: All call sites pass either Tag's private core or private storage.
        // Constructors first validate ASCII tag syntax, clear the backend, and write only
        // validated components; derivation operations use validated components exclusively.
        // Storage is sealed to Fixed/Grow without shared mutable state, and Tag exposes neither
        // references nor ownership of the backend. Hence valid bytes are guaranteed ASCII.
        unsafe { core::str::from_utf8_unchecked(bytes) }
    }
    #[cfg(not(feature = "unsafe"))]
    {
        core::str::from_utf8(bytes)
            .expect("tag invariant violated: internal bytes are not valid UTF-8")
    }
}
