//! `Tag`, `NormalTag`, `PrivateUseTag`, and parsing with an allocation-free cursor.

use alloc::collections::BTreeSet;
use alloc::vec::Vec;
use core::cmp::Ordering;
use core::fmt::{Display, Formatter};
use core::str::FromStr;

use crate::error::{BuildError, ParseError};
use crate::subtag::{Extension, Language, Region, Script, Subtag, Variant};

/// Threshold for switching variant deduplication from a linear scan to a set.
const VARIANT_BTREE_THRESHOLD: usize = 16;

/// A language tag in the supported RFC 5646 subset: normal or private-use-only.
///
/// Values have normalized casing and extension order; [`Display`] emits that form.
/// Registry validity and extension-specific semantics are not checked.
/// Ordering places all normal tags before private-use-only tags, then uses each
/// variant's structural ordering. It is not the ordering of displayed strings.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Tag {
    /// A tag with a primary language and optional script, region, variants,
    /// extensions, and trailing private use.
    Normal(NormalTag),
    /// A private-use-only tag of the form `x-...`.
    PrivateUse(PrivateUseTag),
}

impl Tag {
    /// Parses either a normal tag or a private-use-only tag.
    ///
    /// Accepts everything supported by [`NormalTag::parse`] plus private-use-only tags;
    /// normal input produces equivalent values through both entry points.
    ///
    /// # Examples
    ///
    /// ```
    /// use fy_i18n_lang_tag::Tag;
    ///
    /// let tag = Tag::parse("x-pig-latin")?;
    /// assert!(tag.is_private_use());
    /// # Ok::<(), fy_i18n_lang_tag::ParseError>(())
    /// ```
    pub fn parse(input: &str) -> Result<Tag, ParseError> {
        check_lexical_structure(input)?;
        let mut cursor = SubtagCursor::new(input);
        let first = cursor.peek().ok_or(ParseError::Empty)?;

        if first.1.len() == 1 && first.1.eq_ignore_ascii_case("x") {
            cursor.next(); // Consume the x prefix.
            let subtags = parse_private_use_subtags(&mut cursor, first.0)?;
            Ok(Tag::PrivateUse(PrivateUseTag { subtags }))
        } else {
            Ok(Tag::Normal(parse_normal_with_cursor(&mut cursor)?))
        }
    }

    /// Returns a reference to the normal tag, if present.
    pub const fn as_normal(&self) -> Option<&NormalTag> {
        match self {
            Self::Normal(tag) => Some(tag),
            Self::PrivateUse(_) => None,
        }
    }

    /// Returns a reference to the private-use-only tag, if present.
    pub const fn as_private_use(&self) -> Option<&PrivateUseTag> {
        match self {
            Self::PrivateUse(tag) => Some(tag),
            Self::Normal(_) => None,
        }
    }

    /// Returns whether this is a normal tag.
    pub const fn is_normal(&self) -> bool {
        matches!(self, Self::Normal(_))
    }

    /// Returns whether this is a private-use-only tag.
    pub const fn is_private_use(&self) -> bool {
        matches!(self, Self::PrivateUse(_))
    }

    /// Returns the language, or `None` for private-use-only tags.
    pub const fn language(&self) -> Option<&Language> {
        match self {
            Self::Normal(tag) => Some(tag.language()),
            Self::PrivateUse(_) => None,
        }
    }

    /// Returns the script, or `None` if absent or private-use-only.
    pub const fn script(&self) -> Option<&Script> {
        match self {
            Self::Normal(tag) => tag.script(),
            Self::PrivateUse(_) => None,
        }
    }

    /// Returns the region, or `None` if absent or private-use-only.
    pub const fn region(&self) -> Option<&Region> {
        match self {
            Self::Normal(tag) => tag.region(),
            Self::PrivateUse(_) => None,
        }
    }

    /// Returns the variants, or an empty slice for private-use-only tags.
    pub fn variants(&self) -> &[Variant] {
        match self {
            Self::Normal(tag) => tag.variants(),
            Self::PrivateUse(_) => &[],
        }
    }

    /// Returns the extensions, or an empty slice for private-use-only tags.
    pub fn extensions(&self) -> &[Extension] {
        match self {
            Self::Normal(tag) => tag.extensions(),
            Self::PrivateUse(_) => &[],
        }
    }

    /// Returns the private-use subtags.
    ///
    /// Returns the trailing private-use payload for normal tags, or the entire
    /// payload for private-use-only tags.
    pub fn private_use(&self) -> &[Subtag] {
        match self {
            Self::Normal(tag) => tag.private_use(),
            Self::PrivateUse(tag) => tag.subtags(),
        }
    }
}

impl Display for Tag {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Normal(tag) => Display::fmt(tag, formatter),
            Self::PrivateUse(tag) => Display::fmt(tag, formatter),
        }
    }
}

impl FromStr for Tag {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self, ParseError> {
        Self::parse(input)
    }
}

impl TryFrom<&str> for Tag {
    type Error = ParseError;

    fn try_from(input: &str) -> Result<Self, ParseError> {
        Self::parse(input)
    }
}

impl From<NormalTag> for Tag {
    fn from(tag: NormalTag) -> Self {
        Self::Normal(tag)
    }
}

impl From<PrivateUseTag> for Tag {
    fn from(tag: PrivateUseTag) -> Self {
        Self::PrivateUse(tag)
    }
}

impl TryFrom<Tag> for NormalTag {
    type Error = Tag;

    fn try_from(tag: Tag) -> Result<Self, Self::Error> {
        match tag {
            Tag::Normal(tag) => Ok(tag),
            other => Err(other),
        }
    }
}

impl TryFrom<Tag> for PrivateUseTag {
    type Error = Tag;

    fn try_from(tag: Tag) -> Result<Self, Self::Error> {
        match tag {
            Tag::PrivateUse(tag) => Ok(tag),
            other => Err(other),
        }
    }
}

impl PartialOrd for Tag {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Tag {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Normal(a), Self::Normal(b)) => a.cmp(b),
            (Self::PrivateUse(a), Self::PrivateUse(b)) => a.cmp(b),
            (Self::Normal(_), Self::PrivateUse(_)) => Ordering::Less,
            (Self::PrivateUse(_), Self::Normal(_)) => Ordering::Greater,
        }
    }
}

/// A tag with a primary language and optional script, region, variants,
/// extensions, and trailing private use.
///
/// Component values satisfy this crate's syntax and normalization rules.
/// Registry validity and extension-specific semantics are not checked.
/// Ordering compares language, script, region, variants, extensions, and private
/// use in that order. Optional values put `None` before `Some`; sequences compare
/// lexicographically by element. This differs from displayed-string ordering:
/// `en-US` sorts before `en-Latn`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NormalTag {
    language: Language,
    script: Option<Script>,
    region: Option<Region>,
    variants: Vec<Variant>,
    extensions: Vec<Extension>,
    private_use: Vec<Subtag>,
}

impl NormalTag {
    /// Parses and normalizes a normal tag; private-use-only input is rejected.
    ///
    /// # Examples
    ///
    /// ```
    /// use fy_i18n_lang_tag::NormalTag;
    ///
    /// let tag = NormalTag::parse("ZH-hans-cn")?;
    /// assert_eq!(tag.to_string(), "zh-Hans-CN");
    /// assert_eq!(tag.language().as_str(), "zh");
    /// # Ok::<(), fy_i18n_lang_tag::ParseError>(())
    /// ```
    pub fn parse(input: &str) -> Result<NormalTag, ParseError> {
        check_lexical_structure(input)?;
        let mut cursor = SubtagCursor::new(input);
        parse_normal_with_cursor(&mut cursor)
    }

    /// Constructs a `NormalTag` from validated components.
    ///
    /// Extensions are sorted lexicographically by singleton. Variant and private-use
    /// subtag order is preserved. Missing script and region values remain `None`.
    ///
    /// # Errors
    ///
    /// Returns [`BuildError::DuplicateVariant`] for duplicate variants or
    /// [`BuildError::DuplicateExtensionSingleton`] for duplicate extension singletons.
    pub fn from_parts(
        language: Language,
        script: Option<Script>,
        region: Option<Region>,
        variants: Vec<Variant>,
        mut extensions: Vec<Extension>,
        private_use: Vec<Subtag>,
    ) -> Result<Self, BuildError> {
        // Use linear scans for short sequences and a BTreeSet for longer ones.
        if variants.len() <= VARIANT_BTREE_THRESHOLD {
            for (i, v1) in variants.iter().enumerate() {
                if variants[i + 1..].contains(v1) {
                    return Err(BuildError::DuplicateVariant);
                }
            }
        } else {
            let mut set = BTreeSet::new();
            for v in &variants {
                if !set.insert(*v) {
                    return Err(BuildError::DuplicateVariant);
                }
            }
        }
        // Reject duplicate extension singletons.
        for (i, e1) in extensions.iter().enumerate() {
            if extensions[i + 1..]
                .iter()
                .any(|e2| e1.singleton() == e2.singleton())
            {
                return Err(BuildError::DuplicateExtensionSingleton {
                    singleton: e1.singleton(),
                });
            }
        }
        extensions.sort_unstable_by_key(|e| e.singleton());

        Ok(Self {
            language,
            script,
            region,
            variants,
            extensions,
            private_use,
        })
    }

    /// Consumes the tag and returns its language, script, region, variants,
    /// extensions, and private-use subtags, in that order.
    #[allow(clippy::type_complexity)]
    pub fn into_parts(
        self,
    ) -> (
        Language,
        Option<Script>,
        Option<Region>,
        Vec<Variant>,
        Vec<Extension>,
        Vec<Subtag>,
    ) {
        (
            self.language,
            self.script,
            self.region,
            self.variants,
            self.extensions,
            self.private_use,
        )
    }

    /// Returns the required language in lowercase.
    pub const fn language(&self) -> &Language {
        &self.language
    }

    /// Returns the script, or `None` if not provided.
    pub const fn script(&self) -> Option<&Script> {
        self.script.as_ref()
    }

    /// Returns the region, or `None` if not provided.
    pub const fn region(&self) -> Option<&Region> {
        self.region.as_ref()
    }

    /// Returns lowercase variants in input order.
    pub fn variants(&self) -> &[Variant] {
        &self.variants
    }

    /// Returns extensions sorted lexicographically by their unique singletons.
    pub fn extensions(&self) -> &[Extension] {
        &self.extensions
    }

    /// Returns trailing private-use subtags (`-x-...`), or an empty slice if absent.
    pub fn private_use(&self) -> &[Subtag] {
        &self.private_use
    }

    /// Replaces the language.
    #[must_use]
    pub fn with_language(mut self, language: Language) -> Self {
        self.language = language;
        self
    }

    /// Replaces or removes the script.
    #[must_use]
    pub fn with_script(mut self, script: Option<Script>) -> Self {
        self.script = script;
        self
    }

    /// Removes the script.
    #[must_use]
    pub fn without_script(mut self) -> Self {
        self.script = None;
        self
    }

    /// Replaces or removes the region.
    #[must_use]
    pub fn with_region(mut self, region: Option<Region>) -> Self {
        self.region = region;
        self
    }

    /// Removes the region.
    #[must_use]
    pub fn without_region(mut self) -> Self {
        self.region = None;
        self
    }

    /// Removes all variants.
    #[must_use]
    pub fn without_variants(mut self) -> Self {
        self.variants.clear();
        self
    }

    /// Removes all extensions.
    #[must_use]
    pub fn without_extensions(mut self) -> Self {
        self.extensions.clear();
        self
    }

    /// Removes the trailing private-use sequence.
    #[must_use]
    pub fn without_private_use(mut self) -> Self {
        self.private_use.clear();
        self
    }
}

impl Display for NormalTag {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(self.language.as_str())?;
        if let Some(script) = self.script {
            write!(formatter, "-{script}")?;
        }
        if let Some(region) = self.region {
            write!(formatter, "-{region}")?;
        }
        for variant in &self.variants {
            write!(formatter, "-{variant}")?;
        }
        for extension in &self.extensions {
            write!(formatter, "-{extension}")?;
        }
        if !self.private_use.is_empty() {
            formatter.write_str("-x")?;
            for subtag in &self.private_use {
                write!(formatter, "-{subtag}")?;
            }
        }
        Ok(())
    }
}

impl FromStr for NormalTag {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self, ParseError> {
        Self::parse(input)
    }
}

impl TryFrom<&str> for NormalTag {
    type Error = ParseError;

    fn try_from(input: &str) -> Result<Self, ParseError> {
        Self::parse(input)
    }
}

impl PartialOrd for NormalTag {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NormalTag {
    fn cmp(&self, other: &Self) -> Ordering {
        self.language
            .cmp(&other.language)
            .then_with(|| self.script.cmp(&other.script))
            .then_with(|| self.region.cmp(&other.region))
            .then_with(|| self.variants.cmp(&other.variants))
            .then_with(|| self.extensions.cmp(&other.extensions))
            .then_with(|| self.private_use.cmp(&other.private_use))
    }
}

/// A private-use-only tag containing subtags after the `x` prefix.
/// Ordering compares the subtag sequence lexicographically.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PrivateUseTag {
    subtags: Vec<Subtag>,
}

impl PrivateUseTag {
    /// Parses and normalizes a private-use-only tag (`x-...`).
    /// After lexical validation, a non-`x` first subtag returns
    /// [`ParseError::MissingPrivateUsePrefix`].
    pub fn parse(input: &str) -> Result<Self, ParseError> {
        check_lexical_structure(input)?;
        let mut cursor = SubtagCursor::new(input);
        let first = cursor.next().ok_or(ParseError::Empty)?;
        if first.1.len() != 1 || !first.1.eq_ignore_ascii_case("x") {
            return Err(ParseError::MissingPrivateUsePrefix { position: first.0 });
        }
        let subtags = parse_private_use_subtags(&mut cursor, first.0)?;
        Ok(Self { subtags })
    }

    /// Constructs a private-use-only tag from a nonempty subtag sequence.
    ///
    /// Preserves subtag order. Returns [`BuildError::EmptyPrivateUse`]
    /// if the sequence is empty.
    pub fn new(subtags: Vec<Subtag>) -> Result<Self, BuildError> {
        if subtags.is_empty() {
            return Err(BuildError::EmptyPrivateUse);
        }
        Ok(Self { subtags })
    }

    /// Returns the lowercase private-use subtags.
    pub fn subtags(&self) -> &[Subtag] {
        &self.subtags
    }
}

impl FromStr for PrivateUseTag {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self, ParseError> {
        Self::parse(input)
    }
}

impl TryFrom<&str> for PrivateUseTag {
    type Error = ParseError;

    fn try_from(input: &str) -> Result<Self, ParseError> {
        Self::parse(input)
    }
}

impl Display for PrivateUseTag {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("x")?;
        for subtag in &self.subtags {
            write!(formatter, "-{subtag}")?;
        }
        Ok(())
    }
}

impl PartialOrd for PrivateUseTag {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PrivateUseTag {
    fn cmp(&self, other: &Self) -> Ordering {
        self.subtags.cmp(&other.subtags)
    }
}

/// Checks the character set, nonempty subtags, and per-subtag length (at most 8).
pub(crate) const fn check_lexical_structure(input: &str) -> Result<(), ParseError> {
    if input.is_empty() {
        return Err(ParseError::Empty);
    }
    let bytes = input.as_bytes();
    let len = bytes.len();

    let mut part_start = 0;
    let mut i = 0;
    while i < len {
        let b = bytes[i];
        if b == b'-' {
            let part_len = i - part_start;
            if part_len == 0 {
                return Err(ParseError::EmptySubtag {
                    position: part_start,
                });
            }
            if part_len > 8 {
                return Err(ParseError::SubtagTooLong {
                    position: part_start,
                    length: part_len,
                });
            }
            part_start = i + 1;
        } else if !b.is_ascii_alphanumeric() {
            return Err(ParseError::InvalidCharacter { position: i });
        }
        i += 1;
    }

    let last_part_len = len - part_start;
    if last_part_len == 0 {
        return Err(ParseError::EmptySubtag {
            position: part_start,
        });
    }
    if last_part_len > 8 {
        return Err(ParseError::SubtagTooLong {
            position: part_start,
            length: last_part_len,
        });
    }

    Ok(())
}

/// An allocation-free cursor over input validated as nonempty subtags of at most 8 bytes.
struct SubtagCursor<'a> {
    input: &'a str,
    position: usize,
}

impl<'a> SubtagCursor<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, position: 0 }
    }

    /// Peeks at the next subtag and its byte offset without advancing.
    fn peek(&self) -> Option<(usize, &'a str)> {
        if self.position > self.input.len() {
            return None;
        }
        let rest = &self.input[self.position..];
        let len = rest.bytes().position(|b| b == b'-').unwrap_or(rest.len());
        Some((self.position, &rest[..len]))
    }

    /// Consumes the next subtag and returns it with its byte offset.
    fn next(&mut self) -> Option<(usize, &'a str)> {
        if self.position > self.input.len() {
            return None;
        }
        let rest = &self.input[self.position..];
        let len = rest.bytes().position(|b| b == b'-').unwrap_or(rest.len());
        let current_pos = self.position;
        let part = &rest[..len];
        self.position += len + 1;
        Some((current_pos, part))
    }
}

/// Collects private-use subtags, requiring at least one after the x prefix.
fn parse_private_use_subtags(
    cursor: &mut SubtagCursor<'_>,
    x_pos: usize,
) -> Result<Vec<Subtag>, ParseError> {
    let mut subtags = Vec::new();
    while let Some((_, part)) = cursor.next() {
        let subtag = Subtag::parse_bytes(part.as_bytes())
            .unwrap_or_else(|| unreachable!("validated as 1..=8 ASCII alphanumerics"));
        subtags.push(subtag);
    }
    if subtags.is_empty() {
        return Err(ParseError::InvalidPrivateUse { position: x_pos });
    }
    Ok(subtags)
}

/// Parses a normal tag using an allocation-free cursor.
fn parse_normal_with_cursor(cursor: &mut SubtagCursor<'_>) -> Result<NormalTag, ParseError> {
    // 1. Required language.
    let (lang_pos, lang_part) = cursor.next().ok_or(ParseError::Empty)?;
    let language = Language::parse_bytes(lang_part.as_bytes())
        .ok_or(ParseError::InvalidLanguage { position: lang_pos })?;

    // 2. Optional script.
    let mut script = None;
    if let Some((pos, part)) = cursor.peek()
        && part.len() == 4
    {
        let bytes = part.as_bytes();
        let all_alpha = bytes.iter().all(u8::is_ascii_alphabetic);
        let first_digit = bytes[0].is_ascii_digit();
        if all_alpha {
            cursor.next();
            script = Some(
                Script::parse_bytes(bytes)
                    .unwrap_or_else(|| unreachable!("validated as four ASCII letters")),
            );
        } else if !first_digit {
            // Four characters, neither alphabetic nor digit-leading: invalid in this slot.
            return Err(ParseError::InvalidScript { position: pos });
        }
        // A four-character digit-leading subtag is a variant; skip the script slot.
    }

    // 3. Optional region.
    let mut region = None;
    if let Some((pos, part)) = cursor.peek() {
        let len = part.len();
        if len == 2 || len == 3 {
            let bytes = part.as_bytes();
            let all_alpha = bytes.iter().all(u8::is_ascii_alphabetic);
            let all_digit = bytes.iter().all(u8::is_ascii_digit);
            if (len == 2 && all_alpha) || (len == 3 && all_digit) {
                cursor.next();
                region = Some(
                    Region::parse_bytes(bytes)
                        .unwrap_or_else(|| unreachable!("validated as a region subtag")),
                );
            } else if len == 2 && !all_alpha {
                // Two characters containing a digit.
                return Err(ParseError::InvalidRegion { position: pos });
            } else if len == 3 && !all_digit && !all_alpha {
                // Three characters mixing letters and digits.
                return Err(ParseError::InvalidRegion { position: pos });
            } else if len == 3 && all_alpha {
                // Three letters have the unsupported extlang shape.
                return Err(ParseError::UnexpectedSubtag { position: pos });
            }
        }
    }

    // 4. Variants.
    let mut variants = Vec::new();
    let mut variant_set: Option<BTreeSet<Variant>> = None;
    while let Some((pos, part)) = cursor.peek() {
        if let Some(variant) = Variant::parse_bytes(part.as_bytes()) {
            cursor.next();
            if let Some(ref mut set) = variant_set {
                if !set.insert(variant) {
                    return Err(ParseError::DuplicateVariant { position: pos });
                }
            } else if variants.contains(&variant) {
                return Err(ParseError::DuplicateVariant { position: pos });
            } else if variants.len() >= VARIANT_BTREE_THRESHOLD {
                let mut set = BTreeSet::new();
                for v in &variants {
                    set.insert(*v);
                }
                set.insert(variant);
                variant_set = Some(set);
            }
            variants.push(variant);
        } else {
            break;
        }
    }

    // 5. Extensions.
    let mut extensions = Vec::new();
    while let Some((pos, part)) = cursor.peek() {
        if part.len() != 1 || part.eq_ignore_ascii_case("x") {
            break;
        }
        let singleton_byte = part.as_bytes()[0].to_ascii_lowercase();
        let singleton_char = char::from(singleton_byte);
        if extensions
            .iter()
            .any(|e: &Extension| e.singleton() == singleton_char)
        {
            return Err(ParseError::InvalidExtension { position: pos });
        }
        cursor.next(); // Consume the singleton.
        let mut subtags = Vec::new();
        while let Some((_, sub_part)) = cursor.peek() {
            if sub_part.len() >= 2 {
                cursor.next();
                let subtag = Subtag::parse_bytes(sub_part.as_bytes())
                    .unwrap_or_else(|| unreachable!("cursor guarantees 2..=8 characters"));
                subtags.push(subtag);
            } else {
                break;
            }
        }
        if subtags.is_empty() {
            return Err(ParseError::InvalidExtension { position: pos });
        }
        extensions.push(Extension::new_unchecked(singleton_byte, subtags));
    }
    extensions.sort_unstable_by_key(|e| e.singleton());

    // 6. Trailing private use.
    let mut private_use = Vec::new();
    if let Some((pos, part)) = cursor.peek()
        && part.eq_ignore_ascii_case("x")
    {
        cursor.next(); // Consume x.
        private_use = parse_private_use_subtags(cursor, pos)?;
    }

    // 7. All grammar slots are closed; any remaining subtag is unexpected.
    if let Some((pos, _)) = cursor.peek() {
        return Err(ParseError::UnexpectedSubtag { position: pos });
    }

    Ok(NormalTag {
        language,
        script,
        region,
        variants,
        extensions,
        private_use,
    })
}

#[cfg(feature = "serde")]
mod serde_impl {
    //! Serialize normalized strings; validate syntax and normalize on deserialization.

    use super::{NormalTag, PrivateUseTag, Tag};
    use core::fmt;
    use serde::de::{self, Visitor};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    macro_rules! impl_serde {
        ($ty:ident) => {
            impl Serialize for $ty {
                fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                    serializer.collect_str(self)
                }
            }

            impl<'de> Deserialize<'de> for $ty {
                fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                    struct TagVisitor;

                    impl<'de> Visitor<'de> for TagVisitor {
                        type Value = $ty;

                        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                            formatter.write_str("a modern BCP 47 language-tag string")
                        }

                        fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                            $ty::parse(v).map_err(E::custom)
                        }
                    }

                    deserializer.deserialize_str(TagVisitor)
                }
            }
        };
    }

    impl_serde!(Tag);
    impl_serde!(NormalTag);
    impl_serde!(PrivateUseTag);
}
