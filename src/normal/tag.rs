//! Owned normal language tag structure (`NormalTag`).

use core::cmp::Ordering;
use core::fmt;
use core::str::FromStr;

use super::error::{ParseError, TagError};
use super::extensions::{
    OtherExtensionView, OtherExtensionsIter, parse_and_validate_extensions_sequence,
};
use super::language::LanguageView;
use super::matching::{LanguageMatch, MatchQuality, StructuralFallbackIter};
use super::region::RegionView;
use super::script::ScriptView;
use super::t::TExtensionView;
use super::u::UExtensionView;
use super::view::{ExtIndices, NormalTagView};
use crate::private_use::PrivateUseTagView;
use crate::storage::{CapacityError, Fixed, Storage};

const LANG_START: usize = 0;
const LANG_END: usize = 8;
const SCRIPT_START: usize = 8;
const SCRIPT_END: usize = 12;
const REGION_START: usize = 12;
const REGION_END: usize = 15;
const CORE_BUF_LEN: usize = 15;

/// Generic owned structure for a normal language tag.
///
/// The fixed core subtags (primary language, script, region) are compactly packed into a single 15-byte array `[u8; 15]`:
/// - `0..8`: Primary language (up to 8 ASCII bytes, terminated with `0u8` if shorter, truncated at 8);
/// - `8..12`: Script (first byte `0u8` indicates `None`, otherwise 4-character Titlecase ASCII);
/// - `12..15`: Region (first byte `0u8` indicates `None`, terminated with `0u8` if 2 characters, truncated at 3).
///
/// Long-tail dynamic extensions (variants, u/t extensions, private-use subtags, etc.) are stored in the generic storage backend `S`.
/// The storage backend capacity `MAX` ([`Fixed`] defaults to 32 bytes, `Grow` defaults to `usize::MAX`)
/// **only bounds the extensions string** and does not consume space for core subtags.
///
/// Pre-computed boundary indices `ExtIndices` allow projecting [`as_view`](Self::as_view) without re-parsing;
/// UTF-8 validation is retained by default, and becomes pure O(1) when the `unsafe` feature is enabled.
///
/// # Examples
///
/// ```
/// use fy_i18n_lang_tag::NormalTag;
///
/// // Uses Fixed<32> storage backend by default
/// let tag: NormalTag = NormalTag::parse("zh-Hans-CN-u-ca-gregory").unwrap();
/// assert_eq!(tag.language().as_str(), "zh");
/// assert_eq!(tag.script(), Some("Hans"));
/// assert_eq!(tag.region(), Some("CN"));
/// assert_eq!(tag.extensions(), Some("u-ca-gregory"));
/// assert_eq!(tag.to_string(), "zh-Hans-CN-u-ca-gregory");
/// ```
///
/// Tags do not expose internal storage:
///
/// ```compile_fail
/// use fy_i18n_lang_tag::NormalTag;
/// let tag: NormalTag = NormalTag::parse("en").unwrap();
/// let _ = tag.extensions_storage();
/// ```
#[derive(Clone)]
pub struct NormalTag<S: Storage = Fixed<32>> {
    core: [u8; CORE_BUF_LEN],
    extensions: S,
    indices: ExtIndices,
}

impl<S: Storage> NormalTag<S> {
    /// Validates and parses a string slice into a normal language tag.
    ///
    /// # Errors
    ///
    /// Returns [`TagError::Parse`] if syntax is invalid, or [`TagError::Capacity`] if extensions exceed capacity.
    pub fn parse(s: &str) -> Result<Self, TagError> {
        let view = NormalTagView::parse(s)?;
        Self::from_view(view).map_err(TagError::from)
    }

    /// Parses using the specified storage backend instance.
    pub fn parse_with(s: &str, storage: S) -> Result<Self, TagError> {
        let view = NormalTagView::parse(s)?;
        Self::from_view_with(view, storage).map_err(TagError::from)
    }

    /// Constructs an owned tag from a validated normal tag view.
    ///
    /// # Errors
    ///
    /// Returns [`CapacityError`] if extensions exceed storage capacity.
    pub fn from_view(view: NormalTagView<'_>) -> Result<Self, CapacityError> {
        Self::from_view_with(view, S::default())
    }

    /// Constructs an owned tag from a borrowed view using the specified storage backend instance.
    pub fn from_view_with(view: NormalTagView<'_>, mut storage: S) -> Result<Self, CapacityError> {
        let mut core = [0u8; CORE_BUF_LEN];

        // 1. Normalize primary language (lowercase, zero-terminated if < 8 bytes)
        let lang_bytes = view.language().as_bytes();
        let mut i = 0usize;
        while i < lang_bytes.len() {
            core[LANG_START + i] = lang_bytes[i].to_ascii_lowercase();
            i += 1;
        }

        // 2. Normalize script (Titlecase: first uppercase, next 3 lowercase)
        if let Some(script) = view.script() {
            let bytes = script.as_bytes();
            core[SCRIPT_START] = bytes[0].to_ascii_uppercase();
            core[SCRIPT_START + 1] = bytes[1].to_ascii_lowercase();
            core[SCRIPT_START + 2] = bytes[2].to_ascii_lowercase();
            core[SCRIPT_START + 3] = bytes[3].to_ascii_lowercase();
        }

        // 3. Normalize region (uppercase or digits, zero-terminated if 2 chars)
        if let Some(region) = view.region() {
            let bytes = region.as_bytes();
            let mut j = 0usize;
            while j < bytes.len() {
                core[REGION_START + j] = bytes[j].to_ascii_uppercase();
                j += 1;
            }
        }

        // 4. Collect single-letter extension blocks and precalculate exact required byte count
        #[derive(Clone, Copy)]
        struct ExtBlock<'a> {
            singleton: u8,
            subtags: &'a str,
        }

        let mut blocks = [ExtBlock {
            singleton: 0,
            subtags: "",
        }; 36];
        let mut count = 0usize;

        for ext in view.other_extensions() {
            if count < blocks.len() {
                blocks[count] = ExtBlock {
                    singleton: ext.singleton_byte().to_ascii_lowercase(),
                    subtags: ext.subtags(),
                };
                count += 1;
            }
        }

        if let Some(t) = view.t() {
            if count < blocks.len() {
                blocks[count] = ExtBlock {
                    singleton: b't',
                    subtags: t.as_str(),
                };
                count += 1;
            }
        }

        if let Some(u) = view.u() {
            if count < blocks.len() {
                blocks[count] = ExtBlock {
                    singleton: b'u',
                    subtags: u.as_str(),
                };
                count += 1;
            }
        }

        // Insertion sort extension blocks (lowercase ASCII ascending)
        let mut i = 1usize;
        while i < count {
            let mut j = i;
            while j > 0 && blocks[j - 1].singleton > blocks[j].singleton {
                blocks.swap(j - 1, j);
                j -= 1;
            }
            i += 1;
        }

        // Precalculate exact byte count; fail immediately if exceeding capacity
        let mut total_required = 0usize;
        if let Some(variants) = view.variants() {
            total_required = total_required.saturating_add(variants.len());
        }
        for block in &blocks[..count] {
            if total_required > 0 {
                total_required = total_required.saturating_add(1);
            }
            total_required = total_required.saturating_add(2);
            total_required = total_required.saturating_add(block.subtags.len());
        }
        if let Some(pu) = view.private_use() {
            if total_required > 0 {
                total_required = total_required.saturating_add(1);
            }
            total_required = total_required.saturating_add(2);
            total_required = total_required.saturating_add(pu.as_str().len());
        }
        if total_required > storage.max_capacity() {
            return Err(CapacityError::new(total_required, storage.max_capacity()));
        }

        // 5. Sequentially write extensions to storage and record indices
        storage.clear();
        let mut indices = ExtIndices::default();
        let mut has_written = false;

        // 5.1 Write variants
        if let Some(variants) = view.variants() {
            crate::tag_bytes::push_lowercase(&mut storage, variants)?;
            indices.var_start = 0;
            indices.var_end = storage.len();
            has_written = true;
        }

        // 5.2 Write sorted single-letter extension blocks
        for block in &blocks[..count] {
            if has_written {
                storage.push_byte(b'-')?;
            }
            storage.push_byte(block.singleton)?;
            storage.push_byte(b'-')?;
            let sub_start = storage.len();
            crate::tag_bytes::push_lowercase(&mut storage, block.subtags)?;
            let sub_end = storage.len();
            if block.singleton == b'u' {
                indices.u_start = sub_start;
                indices.u_end = sub_end;
            } else if block.singleton == b't' {
                indices.t_start = sub_start;
                indices.t_end = sub_end;
            } else {
                indices.has_other = true;
            }
            has_written = true;
        }

        // 5.3 Write private use extension (fixed at the very end per RFC 5646)
        if let Some(pu) = view.private_use() {
            if has_written {
                storage.push_byte(b'-')?;
            }
            storage.push_byte(b'x')?;
            storage.push_byte(b'-')?;
            let pu_start = storage.len();
            crate::tag_bytes::push_lowercase(&mut storage, pu.as_str())?;
            let pu_end = storage.len();
            indices.pu_start = pu_start;
            indices.pu_end = pu_end;
        }

        Ok(Self {
            core,
            extensions: storage,
            indices,
        })
    }

    /// Directly constructs an owned tag from component views (using default storage backend).
    ///
    /// LanguageView carries type-level proof of validity; no redundant validation is performed.
    /// Passing `Some("")` to `script`, `region`, or `extensions` automatically normalizes to `None`.
    pub fn from_components(
        language: LanguageView<'_>,
        script: Option<&str>,
        region: Option<&str>,
        extensions: Option<&str>,
    ) -> Result<Self, TagError> {
        Self::from_components_with(language, script, region, extensions, S::default())
    }

    /// Constructs an owned tag from component views using the specified storage backend.
    ///
    /// Passing `Some("")` to `script`, `region`, or `extensions` automatically normalizes to `None`.
    pub fn from_components_with(
        language: LanguageView<'_>,
        script: Option<&str>,
        region: Option<&str>,
        extensions: Option<&str>,
        storage: S,
    ) -> Result<Self, TagError> {
        let script = match script {
            Some(s) if !s.is_empty() => Some(s),
            _ => None,
        };
        let region = match region {
            Some(r) if !r.is_empty() => Some(r),
            _ => None,
        };
        let extensions = match extensions {
            Some(e) if !e.is_empty() => Some(e),
            _ => None,
        };

        let script_str = match script {
            Some(s) => Some(ScriptView::parse(s)?.as_str()),
            None => None,
        };
        let region_str = match region {
            Some(r) => Some(RegionView::parse(r)?.as_str()),
            None => None,
        };
        if let Some(ext) = extensions {
            parse_and_validate_extensions_sequence(ext.as_bytes())?;
        }
        let view =
            NormalTagView::from_raw_parts(language.as_str(), script_str, region_str, extensions);
        Self::from_view_with(view, storage).map_err(TagError::from)
    }

    /// Returns the primary language subtag view (normalized to lowercase).
    #[must_use]
    #[inline]
    pub fn language(&self) -> LanguageView<'_> {
        let mut end = LANG_START;
        while end < LANG_END && self.core[end] != 0 {
            end += 1;
        }
        debug_assert!(
            end - LANG_START >= 2,
            "invariant violated: language subtag must be at least 2 bytes"
        );
        LanguageView::from_str_unchecked(crate::tag_bytes::as_str(&self.core[LANG_START..end]))
    }

    /// Returns the primary language string slice (normalized to lowercase).
    #[must_use]
    #[inline]
    pub fn language_str(&self) -> &str {
        self.language().as_str()
    }

    /// Derives a new tag with the given strongly-typed language view.
    ///
    /// LanguageView carries type-level proof of validity; construction performs zero syntax checks.
    /// Language is stored in the 15-byte core buffer, posing zero capacity risk.
    #[must_use]
    pub fn with_language(&self, language: LanguageView<'_>) -> Self
    where
        S: Clone,
    {
        let mut new_tag = self.clone();
        let bytes = language.as_bytes();
        let len = bytes.len();
        debug_assert!(
            (2..=8).contains(&len),
            "invariant violated: language length must be between 2 and 8"
        );
        let write_len = if len > 8 { 8 } else { len };
        let mut i = 0usize;
        while i < write_len {
            new_tag.core[LANG_START + i] = bytes[i].to_ascii_lowercase();
            i += 1;
        }
        while i < 8 {
            new_tag.core[LANG_START + i] = 0;
            i += 1;
        }
        new_tag
    }

    /// Derives a new tag with an unvalidated string slice.
    pub fn with_language_str(&self, language: &str) -> Result<Self, ParseError>
    where
        S: Clone,
    {
        let view = LanguageView::parse(language)?;
        Ok(self.with_language(view))
    }

    /// Returns the script subtag string slice if present (normalized to Titlecase).
    #[must_use]
    #[inline]
    pub fn script(&self) -> Option<&str> {
        if self.core[SCRIPT_START] == 0 {
            None
        } else {
            Some(crate::tag_bytes::as_str(
                &self.core[SCRIPT_START..SCRIPT_END],
            ))
        }
    }

    /// Returns the strongly-typed script view if present.
    #[must_use]
    #[inline]
    pub fn script_view(&self) -> Option<ScriptView<'_>> {
        self.script().map(ScriptView::from_str_unchecked)
    }

    /// Derives a new tag with an unvalidated script string slice.
    ///
    /// Passing `None` or `Some("")` clears the script subtag.
    pub fn with_script(&self, script: Option<&str>) -> Result<Self, ParseError>
    where
        S: Clone,
    {
        let script = match script {
            Some(s) if !s.is_empty() => Some(s),
            _ => None,
        };
        let mut new_tag = self.clone();
        match script {
            Some(s) => {
                let view = ScriptView::parse(s)?;
                let bytes = view.as_bytes();
                new_tag.core[SCRIPT_START] = bytes[0].to_ascii_uppercase();
                new_tag.core[SCRIPT_START + 1] = bytes[1].to_ascii_lowercase();
                new_tag.core[SCRIPT_START + 2] = bytes[2].to_ascii_lowercase();
                new_tag.core[SCRIPT_START + 3] = bytes[3].to_ascii_lowercase();
            }
            None => {
                new_tag.core[SCRIPT_START..SCRIPT_END].fill(0);
            }
        }
        Ok(new_tag)
    }

    /// Derives a new tag with a strongly-typed script view (zero syntax checks).
    #[must_use]
    pub fn with_script_view(&self, script: Option<ScriptView<'_>>) -> Self
    where
        S: Clone,
    {
        let mut new_tag = self.clone();
        match script {
            Some(s) => {
                let bytes = s.as_bytes();
                debug_assert_eq!(
                    bytes.len(),
                    4,
                    "invariant violated: script subtag must be 4 bytes"
                );
                new_tag.core[SCRIPT_START] = bytes[0].to_ascii_uppercase();
                new_tag.core[SCRIPT_START + 1] = bytes[1].to_ascii_lowercase();
                new_tag.core[SCRIPT_START + 2] = bytes[2].to_ascii_lowercase();
                new_tag.core[SCRIPT_START + 3] = bytes[3].to_ascii_lowercase();
            }
            None => {
                new_tag.core[SCRIPT_START..SCRIPT_END].fill(0);
            }
        }
        new_tag
    }

    /// Returns the region subtag string slice if present (normalized to uppercase).
    #[must_use]
    #[inline]
    pub fn region(&self) -> Option<&str> {
        if self.core[REGION_START] == 0 {
            None
        } else {
            let mut end = REGION_START + 2;
            while end < REGION_END && self.core[end] != 0 {
                end += 1;
            }
            Some(crate::tag_bytes::as_str(&self.core[REGION_START..end]))
        }
    }

    /// Returns the strongly-typed region view if present.
    #[must_use]
    #[inline]
    pub fn region_view(&self) -> Option<RegionView<'_>> {
        self.region().map(RegionView::from_str_unchecked)
    }

    /// Derives a new tag with an unvalidated region string slice.
    ///
    /// Passing `None` or `Some("")` clears the region subtag.
    pub fn with_region(&self, region: Option<&str>) -> Result<Self, ParseError>
    where
        S: Clone,
    {
        let region = match region {
            Some(r) if !r.is_empty() => Some(r),
            _ => None,
        };
        let mut new_tag = self.clone();
        match region {
            Some(r) => {
                let view = RegionView::parse(r)?;
                let bytes = view.as_bytes();
                let mut j = 0usize;
                while j < bytes.len() {
                    new_tag.core[REGION_START + j] = bytes[j].to_ascii_uppercase();
                    j += 1;
                }
                if j < 3 {
                    new_tag.core[REGION_START + j] = 0;
                }
            }
            None => {
                new_tag.core[REGION_START..REGION_END].fill(0);
            }
        }
        Ok(new_tag)
    }

    /// Derives a new tag with a strongly-typed region view (zero syntax checks).
    #[must_use]
    pub fn with_region_view(&self, region: Option<RegionView<'_>>) -> Self
    where
        S: Clone,
    {
        let mut new_tag = self.clone();
        match region {
            Some(r) => {
                let bytes = r.as_bytes();
                let len = bytes.len();
                debug_assert!(
                    len == 2 || len == 3,
                    "invariant violated: region subtag must be 2 or 3 bytes"
                );
                let write_len = if len > 3 { 3 } else { len };
                let mut j = 0usize;
                while j < write_len {
                    new_tag.core[REGION_START + j] = bytes[j].to_ascii_uppercase();
                    j += 1;
                }
                while j < 3 {
                    new_tag.core[REGION_START + j] = 0;
                    j += 1;
                }
            }
            None => {
                new_tag.core[REGION_START..REGION_END].fill(0);
            }
        }
        new_tag
    }

    /// Derives a clean tag stripping all extensions (variants, u/t/other extensions, and private use).
    #[must_use]
    pub fn without_extensions(&self) -> Self
    where
        S: Clone,
    {
        let mut new_storage = self.extensions.clone();
        new_storage.clear();
        Self {
            core: self.core,
            extensions: new_storage,
            indices: ExtIndices::default(),
        }
    }

    /// Derives a new tag with the specified extensions string slice.
    ///
    /// Passing `None` or `Some("")` clears all extensions.
    pub fn with_extensions(&self, extensions: Option<&str>) -> Result<Self, TagError>
    where
        S: Clone,
    {
        let extensions = match extensions {
            Some(e) if !e.is_empty() => Some(e),
            _ => None,
        };
        if let Some(ext) = extensions {
            parse_and_validate_extensions_sequence(ext.as_bytes())?;
        }
        let view = NormalTagView::from_raw_parts(
            self.language().as_str(),
            self.script(),
            self.region(),
            extensions,
        );
        let mut storage = self.extensions.clone();
        storage.clear();
        Self::from_view_with(view, storage).map_err(TagError::from)
    }

    /// Returns the extensions string slice if present.
    #[must_use]
    #[inline]
    pub fn extensions(&self) -> Option<&str> {
        if self.extensions.is_empty() {
            None
        } else {
            Some(crate::tag_bytes::as_str(self.extensions.as_bytes()))
        }
    }

    /// Returns whether extensions are present.
    #[must_use]
    #[inline]
    pub fn has_extensions(&self) -> bool {
        !self.extensions.is_empty()
    }

    /// Returns the maximum capacity of the extensions storage backend.
    #[must_use]
    #[inline]
    pub fn extensions_max_capacity(&self) -> usize {
        self.extensions.max_capacity()
    }

    /// Borrows as a lightweight [`NormalTagView`].
    ///
    /// Pre-computed boundary indices avoid re-parsing; UTF-8 validation is retained by default,
    /// becoming pure O(1) when the `unsafe` feature is enabled.
    #[must_use]
    #[inline]
    pub fn as_view(&self) -> NormalTagView<'_> {
        NormalTagView::from_raw_parts_with_indices(
            self.language().as_str(),
            self.script(),
            self.region(),
            self.extensions(),
            &self.indices,
        )
    }

    /// Returns the variant subtags sequence slice if present.
    #[must_use]
    #[inline]
    pub fn variants(&self) -> Option<&str> {
        self.as_view().variants()
    }

    /// Returns the Unicode locale extension view (`u` extension, RFC 6067).
    #[must_use]
    #[inline]
    pub fn u(&self) -> Option<UExtensionView<'_>> {
        self.as_view().u()
    }

    /// Returns the transformed content extension view (`t` extension, RFC 6497).
    #[must_use]
    #[inline]
    pub fn t(&self) -> Option<TExtensionView<'_>> {
        self.as_view().t()
    }

    /// Returns an iterator over all other single-letter extensions (excluding `u`, `t`, `x`, e.g., `a-foo`, `b-bar`).
    #[must_use]
    #[inline]
    pub fn other_extensions(&self) -> OtherExtensionsIter<'_> {
        self.as_view().other_extensions()
    }

    /// Retrieves the single-letter extension view matching the specified singleton.
    #[must_use]
    #[inline]
    pub fn other_extension(&self, singleton: char) -> Option<OtherExtensionView<'_>> {
        self.as_view().other_extension(singleton)
    }

    /// Returns whether generic other single-letter extensions (excluding `u`, `t`, `x`) are present.
    #[must_use]
    #[inline]
    pub fn has_other_extensions(&self) -> bool {
        self.indices.has_other
    }

    /// Returns the private-use subtag sequence view (`x-` private use, reusing [`PrivateUseTagView`]).
    #[must_use]
    #[inline]
    pub fn private_use(&self) -> Option<PrivateUseTagView<'_>> {
        self.as_view().private_use()
    }

    /// Constructs a structural fallback iterator stripping components.
    #[must_use]
    #[inline]
    pub fn structural_fallback(&self) -> StructuralFallbackIter<'_> {
        self.as_view().structural_fallback()
    }

    /// Evaluates structural matching against a candidate.
    #[must_use]
    #[inline]
    pub fn structural_matches<C: ?Sized>(&self, candidate: &C) -> bool
    where
        Self: LanguageMatch<C>,
    {
        self.matches(candidate)
    }

    /// Evaluates structural matching quality against a candidate.
    #[must_use]
    #[inline]
    pub fn structural_match_quality<C: ?Sized>(&self, candidate: &C) -> MatchQuality
    where
        Self: LanguageMatch<C>,
    {
        self.match_quality(candidate)
    }
}

impl<S: Storage> fmt::Display for NormalTag<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.language())?;
        if let Some(script) = self.script() {
            write!(f, "-{}", script)?;
        }
        if let Some(region) = self.region() {
            write!(f, "-{}", region)?;
        }
        if let Some(ext) = self.extensions() {
            write!(f, "-{}", ext)?;
        }
        Ok(())
    }
}

impl<S: Storage> fmt::Debug for NormalTag<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NormalTag(\"{self}\")")
    }
}

impl<S: Storage> TryFrom<&str> for NormalTag<S> {
    type Error = TagError;

    #[inline]
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::parse(s)
    }
}

impl<S: Storage> FromStr for NormalTag<S> {
    type Err = TagError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

// Supports mutual comparison between owned tags (delegated to borrowed views for transitivity).
impl<S1: Storage, S2: Storage> PartialEq<NormalTag<S2>> for NormalTag<S1> {
    #[inline]
    fn eq(&self, other: &NormalTag<S2>) -> bool {
        self.as_view() == other.as_view()
    }
}

impl<S: Storage> Eq for NormalTag<S> {}

impl<S1: Storage, S2: Storage> PartialOrd<NormalTag<S2>> for NormalTag<S1> {
    #[inline]
    fn partial_cmp(&self, other: &NormalTag<S2>) -> Option<Ordering> {
        Some(self.as_view().cmp(&other.as_view()))
    }
}

impl<S: Storage> Ord for NormalTag<S> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_view().cmp(&other.as_view())
    }
}

impl<S: Storage> core::hash::Hash for NormalTag<S> {
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.as_view().hash(state);
    }
}

// Supports mutual comparison between owned tags and borrowed views.
impl<S: Storage> PartialEq<NormalTagView<'_>> for NormalTag<S> {
    #[inline]
    fn eq(&self, other: &NormalTagView<'_>) -> bool {
        self.as_view() == *other
    }
}

impl<S: Storage> PartialEq<NormalTag<S>> for NormalTagView<'_> {
    #[inline]
    fn eq(&self, other: &NormalTag<S>) -> bool {
        *self == other.as_view()
    }
}

impl<S: Storage> PartialOrd<NormalTagView<'_>> for NormalTag<S> {
    #[inline]
    fn partial_cmp(&self, other: &NormalTagView<'_>) -> Option<Ordering> {
        Some(self.as_view().cmp(other))
    }
}

impl<S: Storage> PartialOrd<NormalTag<S>> for NormalTagView<'_> {
    #[inline]
    fn partial_cmp(&self, other: &NormalTag<S>) -> Option<Ordering> {
        Some(self.cmp(&other.as_view()))
    }
}
