//! Simple syntax-level language tag matching and structural fallback.
//!
//! Built-in matching follows fixed rules: full tag equality is prioritized; otherwise, only language, script, and region are compared.
//! Tags do not match if primary languages differ or if explicit script/region disagree; omitting a subtag on one side does not conflict.
//! Variants and extensions only affect full equality (Exact) and do not alter core ratings.
//! Does not depend on registry or CLDR data; does not infer relationships like `zh-TW` and `zh-Hant`.
//! This fixed structural matching and component fallback does not claim to implement RFC 4647 Lookup.
//!
//! # Architecture and Extensibility
//!
//! Defines a unified [`LanguageMatch`] trait.
//! Higher-level match packages (such as `fy-i18n-lang-match`) or facade packages (such as `fy-i18n-lang`)
//! can provide advanced semantic matching with CLDR data (Likely Subtags, alias mapping, macrolanguage coverage)
//! by extending this trait or re-exporting advanced matchers.

use super::tag::NormalTag;
use super::view::NormalTagView;
use crate::storage::Storage;

/// Language tag match quality levels.
///
/// Fixed priority: Exact > CoreExact > LanguageScript > LanguageRegion > Language > NoMatch.
/// Script takes precedence over region; no policy switches or weights. Variants and extensions only affect Exact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum MatchQuality {
    /// No match (primary language mismatch, or conflicting script/region).
    NoMatch = 0,
    /// Primary language matches only (e.g., request `"zh-Hans-CN"` vs candidate `"zh"`).
    Language = 1,
    /// Same explicitly specified region, script specified by only one party (e.g., `"zh-Hans-CN"` vs `"zh-CN"`).
    LanguageRegion = 2,
    /// Same explicitly specified script, region specified by only one party (e.g., `"zh-Hans-CN"` vs `"zh-Hans"`).
    LanguageScript = 3,
    /// Language, script, and region identical in value and omission status, but full tags are not equal.
    CoreExact = 4,
    /// Strict exact match (core subtags, variants, and extensions completely identical).
    Exact = 5,
}

/// Language tag match trait.
///
/// Enables seamless matching between borrowed views [`NormalTagView`], owned tags [`NormalTag`],
/// and raw string slices.
pub trait LanguageMatch<Candidate: ?Sized = Self> {
    /// Checks whether the candidate language tag structurally matches the request tag.
    #[must_use]
    fn matches(&self, candidate: &Candidate) -> bool {
        self.match_quality(candidate) > MatchQuality::NoMatch
    }

    /// Evaluates the match quality level of candidate relative to request.
    #[must_use]
    fn match_quality(&self, candidate: &Candidate) -> MatchQuality;
}

/// Evaluates the fixed structural match quality between two borrowed views.
///
/// Symmetric between both sides. Tags do not match if primary languages differ or if explicit script/region disagree.
/// Outside of full equality, only core subtags are compared; differences in variants or extensions do not reject matching.
#[must_use]
pub fn eval_match_quality(
    request: &NormalTagView<'_>,
    candidate: &NormalTagView<'_>,
) -> MatchQuality {
    // 1. Exactly identical (case-insensitive)
    if request == candidate {
        return MatchQuality::Exact;
    }

    // 2. Primary language must match
    if request.language() != candidate.language() {
        return MatchQuality::NoMatch;
    }

    // 3. Script comparison
    let script_match = match (request.script(), candidate.script()) {
        (Some(s1), Some(s2)) => {
            if !s1.eq_ignore_ascii_case(s2) {
                return MatchQuality::NoMatch; // Conflicting script (e.g., Hans vs Hant)
            }
            true
        }
        (None, None) => true,
        _ => false, // One side omitted script
    };

    // 4. Region comparison
    let region_match = match (request.region(), candidate.region()) {
        (Some(r1), Some(r2)) => {
            if !r1.eq_ignore_ascii_case(r2) {
                return MatchQuality::NoMatch; // Conflicting region (e.g., CN vs TW)
            }
            true
        }
        (None, None) => true,
        _ => false, // One side omitted region
    };

    // 5. Outside of exact equality, rating is based on core subtags only.
    if script_match && region_match {
        MatchQuality::CoreExact
    } else if script_match && request.script().is_some() {
        MatchQuality::LanguageScript
    } else if region_match && request.region().is_some() {
        MatchQuality::LanguageRegion
    } else {
        MatchQuality::Language
    }
}

impl<'a, 'b> LanguageMatch<NormalTagView<'b>> for NormalTagView<'a> {
    #[inline]
    fn match_quality(&self, candidate: &NormalTagView<'b>) -> MatchQuality {
        eval_match_quality(self, candidate)
    }
}

impl<'a, S: Storage> LanguageMatch<NormalTag<S>> for NormalTagView<'a> {
    #[inline]
    fn match_quality(&self, candidate: &NormalTag<S>) -> MatchQuality {
        eval_match_quality(self, &candidate.as_view())
    }
}

impl<'a> LanguageMatch<str> for NormalTagView<'a> {
    fn match_quality(&self, candidate: &str) -> MatchQuality {
        match NormalTagView::parse(candidate) {
            Ok(view) => eval_match_quality(self, &view),
            Err(_) => MatchQuality::NoMatch,
        }
    }
}

impl<'a> LanguageMatch<&str> for NormalTagView<'a> {
    #[inline]
    fn match_quality(&self, candidate: &&str) -> MatchQuality {
        self.match_quality(*candidate)
    }
}

impl<S1: Storage, S2: Storage> LanguageMatch<NormalTag<S2>> for NormalTag<S1> {
    #[inline]
    fn match_quality(&self, candidate: &NormalTag<S2>) -> MatchQuality {
        eval_match_quality(&self.as_view(), &candidate.as_view())
    }
}

impl<S: Storage> LanguageMatch<NormalTagView<'_>> for NormalTag<S> {
    #[inline]
    fn match_quality(&self, candidate: &NormalTagView<'_>) -> MatchQuality {
        eval_match_quality(&self.as_view(), candidate)
    }
}

impl<S: Storage> LanguageMatch<str> for NormalTag<S> {
    #[inline]
    fn match_quality(&self, candidate: &str) -> MatchQuality {
        self.as_view().match_quality(candidate)
    }
}

impl<S: Storage> LanguageMatch<&str> for NormalTag<S> {
    #[inline]
    fn match_quality(&self, candidate: &&str) -> MatchQuality {
        self.as_view().match_quality(*candidate)
    }
}

impl<'b> LanguageMatch<NormalTagView<'b>> for str {
    fn match_quality(&self, candidate: &NormalTagView<'b>) -> MatchQuality {
        match NormalTagView::parse(self) {
            Ok(view) => eval_match_quality(&view, candidate),
            Err(_) => MatchQuality::NoMatch,
        }
    }
}

impl<S: Storage> LanguageMatch<NormalTag<S>> for str {
    fn match_quality(&self, candidate: &NormalTag<S>) -> MatchQuality {
        match NormalTagView::parse(self) {
            Ok(view) => eval_match_quality(&view, &candidate.as_view()),
            Err(_) => MatchQuality::NoMatch,
        }
    }
}

impl LanguageMatch<str> for str {
    fn match_quality(&self, candidate: &str) -> MatchQuality {
        match (NormalTagView::parse(self), NormalTagView::parse(candidate)) {
            (Ok(v1), Ok(v2)) => eval_match_quality(&v1, &v2),
            _ => MatchQuality::NoMatch,
        }
    }
}

impl LanguageMatch<&str> for str {
    #[inline]
    fn match_quality(&self, candidate: &&str) -> MatchQuality {
        self.match_quality(*candidate)
    }
}

impl<C: ?Sized> LanguageMatch<C> for &str
where
    str: LanguageMatch<C>,
{
    #[inline]
    fn match_quality(&self, candidate: &C) -> MatchQuality {
        (*self).match_quality(candidate)
    }
}

/// Multi-request language tag match result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MatchResult<'a, C> {
    /// Index of matching request in `requests` slice.
    pub request_index: usize,
    /// Index of matching candidate in `available` slice.
    pub candidate_index: usize,
    /// Reference to matched candidate.
    pub candidate: &'a C,
    /// Match quality level.
    pub quality: MatchQuality,
}

impl<'a, C> MatchResult<'a, C> {
    /// Returns the matched candidate tag reference.
    #[must_use]
    #[inline]
    pub const fn candidate(&self) -> &'a C {
        self.candidate
    }

    /// Returns the index of the matched request in priority order.
    #[must_use]
    #[inline]
    pub const fn request_index(&self) -> usize {
        self.request_index
    }

    /// Returns the index of the matched candidate in the available list.
    #[must_use]
    #[inline]
    pub const fn candidate_index(&self) -> usize {
        self.candidate_index
    }

    /// Returns the match quality level.
    #[must_use]
    #[inline]
    pub const fn quality(&self) -> MatchQuality {
        self.quality
    }

    /// Deconstructs into `(request_index, candidate_index, &candidate)` tuple.
    #[must_use]
    #[inline]
    pub const fn into_tuple(self) -> (usize, usize, &'a C) {
        (self.request_index, self.candidate_index, self.candidate)
    }
}

/// Finds the candidate tag in a slice that has the highest structural match quality with the request tag.
///
/// Uses fixed six-level structural matching rules, without depending on external CLDR data.
/// Returns the first appearing candidate for ties; returns None on empty lists or when all candidates fail to match.
///
/// # Examples
///
/// ```
/// use fy_i18n_lang_tag::{NormalTag, match_language};
///
/// let req: NormalTag = NormalTag::parse("zh-Hans-CN-u-ca-gregory").unwrap();
/// let available: [NormalTag; 4] = [
///     NormalTag::parse("en-US").unwrap(),
///     NormalTag::parse("zh").unwrap(),
///     NormalTag::parse("zh-Hans").unwrap(),
///     NormalTag::parse("zh-Hans-CN").unwrap(),
/// ];
///
/// let (idx, matched) = match_language(&req, &available).unwrap();
/// assert_eq!(idx, 3);
/// assert_eq!(matched.to_string(), "zh-Hans-CN");
/// ```
#[must_use]
pub fn match_language<'a, R, C>(request: &R, available: &'a [C]) -> Option<(usize, &'a C)>
where
    R: LanguageMatch<C> + ?Sized,
{
    let mut best: Option<(usize, &'a C, MatchQuality)> = None;

    for (idx, candidate) in available.iter().enumerate() {
        let quality = request.match_quality(candidate);
        if quality > MatchQuality::NoMatch {
            match best {
                Some((_, _, best_quality)) => {
                    if quality > best_quality {
                        best = Some((idx, candidate, quality));
                    }
                }
                None => {
                    best = Some((idx, candidate, quality));
                }
            }
        }
    }

    best.map(|(idx, cand, _)| (idx, cand))
}

/// Finds the best matching candidate in priority order across multiple requests.
///
/// Evaluates each request in `requests` sequentially (first trying higher priority, e.g., `zh-Hans`, then `zh-Hant`, then `en`).
/// For the current request, searches `available` for the highest match quality above [`MatchQuality::NoMatch`].
/// Returns immediately upon finding a valid match for a higher priority request.
/// Returns `None` if no request matches any candidate.
///
/// # Examples
///
/// ```
/// use fy_i18n_lang_tag::{NormalTag, match_languages, MatchQuality};
///
/// let requests: [NormalTag; 3] = [
///     NormalTag::parse("zh-Hans").unwrap(), // Priority 1: Simplified Chinese
///     NormalTag::parse("zh-Hant").unwrap(), // Priority 2: Traditional Chinese
///     NormalTag::parse("en").unwrap(),      // Priority 3: English
/// ];
///
/// let available: [NormalTag; 2] = [
///     NormalTag::parse("en-US").unwrap(),
///     NormalTag::parse("zh-Hant-TW").unwrap(),
/// ];
///
/// let matched = match_languages(&requests, &available).unwrap();
/// assert_eq!(matched.request_index, 1); // Matched zh-Hant
/// assert_eq!(matched.candidate_index, 1); // Matched zh-Hant-TW
/// assert_eq!(matched.candidate.to_string(), "zh-Hant-TW");
/// assert_eq!(matched.quality, MatchQuality::LanguageScript);
/// ```
#[must_use]
pub fn match_languages<'a, R, C>(requests: &[R], available: &'a [C]) -> Option<MatchResult<'a, C>>
where
    R: LanguageMatch<C>,
{
    if requests.is_empty() || available.is_empty() {
        return None;
    }

    for (req_idx, request) in requests.iter().enumerate() {
        let mut best: Option<(usize, &'a C, MatchQuality)> = None;

        for (cand_idx, candidate) in available.iter().enumerate() {
            let quality = request.match_quality(candidate);
            if quality > MatchQuality::NoMatch {
                match best {
                    Some((_, _, best_quality)) => {
                        if quality > best_quality {
                            best = Some((cand_idx, candidate, quality));
                        }
                    }
                    None => {
                        best = Some((cand_idx, candidate, quality));
                    }
                }
            }
        }

        if let Some((cand_idx, candidate, quality)) = best {
            return Some(MatchResult {
                request_index: req_idx,
                candidate_index: cand_idx,
                candidate,
                quality,
            });
        }
    }

    None
}

/// Structural fallback iterator stripping complete components.
///
/// Designed for external resource key lookups, independent of `match_language(s)`; does not implement RFC 4647 Lookup.
///
/// Generates fallback views lazily without heap allocation:
/// 1. Original full tag (including extensions and private use);
/// 2. Stripping all extensions (u/t/other) and private use;
/// 3. Stripping variants;
/// 4. Stripping region;
/// 5. Stripping script, leaving only primary language.
#[derive(Debug, Clone)]
pub struct StructuralFallbackIter<'a> {
    view: NormalTagView<'a>,
    stage: FallbackStage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FallbackStage {
    Exact,
    StripExtensions,
    StripVariants,
    StripRegion,
    StripScript,
    Done,
}

impl<'a> StructuralFallbackIter<'a> {
    /// Constructs a structural fallback iterator.
    #[must_use]
    #[inline]
    pub const fn new(view: NormalTagView<'a>) -> Self {
        Self {
            view,
            stage: FallbackStage::Exact,
        }
    }
}

impl<'a> Iterator for StructuralFallbackIter<'a> {
    type Item = NormalTagView<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.stage {
                FallbackStage::Exact => {
                    self.stage = FallbackStage::StripExtensions;
                    return Some(self.view);
                }
                FallbackStage::StripExtensions => {
                    self.stage = FallbackStage::StripVariants;
                    if self.view.has_other_extensions()
                        || self.view.u().is_some()
                        || self.view.t().is_some()
                        || self.view.private_use().is_some()
                    {
                        return Some(NormalTagView::from_raw_parts(
                            self.view.language_str(),
                            self.view.script(),
                            self.view.region(),
                            self.view.variants(),
                        ));
                    }
                }
                FallbackStage::StripVariants => {
                    self.stage = FallbackStage::StripRegion;
                    if self.view.variants().is_some() {
                        return Some(NormalTagView::from_raw_parts(
                            self.view.language_str(),
                            self.view.script(),
                            self.view.region(),
                            None,
                        ));
                    }
                }
                FallbackStage::StripRegion => {
                    self.stage = FallbackStage::StripScript;
                    if self.view.region().is_some() {
                        return Some(NormalTagView::from_raw_parts(
                            self.view.language_str(),
                            self.view.script(),
                            None,
                            None,
                        ));
                    }
                }
                FallbackStage::StripScript => {
                    self.stage = FallbackStage::Done;
                    if self.view.script().is_some() {
                        return Some(NormalTagView::from_raw_parts(
                            self.view.language_str(),
                            None,
                            None,
                            None,
                        ));
                    }
                }
                FallbackStage::Done => return None,
            }
        }
    }
}
