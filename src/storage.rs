//! Tag storage abstractions and backend implementations.
//!
//! Provides inline fixed-capacity stack storage [`Fixed`] and dynamically allocated heap storage `Grow` (requires `alloc` feature).

pub mod fixed;
pub mod traits;

#[cfg(feature = "alloc")]
pub mod grow;

pub use fixed::Fixed;
pub use traits::{CapacityError, Storage};

#[cfg(feature = "alloc")]
pub use grow::Grow;

// Sealed backend pattern: ensures Tag private storage cannot be altered by external implementations after construction.
mod sealed {
    pub trait Sealed {}
    impl<const MAX: usize> Sealed for super::Fixed<MAX> {}
    #[cfg(feature = "alloc")]
    impl<const MAX: usize> Sealed for super::Grow<MAX> {}
}
