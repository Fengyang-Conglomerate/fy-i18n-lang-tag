//! Pure private-use language tag borrowed view (`PrivateUseTagView`).

use super::error::ParseError;
use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::ops::Deref;

/// Checks whether a single byte is a private-use singleton (`'x'` or `'X'`).
#[must_use]
#[inline]
pub const fn is_singleton(b: u8) -> bool {
    b == b'x' || b == b'X'
}

/// Checks whether the given byte slice is a valid private-use subtag (1 to 8 ASCII alphanumerics).
#[must_use]
pub const fn is_subtag_bytes(bytes: &[u8]) -> bool {
    let len = bytes.len();
    if len < 1 || len > 8 {
        return false;
    }
    let mut i = 0usize;
    while i < len {
        if !bytes[i].is_ascii_alphanumeric() {
            return false;
        }
        i += 1;
    }
    true
}

/// Zero-allocation borrowed view of a pure private-use language tag.
///
/// Internally stores the subtag sequence stripped of the leading `x-` or `X-` prefix
/// (e.g., for input `"x-a-b-c"`, it stores `"a-b-c"`).
///
/// Conforms to BCP 47 / RFC 5646 rules:
/// - Case-insensitive comparison;
/// - Consists of one or more subtags separated by `-`;
/// - Each subtag is 1 to 8 ASCII letters or digits;
/// - [`Display`](fmt::Display) automatically formats to lowercase per standard recommendations.
#[derive(Debug, Clone, Copy)]
pub struct PrivateUseTagView<'a>(pub(crate) &'a str);

impl<'a> PrivateUseTagView<'a> {
    /// Validates and parses a full string slice including `x-` prefix into a private-use tag borrowed view.
    ///
    /// Validates whether the input is a well-formed private-use tag. Upon success,
    /// the internal slice stores the subtag sequence stripped of the `x-` (or `X-`) prefix.
    ///
    /// Callable in `const` contexts.
    ///
    /// # Errors
    ///
    /// Returns a [`ParseError`] if input is not a valid private-use tag.
    ///
    /// # Examples
    ///
    /// ```
    /// use fy_i18n_lang_tag::PrivateUseTagView;
    ///
    /// const VIEW: PrivateUseTagView<'static> = match PrivateUseTagView::parse("x-a-b-c") {
    ///     Ok(v) => v,
    ///     Err(_) => unreachable!(),
    /// };
    /// assert_eq!(VIEW.as_str(), "a-b-c");
    /// ```
    pub const fn parse(s: &'a str) -> Result<Self, ParseError> {
        let bytes = s.as_bytes();
        if bytes.is_empty() {
            return Err(ParseError::Empty);
        }

        // Must begin with 'x-' or 'X-' (at least 3 bytes, e.g., "x-a")
        if bytes.len() < 3 || !(bytes[0] == b'x' || bytes[0] == b'X') || bytes[1] != b'-' {
            return Err(ParseError::InvalidPrefix);
        }

        // Strip "x-" prefix (length 2) and delegate to from_subtags validation
        let (_, rest_bytes) = bytes.split_at(2);
        Self::from_subtags_inner(s, rest_bytes, 2)
    }

    /// Constructs and validates a view from a pure subtag sequence string without `x-` prefix.
    ///
    /// Validates whether the provided subtag sequence conforms to RFC 5646 private-use syntax:
    /// - Cannot be empty;
    /// - Composed of one or more subtags separated by `-`;
    /// - Cannot contain empty subtags (no consecutive hyphens `--`);
    /// - Cannot end with `-`;
    /// - Each subtag must be 1 to 8 ASCII letters or digits.
    ///
    /// Callable in `const` contexts.
    ///
    /// # Examples
    ///
    /// ```
    /// use fy_i18n_lang_tag::PrivateUseTagView;
    ///
    /// const VIEW: PrivateUseTagView<'static> = match PrivateUseTagView::from_subtags("a-b-c") {
    ///     Ok(v) => v,
    ///     Err(_) => unreachable!(),
    /// };
    /// assert_eq!(VIEW.as_str(), "a-b-c");
    /// assert_eq!(VIEW.to_string(), "x-a-b-c");
    /// ```
    pub const fn from_subtags(subtags: &'a str) -> Result<Self, ParseError> {
        Self::from_subtags_inner(subtags, subtags.as_bytes(), 0)
    }

    /// Internal unchecked constructor for already-validated subtag sequences.
    #[must_use]
    #[inline]
    pub(crate) const fn from_subtags_unchecked(subtags: &'a str) -> Self {
        Self(subtags)
    }

    /// Internal unified const validation logic.
    const fn from_subtags_inner(
        full_str: &'a str,
        subtags_bytes: &[u8],
        offset: usize,
    ) -> Result<Self, ParseError> {
        if subtags_bytes.is_empty() {
            return Err(ParseError::EmptySubtag);
        }

        // Cannot end with '-'
        if subtags_bytes[subtags_bytes.len() - 1] == b'-' {
            return Err(ParseError::TrailingSeparator);
        }

        let mut i = 0usize;
        let mut subtag_len = 0usize;

        while i < subtags_bytes.len() {
            let b = subtags_bytes[i];
            if b == b'-' {
                if subtag_len == 0 {
                    return Err(ParseError::EmptySubtag);
                }
                subtag_len = 0;
            } else {
                if !b.is_ascii_alphanumeric() {
                    return Err(ParseError::InvalidCharacter);
                }
                subtag_len += 1;
                if subtag_len > 8 {
                    return Err(ParseError::SubtagTooLong);
                }
            }
            i += 1;
        }

        let (_, tail) = full_str.split_at(offset);
        Ok(Self(tail))
    }

    /// Returns the subtag sequence slice without `x-` prefix (e.g., `"a-b-c"`).
    ///
    /// Unlike [`AsRef::as_ref`], this inherent method preserves the original lifetime `'a`.
    #[must_use]
    #[inline]
    pub const fn as_str(&self) -> &'a str {
        self.0
    }

    /// Iterates over individual private-use subtag slices (e.g., yielding `"a"`, `"b"`, `"c"`).
    #[inline]
    pub fn subtags(&self) -> impl Iterator<Item = &'a str> {
        self.0.split('-')
    }
}

impl<'a> AsRef<str> for PrivateUseTagView<'a> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.0
    }
}

impl<'a> Deref for PrivateUseTagView<'a> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a> TryFrom<&'a str> for PrivateUseTagView<'a> {
    type Error = ParseError;

    #[inline]
    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        Self::parse(s)
    }
}

/// Formats output in lowercase recommended format according to RFC 5646 §2.1.1 (e.g., `"x-a-b-c"`).
impl fmt::Display for PrivateUseTagView<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("x-")?;
        for b in self.0.bytes() {
            use core::fmt::Write;
            f.write_char(b.to_ascii_lowercase() as char)?;
        }
        Ok(())
    }
}

/// Case-insensitive equality comparison per RFC 5646.
impl PartialEq for PrivateUseTagView<'_> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0.eq_ignore_ascii_case(other.0)
    }
}

impl Eq for PrivateUseTagView<'_> {}

impl Hash for PrivateUseTagView<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for b in self.0.bytes() {
            state.write_u8(b.to_ascii_lowercase());
        }
        state.write_u8(0xFF);
    }
}

impl PartialOrd for PrivateUseTagView<'_> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PrivateUseTagView<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        let it1 = self.0.bytes().map(|b| b.to_ascii_lowercase());
        let it2 = other.0.bytes().map(|b| b.to_ascii_lowercase());
        it1.cmp(it2)
    }
}
