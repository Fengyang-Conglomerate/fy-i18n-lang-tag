//! Normal language tag (`NormalTagView`) borrowed view.

use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};

use super::error::ParseError;
use super::extensions::{self, OtherExtensionView, OtherExtensionsIter};
use super::language::{self, LanguageView};
use super::matching::{LanguageMatch, MatchQuality, StructuralFallbackIter};
use super::region::{self, RegionView};
use super::script::{self, ScriptView};
use super::t::TExtensionView;
use super::u::UExtensionView;
use crate::private_use::PrivateUseTagView;

/// Internal pre-parsed extension range indices.
///
/// Owned tags record field boundaries when writing normalized content to storage,
/// allowing projection of views in pure O(1) time without re-parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct ExtIndices {
    pub(crate) var_start: usize,
    pub(crate) var_end: usize,
    pub(crate) u_start: usize,
    pub(crate) u_end: usize,
    pub(crate) t_start: usize,
    pub(crate) t_end: usize,
    pub(crate) pu_start: usize,
    pub(crate) pu_end: usize,
    pub(crate) has_other: bool,
}

/// Zero-allocation borrowed view of a normal language tag.
///
/// Follows RFC 5646 syntax:
/// `language ["-" script] ["-" region] *("-" variant) *("-" extension) ["-" privateuse]`
///
/// Core subtags are managed via strongly-typed views:
/// - Primary language ([`LanguageView`]);
/// - Script ([`ScriptView`]);
/// - Region ([`RegionView`]);
///
/// Extension subtags are structurally divided into:
/// - Variants (`variants`);
/// - Unicode locale extension (`u`, RFC 6067);
/// - Transformed content extension (`t`, RFC 6497);
/// - Other single-letter extensions (accessed via [`other_extensions`](Self::other_extensions) iterator);
/// - Private-use subtag sequence (`private_use`, RFC 5646).
#[derive(Debug, Clone, Copy)]
pub struct NormalTagView<'a> {
    language: LanguageView<'a>,
    script: Option<&'a str>,
    region: Option<&'a str>,
    variants: Option<&'a str>,
    u: Option<UExtensionView<'a>>,
    t: Option<TExtensionView<'a>>,
    extensions_raw: Option<&'a str>,
    has_other: bool,
    private_use: Option<PrivateUseTagView<'a>>,
}

impl<'a> NormalTagView<'a> {
    /// Validates and parses a string slice into a normal language tag borrowed view.
    ///
    /// Callable in `const` contexts.
    ///
    /// # Errors
    ///
    /// Returns a [`ParseError`] if input does not conform to RFC 5646 syntax.
    ///
    /// # Examples
    ///
    /// ```
    /// use fy_i18n_lang_tag::NormalTagView;
    ///
    /// const VIEW: NormalTagView<'static> = match NormalTagView::parse("zh-Hans-CN-u-ca-gregory") {
    ///     Ok(v) => v,
    ///     Err(_) => unreachable!(),
    /// };
    /// assert_eq!(VIEW.language().as_str(), "zh");
    /// assert_eq!(VIEW.script(), Some("Hans"));
    /// assert_eq!(VIEW.region(), Some("CN"));
    /// ```
    pub const fn parse(s: &'a str) -> Result<Self, ParseError> {
        let bytes = s.as_bytes();
        if bytes.is_empty() {
            return Err(ParseError::Empty);
        }
        if bytes[bytes.len() - 1] == b'-' {
            return Err(ParseError::TrailingSeparator);
        }

        let mut pos = 0usize;

        // 1. Parse primary language: 2-8 ASCII letters (covering 4ALPHA reserved grammar)
        let lang_end = find_next_sep(bytes, pos);
        let lang_bytes = slice_range(bytes, pos, lang_end);
        if !language::is_valid_bytes(lang_bytes) {
            return Err(ParseError::InvalidLanguage);
        }
        let language = LanguageView::from_str_unchecked(match_str(s, pos, lang_end));
        pos = lang_end;

        let mut script: Option<&'a str> = None;
        let mut region: Option<&'a str> = None;
        let mut extensions_str: Option<&'a str> = None;

        // If there are subsequent subtags
        if pos < bytes.len() {
            pos += 1; // Skip '-'
            let next_end = find_next_sep(bytes, pos);
            let next_sub = slice_range(bytes, pos, next_end);

            // 2. Check if script: exactly 4 ASCII letters
            if script::is_valid_bytes(next_sub) {
                script = Some(match_str(s, pos, next_end));
                pos = next_end;
                if pos < bytes.len() {
                    pos += 1; // Skip '-'
                    let reg_end = find_next_sep(bytes, pos);
                    let reg_sub = slice_range(bytes, pos, reg_end);

                    // Check if script is followed by region
                    if region::is_valid_bytes(reg_sub) {
                        region = Some(match_str(s, pos, reg_end));
                        pos = reg_end;
                        if pos < bytes.len() {
                            pos += 1; // Skip '-'
                            extensions_str = Some(match_str(s, pos, bytes.len()));
                        }
                    } else {
                        // No region after script; remaining subtags belong to extensions
                        extensions_str = Some(match_str(s, pos, bytes.len()));
                    }
                }
            } else if region::is_valid_bytes(next_sub) {
                // No script; matches region directly (2 letters or 3 digits)
                region = Some(match_str(s, pos, next_end));
                pos = next_end;
                if pos < bytes.len() {
                    pos += 1; // Skip '-'
                    extensions_str = Some(match_str(s, pos, bytes.len()));
                }
            } else {
                // Neither script nor region; remainder belongs to extensions
                extensions_str = Some(match_str(s, pos, bytes.len()));
            }
        }

        // 3. Structurally parse extensions sequence
        let (variants, u, t, has_other, private_use) = if let Some(ext) = extensions_str {
            let parsed = match extensions::parse_and_validate_extensions_sequence(ext.as_bytes()) {
                Ok(p) => p,
                Err(err) => return Err(err),
            };
            let v = match parsed.variants {
                Some((s_idx, e_idx)) => Some(match_str(ext, s_idx, e_idx)),
                None => None,
            };
            let u_view = match parsed.u {
                Some((s_idx, e_idx)) => Some(super::u::UExtensionView::from_subtags_unchecked(
                    match_str(ext, s_idx, e_idx),
                )),
                None => None,
            };
            let t_view = match parsed.t {
                Some((s_idx, e_idx)) => Some(super::t::TExtensionView::from_subtags_unchecked(
                    match_str(ext, s_idx, e_idx),
                )),
                None => None,
            };
            let pu_view = match parsed.private_use {
                Some((s_idx, e_idx)) => Some(PrivateUseTagView::from_subtags_unchecked(match_str(
                    ext, s_idx, e_idx,
                ))),
                None => None,
            };
            (v, u_view, t_view, parsed.has_other, pu_view)
        } else {
            (None, None, None, false, None)
        };

        Ok(Self {
            language,
            script,
            region,
            variants,
            u,
            t,
            extensions_raw: extensions_str,
            has_other,
            private_use,
        })
    }

    /// Returns the primary language subtag view (e.g., `"en"`, `"zh"`).
    #[must_use]
    #[inline]
    pub const fn language(&self) -> LanguageView<'a> {
        self.language
    }

    /// Returns the primary language string slice.
    #[must_use]
    #[inline]
    pub const fn language_str(&self) -> &'a str {
        self.language.as_str()
    }

    /// Returns the script subtag string slice if present (e.g., `Some("Hans")`, `Some("Latn")`).
    #[must_use]
    #[inline]
    pub const fn script(&self) -> Option<&'a str> {
        self.script
    }

    /// Returns the strongly-typed script view if present.
    #[must_use]
    #[inline]
    pub const fn script_view(&self) -> Option<ScriptView<'a>> {
        match self.script {
            Some(s) => Some(ScriptView::from_str_unchecked(s)),
            None => None,
        }
    }

    /// Returns the region subtag string slice if present (e.g., `Some("CN")`, `Some("US")`, `Some("419")`).
    #[must_use]
    #[inline]
    pub const fn region(&self) -> Option<&'a str> {
        self.region
    }

    /// Returns the strongly-typed region view if present.
    #[must_use]
    #[inline]
    pub const fn region_view(&self) -> Option<RegionView<'a>> {
        match self.region {
            Some(r) => Some(RegionView::from_str_unchecked(r)),
            None => None,
        }
    }

    /// Returns the variant subtags sequence slice if present (e.g., `Some("rozaj-biske")`).
    #[must_use]
    #[inline]
    pub const fn variants(&self) -> Option<&'a str> {
        self.variants
    }

    /// Returns the Unicode locale extension view (`u` extension, RFC 6067).
    #[must_use]
    #[inline]
    pub const fn u(&self) -> Option<UExtensionView<'a>> {
        self.u
    }

    /// Returns the transformed content extension view (`t` extension, RFC 6497).
    #[must_use]
    #[inline]
    pub const fn t(&self) -> Option<TExtensionView<'a>> {
        self.t
    }

    /// Returns an iterator over all other single-letter extensions (excluding `u`, `t`, `x`, e.g., `a-foo`, `b-bar`).
    #[must_use]
    #[inline]
    pub fn other_extensions(&self) -> OtherExtensionsIter<'a> {
        if self.has_other {
            OtherExtensionsIter::new(self.extensions_raw.unwrap_or(""))
        } else {
            OtherExtensionsIter::new("")
        }
    }

    /// Retrieves the single-letter extension view matching the specified singleton.
    #[must_use]
    #[inline]
    pub fn other_extension(&self, singleton: char) -> Option<OtherExtensionView<'a>> {
        self.other_extensions().find_singleton(singleton)
    }

    /// Returns whether generic other single-letter extensions (excluding `u`, `t`, `x`) are present.
    #[must_use]
    #[inline]
    pub const fn has_other_extensions(&self) -> bool {
        self.has_other
    }

    /// Returns the private-use subtag sequence view (`x-` private use, reusing [`PrivateUseTagView`]).
    #[must_use]
    #[inline]
    pub const fn private_use(&self) -> Option<PrivateUseTagView<'a>> {
        self.private_use
    }

    /// Returns the raw slice of all extensions (if present).
    #[must_use]
    #[inline]
    pub const fn extensions(&self) -> Option<&'a str> {
        self.extensions_raw
    }

    /// Returns whether any extensions (variants, u/t/other extensions, or private use) are present.
    #[must_use]
    #[inline]
    pub const fn has_extensions(&self) -> bool {
        self.variants.is_some()
            || self.u.is_some()
            || self.t.is_some()
            || self.has_other
            || self.private_use.is_some()
    }

    /// Constructs a structural fallback iterator stripping components.
    #[must_use]
    #[inline]
    pub const fn structural_fallback(&self) -> StructuralFallbackIter<'a> {
        StructuralFallbackIter::new(*self)
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

    /// Internal unchecked constructor from un-indexed raw components.
    #[must_use]
    #[inline]
    pub(crate) const fn from_raw_parts(
        language: &'a str,
        script: Option<&'a str>,
        region: Option<&'a str>,
        extensions: Option<&'a str>,
    ) -> Self {
        let (variants, u, t, has_other, private_use, extensions_raw) = if let Some(ext) = extensions
        {
            match extensions::parse_and_validate_extensions_sequence(ext.as_bytes()) {
                Ok(parsed) => {
                    let v = match parsed.variants {
                        Some((s, e)) => Some(match_str(ext, s, e)),
                        None => None,
                    };
                    let u_view = match parsed.u {
                        Some((s, e)) => Some(super::u::UExtensionView::from_subtags_unchecked(
                            match_str(ext, s, e),
                        )),
                        None => None,
                    };
                    let t_view = match parsed.t {
                        Some((s, e)) => Some(super::t::TExtensionView::from_subtags_unchecked(
                            match_str(ext, s, e),
                        )),
                        None => None,
                    };
                    let pu_view = match parsed.private_use {
                        Some((s, e)) => Some(PrivateUseTagView::from_subtags_unchecked(match_str(
                            ext, s, e,
                        ))),
                        None => None,
                    };
                    (v, u_view, t_view, parsed.has_other, pu_view, Some(ext))
                }
                Err(_) => (None, None, None, false, None, None),
            }
        } else {
            (None, None, None, false, None, None)
        };

        Self {
            language: LanguageView::from_str_unchecked(language),
            script,
            region,
            variants,
            u,
            t,
            extensions_raw,
            has_other,
            private_use,
        }
    }

    /// Internal unchecked constructor from pre-computed indices (O(1) zero cost projection).
    #[must_use]
    #[inline]
    pub(crate) const fn from_raw_parts_with_indices(
        language: &'a str,
        script: Option<&'a str>,
        region: Option<&'a str>,
        extensions: Option<&'a str>,
        indices: &ExtIndices,
    ) -> Self {
        let (variants, u, t, pu) = if let Some(ext) = extensions {
            let v = if indices.var_start != indices.var_end {
                Some(match_str(ext, indices.var_start, indices.var_end))
            } else {
                None
            };
            let u_view = if indices.u_start != indices.u_end {
                Some(UExtensionView::from_subtags_unchecked(match_str(
                    ext,
                    indices.u_start,
                    indices.u_end,
                )))
            } else {
                None
            };
            let t_view = if indices.t_start != indices.t_end {
                Some(TExtensionView::from_subtags_unchecked(match_str(
                    ext,
                    indices.t_start,
                    indices.t_end,
                )))
            } else {
                None
            };
            let pu_view = if indices.pu_start != indices.pu_end {
                Some(PrivateUseTagView::from_subtags_unchecked(match_str(
                    ext,
                    indices.pu_start,
                    indices.pu_end,
                )))
            } else {
                None
            };
            (v, u_view, t_view, pu_view)
        } else {
            (None, None, None, None)
        };

        Self {
            language: LanguageView::from_str_unchecked(language),
            script,
            region,
            variants,
            u,
            t,
            extensions_raw: extensions,
            has_other: indices.has_other,
            private_use: pu,
        }
    }
}

/// Finds the next '-' hyphen or end of slice.
const fn find_next_sep(bytes: &[u8], start: usize) -> usize {
    let mut i = start;
    while i < bytes.len() {
        if bytes[i] == b'-' {
            return i;
        }
        i += 1;
    }
    bytes.len()
}

/// Safely extracts a string slice in const contexts.
const fn match_str(s: &str, start: usize, end: usize) -> &str {
    let (_, tail) = s.split_at(start);
    let (target, _) = tail.split_at(end - start);
    target
}

const fn slice_range(bytes: &[u8], start: usize, end: usize) -> &[u8] {
    let (_, tail) = bytes.split_at(start);
    let (target, _) = tail.split_at(end - start);
    target
}

impl<'a> TryFrom<&'a str> for NormalTagView<'a> {
    type Error = ParseError;

    #[inline]
    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        Self::parse(s)
    }
}

/// Formats output according to RFC 5646 §2.1.1 recommendations:
/// - Language in lowercase;
/// - Script in Titlecase;
/// - Region in uppercase;
/// - Extensions in lowercase;
/// - Single-letter extensions sorted alphabetically.
impl fmt::Display for NormalTagView<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use core::fmt::Write;

        // Language: all lowercase
        for b in self.language.as_bytes() {
            f.write_char(b.to_ascii_lowercase() as char)?;
        }

        // Script: Titlecase (first character uppercase, remaining lowercase)
        if let Some(script) = self.script {
            f.write_char('-')?;
            for (idx, b) in script.as_bytes().iter().enumerate() {
                if idx == 0 {
                    f.write_char(b.to_ascii_uppercase() as char)?;
                } else {
                    f.write_char(b.to_ascii_lowercase() as char)?;
                }
            }
        }

        // Region: all uppercase
        if let Some(region) = self.region {
            f.write_char('-')?;
            for b in region.as_bytes() {
                f.write_char(b.to_ascii_uppercase() as char)?;
            }
        }

        // Variants: all lowercase
        if let Some(variants) = self.variants {
            f.write_char('-')?;
            for b in variants.as_bytes() {
                f.write_char(b.to_ascii_lowercase() as char)?;
            }
        }

        // Collect all single-letter extension blocks (other, t, u), sorted according to RFC 5646 §4.5 step 1
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

        for ext in self.other_extensions() {
            if count < blocks.len() {
                blocks[count] = ExtBlock {
                    singleton: ext.singleton_byte().to_ascii_lowercase(),
                    subtags: ext.subtags(),
                };
                count += 1;
            }
        }

        if let Some(t) = self.t {
            if count < blocks.len() {
                blocks[count] = ExtBlock {
                    singleton: b't',
                    subtags: t.as_str(),
                };
                count += 1;
            }
        }

        if let Some(u) = self.u {
            if count < blocks.len() {
                blocks[count] = ExtBlock {
                    singleton: b'u',
                    subtags: u.as_str(),
                };
                count += 1;
            }
        }

        // Insertion sort
        let mut i = 1usize;
        while i < count {
            let mut j = i;
            while j > 0 && blocks[j - 1].singleton > blocks[j].singleton {
                blocks.swap(j - 1, j);
                j -= 1;
            }
            i += 1;
        }

        for block in &blocks[..count] {
            f.write_char('-')?;
            f.write_char((block.singleton as char).to_ascii_lowercase())?;
            f.write_char('-')?;
            for b in block.subtags.as_bytes() {
                f.write_char(b.to_ascii_lowercase() as char)?;
            }
        }

        // Private use extension (RFC 5646 puts it at the very end)
        if let Some(pu) = self.private_use {
            f.write_char('-')?;
            write!(f, "{}", pu)?;
        }

        Ok(())
    }
}

/// Case-insensitive equality comparison for language tags.
impl PartialEq for NormalTagView<'_> {
    fn eq(&self, other: &Self) -> bool {
        if self.language != other.language {
            return false;
        }
        match (self.script, other.script) {
            (Some(s1), Some(s2)) => {
                if !s1.eq_ignore_ascii_case(s2) {
                    return false;
                }
            }
            (None, None) => {}
            _ => return false,
        }
        match (self.region, other.region) {
            (Some(r1), Some(r2)) => {
                if !r1.eq_ignore_ascii_case(r2) {
                    return false;
                }
            }
            (None, None) => {}
            _ => return false,
        }
        match (self.variants, other.variants) {
            (Some(v1), Some(v2)) => {
                if !v1.eq_ignore_ascii_case(v2) {
                    return false;
                }
            }
            (None, None) => {}
            _ => return false,
        }
        if self.u != other.u {
            return false;
        }
        if self.t != other.t {
            return false;
        }
        if self.has_other != other.has_other {
            return false;
        }
        if self.has_other {
            let mut count_a = 0usize;
            for ext_a in self.other_extensions() {
                count_a += 1;
                match other.other_extension(ext_a.singleton()) {
                    Some(ext_b) => {
                        if ext_a != ext_b {
                            return false;
                        }
                    }
                    None => return false,
                }
            }
            let mut count_b = 0usize;
            for _ in other.other_extensions() {
                count_b += 1;
            }
            if count_a != count_b {
                return false;
            }
        }
        if self.private_use != other.private_use {
            return false;
        }
        true
    }
}

impl Eq for NormalTagView<'_> {}

impl Hash for NormalTagView<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for b in self.language.as_bytes() {
            state.write_u8(b.to_ascii_lowercase());
        }
        state.write_u8(0xFF);
        if let Some(script) = self.script {
            for b in script.bytes() {
                state.write_u8(b.to_ascii_lowercase());
            }
        }
        state.write_u8(0xFF);
        if let Some(region) = self.region {
            for b in region.bytes() {
                state.write_u8(b.to_ascii_lowercase());
            }
        }
        state.write_u8(0xFF);
        if let Some(variants) = self.variants {
            for b in variants.bytes() {
                state.write_u8(b.to_ascii_lowercase());
            }
        }
        state.write_u8(0xFF);
        if let Some(u) = self.u {
            u.hash(state);
        }
        state.write_u8(0xFF);
        if let Some(t) = self.t {
            t.hash(state);
        }
        state.write_u8(0xFF);
        if self.has_other {
            let mut other_exts: [Option<OtherExtensionView<'_>>; 36] = [None; 36];
            let mut count = 0usize;
            for ext in self.other_extensions() {
                if count < 36 {
                    other_exts[count] = Some(ext);
                    count += 1;
                }
            }
            let slice = &mut other_exts[..count];
            let mut i = 1usize;
            while i < slice.len() {
                let mut j = i;
                while j > 0 {
                    let prev_byte = match slice[j - 1] {
                        Some(ext) => ext.singleton_byte().to_ascii_lowercase(),
                        None => 0,
                    };
                    let curr_byte = match slice[j] {
                        Some(ext) => ext.singleton_byte().to_ascii_lowercase(),
                        None => 0,
                    };
                    if prev_byte > curr_byte {
                        slice.swap(j - 1, j);
                        j -= 1;
                    } else {
                        break;
                    }
                }
                i += 1;
            }
            for ext in slice.iter().flatten() {
                ext.hash(state);
            }
        }
        state.write_u8(0xFF);
        if let Some(pu) = self.private_use {
            state.write_u8(1);
            pu.hash(state);
        } else {
            state.write_u8(0);
        }
    }
}

impl PartialOrd for NormalTagView<'_> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NormalTagView<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        let ord = self.language.cmp(&other.language);
        if ord != Ordering::Equal {
            return ord;
        }

        let ord = self.script_view().cmp(&other.script_view());
        if ord != Ordering::Equal {
            return ord;
        }

        let ord = self.region_view().cmp(&other.region_view());
        if ord != Ordering::Equal {
            return ord;
        }

        let ord = match (self.variants, other.variants) {
            (Some(v1), Some(v2)) => {
                let a = v1.as_bytes();
                let b = v2.as_bytes();
                let min_len = if a.len() < b.len() { a.len() } else { b.len() };
                let mut i = 0;
                let mut res = Ordering::Equal;
                while i < min_len {
                    let ba = a[i].to_ascii_lowercase();
                    let bb = b[i].to_ascii_lowercase();
                    if ba != bb {
                        res = ba.cmp(&bb);
                        break;
                    }
                    i += 1;
                }
                if res == Ordering::Equal {
                    a.len().cmp(&b.len())
                } else {
                    res
                }
            }
            (Some(_), None) => Ordering::Greater,
            (None, Some(_)) => Ordering::Less,
            (None, None) => Ordering::Equal,
        };
        if ord != Ordering::Equal {
            return ord;
        }

        // Canonical collection of all single-letter extensions (other, t, u) for sorted comparison
        #[derive(Clone, Copy)]
        struct ExtBlock<'a> {
            singleton: u8,
            subtags: &'a str,
        }

        fn collect_blocks<'a>(view: &NormalTagView<'a>, blocks: &mut [ExtBlock<'a>; 36]) -> usize {
            let mut count = 0;
            for ext in view.other_extensions() {
                if count < 36 {
                    blocks[count] = ExtBlock {
                        singleton: ext.singleton_byte().to_ascii_lowercase(),
                        subtags: ext.subtags(),
                    };
                    count += 1;
                }
            }
            if let Some(t) = view.t() {
                if count < 36 {
                    blocks[count] = ExtBlock {
                        singleton: b't',
                        subtags: t.as_str(),
                    };
                    count += 1;
                }
            }
            if let Some(u) = view.u() {
                if count < 36 {
                    blocks[count] = ExtBlock {
                        singleton: b'u',
                        subtags: u.as_str(),
                    };
                    count += 1;
                }
            }
            let mut i = 1;
            while i < count {
                let mut j = i;
                while j > 0 && blocks[j - 1].singleton > blocks[j].singleton {
                    blocks.swap(j - 1, j);
                    j -= 1;
                }
                i += 1;
            }
            count
        }

        let mut blocks_a = [ExtBlock {
            singleton: 0,
            subtags: "",
        }; 36];
        let mut blocks_b = [ExtBlock {
            singleton: 0,
            subtags: "",
        }; 36];
        let count_a = collect_blocks(self, &mut blocks_a);
        let count_b = collect_blocks(other, &mut blocks_b);

        let min_count = if count_a < count_b { count_a } else { count_b };
        let mut i = 0;
        while i < min_count {
            if blocks_a[i].singleton != blocks_b[i].singleton {
                return blocks_a[i].singleton.cmp(&blocks_b[i].singleton);
            }
            let a = blocks_a[i].subtags.as_bytes();
            let b = blocks_b[i].subtags.as_bytes();
            let min_len = if a.len() < b.len() { a.len() } else { b.len() };
            let mut j = 0;
            let mut res = Ordering::Equal;
            while j < min_len {
                let ba = a[j].to_ascii_lowercase();
                let bb = b[j].to_ascii_lowercase();
                if ba != bb {
                    res = ba.cmp(&bb);
                    break;
                }
                j += 1;
            }
            let block_ord = if res == Ordering::Equal {
                a.len().cmp(&b.len())
            } else {
                res
            };
            if block_ord != Ordering::Equal {
                return block_ord;
            }
            i += 1;
        }
        if count_a != count_b {
            return count_a.cmp(&count_b);
        }

        self.private_use.cmp(&other.private_use)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_raw_parts_coverage() {
        let v_err = NormalTagView::from_raw_parts("zh", None, None, Some("-bad--ext-"));
        assert!(!v_err.has_extensions());
        assert_eq!(v_err.extensions(), None);
        assert_eq!(v_err.variants(), None);
        assert!(v_err.u().is_none());
        assert!(v_err.t().is_none());
        assert_eq!(v_err.other_extensions().count(), 0);

        let v_pu = NormalTagView::from_raw_parts("zh", None, None, Some("x-test"));
        assert!(v_pu.private_use().is_some());
        assert_eq!(v_pu.extensions(), Some("x-test"));
    }
}
