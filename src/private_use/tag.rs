//! Owned pure private-use language tag structure (`PrivateUseTag`).

use core::fmt;
use core::ops::Deref;
use core::str::FromStr;

use super::error::TagError;
use super::view::PrivateUseTagView;
use crate::storage::{CapacityError, Fixed, Storage};

/// Generic owned structure for a pure private-use language tag.
///
/// Manages a normalized lowercase subtag sequence via a configurable underlying storage backend `S`.
/// Defaults to [`Fixed<32>`] using a fixed stack buffer, working out-of-the-box in `no_std` environments with zero heap allocations.
///
/// # Examples
///
/// ```
/// use fy_i18n_lang_tag::PrivateUseTag;
///
/// // Uses Fixed<32> by default
/// let tag: PrivateUseTag = PrivateUseTag::parse("x-a-b-c").unwrap();
/// assert_eq!(tag.as_str(), "a-b-c");
/// assert_eq!(tag.to_string(), "x-a-b-c");
/// ```
///
/// Tags do not expose internal storage:
///
/// ```compile_fail
/// use fy_i18n_lang_tag::PrivateUseTag;
/// let tag: PrivateUseTag = PrivateUseTag::parse("x-a").unwrap();
/// let _ = tag.storage();
/// ```
#[derive(Clone)]
pub struct PrivateUseTag<S: Storage = Fixed<32>> {
    storage: S,
}

impl<S1: Storage, S2: Storage> PartialEq<PrivateUseTag<S2>> for PrivateUseTag<S1> {
    #[inline]
    fn eq(&self, other: &PrivateUseTag<S2>) -> bool {
        self.as_str().eq_ignore_ascii_case(other.as_str())
    }
}

impl<S: Storage> Eq for PrivateUseTag<S> {}

impl<S1: Storage, S2: Storage> PartialOrd<PrivateUseTag<S2>> for PrivateUseTag<S1> {
    #[inline]
    fn partial_cmp(&self, other: &PrivateUseTag<S2>) -> Option<core::cmp::Ordering> {
        Some(self.as_view().cmp(&other.as_view()))
    }
}

impl<S: Storage> Ord for PrivateUseTag<S> {
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.as_view().cmp(&other.as_view())
    }
}

impl<S: Storage> core::hash::Hash for PrivateUseTag<S> {
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.as_view().hash(state);
    }
}

impl<S: Storage> PrivateUseTag<S> {
    /// Validates and parses a string with `x-` prefix into an owned private-use tag.
    ///
    /// Internally normalizes subtags to lowercase per RFC 5646 recommendations into the storage backend.
    ///
    /// # Errors
    ///
    /// Returns [`TagError::Parse`] if syntax is invalid, or [`TagError::Capacity`] if exceeding capacity.
    pub fn parse(s: &str) -> Result<Self, TagError> {
        let view = PrivateUseTagView::parse(s)?;
        Self::from_view(view).map_err(TagError::from)
    }

    /// Parses using the specified storage backend instance.
    pub fn parse_with(s: &str, storage: S) -> Result<Self, TagError> {
        let view = PrivateUseTagView::parse(s)?;
        Self::from_view_with(view, storage).map_err(TagError::from)
    }

    /// Constructs an owned tag from a pure subtags sequence without `x-` prefix.
    ///
    /// # Errors
    ///
    /// Returns [`TagError::Parse`] if syntax is invalid, or [`TagError::Capacity`] if exceeding capacity.
    pub fn from_subtags(subtags: &str) -> Result<Self, TagError> {
        let view = PrivateUseTagView::from_subtags(subtags)?;
        Self::from_view(view).map_err(TagError::from)
    }

    /// Constructs an owned private-use tag from a validated borrowed view.
    ///
    /// # Errors
    ///
    /// Returns [`CapacityError`] if subtag length exceeds storage capacity.
    pub fn from_view(view: PrivateUseTagView<'_>) -> Result<Self, CapacityError> {
        Self::from_view_with(view, S::default())
    }

    /// Constructs using the specified storage backend instance from a validated borrowed view.
    pub fn from_view_with(
        view: PrivateUseTagView<'_>,
        mut storage: S,
    ) -> Result<Self, CapacityError> {
        let required = view.as_str().len();
        if required > storage.max_capacity() {
            return Err(CapacityError::new(required, storage.max_capacity()));
        }
        storage.clear();
        crate::tag_bytes::push_lowercase(&mut storage, view.as_str())?;
        Ok(Self { storage })
    }

    /// Borrows as a lightweight [`PrivateUseTagView`].
    #[must_use]
    #[inline]
    pub fn as_view(&self) -> PrivateUseTagView<'_> {
        PrivateUseTagView(crate::tag_bytes::as_str(self.storage.as_bytes()))
    }

    /// Returns the stored subtag sequence string slice without `x-` prefix (normalized to lowercase).
    #[must_use]
    #[inline]
    pub fn as_str(&self) -> &str {
        crate::tag_bytes::as_str(self.storage.as_bytes())
    }

    /// Iterates over individual private-use subtag slices.
    #[inline]
    pub fn subtags(&self) -> impl Iterator<Item = &str> {
        crate::tag_bytes::as_str(self.storage.as_bytes()).split('-')
    }

    /// Returns the maximum capacity (in bytes) of the storage backend.
    #[must_use]
    #[inline]
    pub fn max_capacity(&self) -> usize {
        self.storage.max_capacity()
    }
}

impl<S: Storage> AsRef<str> for PrivateUseTag<S> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<S: Storage> Deref for PrivateUseTag<S> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl<S: Storage> fmt::Display for PrivateUseTag<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "x-{}", self.as_str())
    }
}

impl<S: Storage> fmt::Debug for PrivateUseTag<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PrivateUseTag(\"x-{}\")", self.as_str())
    }
}

impl<S: Storage> TryFrom<&str> for PrivateUseTag<S> {
    type Error = TagError;

    #[inline]
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::parse(s)
    }
}

impl<S: Storage> FromStr for PrivateUseTag<S> {
    type Err = TagError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

// Supports mutual comparison between owned tags and borrowed views.
impl<S: Storage> PartialEq<PrivateUseTagView<'_>> for PrivateUseTag<S> {
    #[inline]
    fn eq(&self, other: &PrivateUseTagView<'_>) -> bool {
        self.as_view() == *other
    }
}

impl<S: Storage> PartialEq<PrivateUseTag<S>> for PrivateUseTagView<'_> {
    #[inline]
    fn eq(&self, other: &PrivateUseTag<S>) -> bool {
        *self == other.as_view()
    }
}
