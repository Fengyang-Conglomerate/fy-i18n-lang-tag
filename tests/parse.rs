//! Integration tests for parsing, normalization, rejection, transformations, and Serde.

#[test]
fn error_messages_are_english_and_preserve_context() {
    let cases = [
        (ParseError::Empty, "input is empty"),
        (
            ParseError::InvalidCharacter { position: 7 },
            "invalid character at byte 7",
        ),
        (
            ParseError::EmptySubtag { position: 7 },
            "empty subtag at byte 7: consecutive, leading, or trailing hyphens",
        ),
        (
            ParseError::SubtagTooLong {
                position: 7,
                length: 9,
            },
            "subtag at byte 7 is 9 bytes long; maximum is 8",
        ),
        (
            ParseError::InvalidLanguage { position: 7 },
            "invalid language subtag at byte 7: expected 2-3 or 5-8 ASCII letters",
        ),
        (
            ParseError::InvalidScript { position: 7 },
            "invalid script subtag at byte 7: expected 4 ASCII letters",
        ),
        (
            ParseError::InvalidRegion { position: 7 },
            "invalid region subtag at byte 7: expected 2 ASCII letters or 3 ASCII digits",
        ),
        (
            ParseError::InvalidVariant { position: 7 },
            "invalid variant subtag at byte 7: expected 5-8 ASCII alphanumerics or 4 starting with a digit",
        ),
        (
            ParseError::DuplicateVariant { position: 7 },
            "duplicate variant subtag at byte 7",
        ),
        (
            ParseError::InvalidExtension { position: 7 },
            "invalid extension at byte 7: invalid or duplicate singleton, or missing or invalid subtags",
        ),
        (
            ParseError::InvalidPrivateUse { position: 7 },
            "empty private-use sequence at byte 7: expected at least one subtag",
        ),
        (
            ParseError::UnexpectedSubtag { position: 7 },
            "unexpected subtag at byte 7: not allowed here by the supported syntax",
        ),
        (
            ParseError::MissingPrivateUsePrefix { position: 7 },
            "missing private-use prefix at byte 7: expected 'x'",
        ),
    ];
    for (error, expected) in cases {
        assert_eq!(
            error.position(),
            if error == ParseError::Empty { 0 } else { 7 }
        );
        assert_eq!(error.to_string(), expected);
    }
    assert_eq!(
        BuildError::DuplicateVariant.to_string(),
        "cannot build tag: duplicate variant subtag"
    );
    assert_eq!(
        BuildError::DuplicateExtensionSingleton { singleton: 'u' }.to_string(),
        "cannot build tag: duplicate extension singleton 'u'"
    );
}

#[cfg(feature = "serde")]
#[test]
fn serde_errors_are_english() {
    let error = serde_json::from_str::<Tag>("\"zh_CN\"")
        .unwrap_err()
        .to_string();
    assert!(error.contains("invalid character at byte 2"));
    let error = serde_json::from_str::<Tag>("42").unwrap_err().to_string();
    assert!(error.contains("a modern BCP 47 language-tag string"));
    let error = serde_json::from_str::<Language>("42")
        .unwrap_err()
        .to_string();
    assert!(error.contains("a BCP 47 subtag string"));
    let error = serde_json::from_str::<Extension>("42")
        .unwrap_err()
        .to_string();
    assert!(error.contains("a BCP 47 extension string"));
}

use core::hash::{Hash, Hasher};
use core::str::FromStr;
use std::hash::DefaultHasher;

use fy_i18n_lang_tag::{
    BuildError, Extension, Language, NormalTag, ParseError, PrivateUseTag, Region, Script, Subtag,
    Tag, Variant,
};

/// Successful parsing, decomposition, and accessors.
#[test]
fn parse_normal_structure() {
    let tag = NormalTag::parse("zh").unwrap();
    assert_eq!(tag.language().as_str(), "zh");
    assert_eq!(tag.script(), None);
    assert_eq!(tag.region(), None);
    assert!(tag.variants().is_empty());
    assert!(tag.extensions().is_empty());
    assert!(tag.private_use().is_empty());

    let tag = NormalTag::parse("zh-Hans-CN-variant-u-ca-buddhist-x-note").unwrap();
    assert_eq!(tag.language().as_str(), "zh");
    assert_eq!(tag.script().unwrap().as_str(), "Hans");
    assert_eq!(tag.region().unwrap().as_str(), "CN");
    assert_eq!(tag.variants().len(), 1);
    assert_eq!(tag.variants()[0].as_str(), "variant");
    assert_eq!(tag.extensions().len(), 1);
    assert_eq!(tag.extensions()[0].singleton(), 'u');
    assert_eq!(
        tag.extensions()[0]
            .subtags()
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>(),
        ["ca", "buddhist"]
    );
    assert_eq!(
        tag.private_use()
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>(),
        ["note"]
    );

    // Accessors return references that can be retained without borrowing temporaries.
    let lang_ref: &str = tag.language().as_str();
    assert_eq!(lang_ref, "zh");
    let script_ref: &str = tag.script().unwrap().as_str();
    assert_eq!(script_ref, "Hans");
    let region_ref: &str = tag.region().unwrap().as_str();
    assert_eq!(region_ref, "CN");
}

/// Supported language lengths and numeric regions.
#[test]
fn parse_language_shapes() {
    for input in ["zh", "yue", "abcde", "abcdefgh", "en"] {
        NormalTag::parse(input)
            .unwrap_or_else(|e| panic!("{input} should parse successfully: {e}"));
    }
    // Numeric region (UN M.49).
    let tag = NormalTag::parse("es-419").unwrap();
    assert_eq!(tag.region().unwrap().as_str(), "419");
    // A 5-8 letter language with script and region.
    let tag = NormalTag::parse("abcdefgh-Hans-CN").unwrap();
    assert_eq!(tag.language().as_str(), "abcdefgh");
}

/// Short digit-leading and regular variants, including Display order.
#[test]
fn parse_variants() {
    let tag = NormalTag::parse("en-1996-POSIX").unwrap();
    assert_eq!(
        tag.variants()
            .iter()
            .map(|v| v.as_str())
            .collect::<Vec<_>>(),
        ["1996", "posix"]
    );
    assert_eq!(tag.to_string(), "en-1996-posix");
}

/// Private-use-only tags and trailing private use.
#[test]
fn parse_private_use() {
    let tag = Tag::parse("x-pig-latin").unwrap();
    let Tag::PrivateUse(private) = &tag else {
        panic!("expected a private-use-only tag")
    };
    assert_eq!(
        private
            .subtags()
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>(),
        ["pig", "latin"]
    );
    assert_eq!(tag.to_string(), "x-pig-latin");

    // Trailing private use belongs to a normal tag.
    let tag = NormalTag::parse("en-US-x-twain").unwrap();
    assert_eq!(tag.language().as_str(), "en");
    assert_eq!(
        tag.private_use()
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>(),
        ["twain"]
    );

    // PrivateUseTag::new and parse produce equivalent values.
    let custom = PrivateUseTag::new(vec![Subtag::parse("abc").unwrap()]).unwrap();
    assert_eq!(custom.to_string(), "x-abc");
    assert_eq!(
        PrivateUseTag::new(vec![]).unwrap_err(),
        BuildError::EmptyPrivateUse
    );
    let parsed_p = PrivateUseTag::parse("x-custom-demo").unwrap();
    assert_eq!(parsed_p.to_string(), "x-custom-demo");
    let from_str_p: PrivateUseTag = "x-custom-demo".parse().unwrap();
    assert_eq!(parsed_p, from_str_p);
}

/// Casing is normalized during parsing.
#[test]
fn canonicalize_casing() {
    assert_eq!(NormalTag::parse("ZH").unwrap().to_string(), "zh");
    assert_eq!(
        NormalTag::parse("zh-HANS-cn").unwrap().to_string(),
        "zh-Hans-CN"
    );
    assert_eq!(
        NormalTag::parse("EN-LATN-GB-SCOTLAND").unwrap().to_string(),
        "en-Latn-GB-scotland"
    );
    assert_eq!(
        Tag::parse("X-PIG-LATIN").unwrap().to_string(),
        "x-pig-latin"
    );
    assert_eq!(
        NormalTag::parse("EN-1996-X-TWAIN").unwrap().to_string(),
        "en-1996-x-twain"
    );
}

/// Preserve variant order and recommended prefix dependencies (RFC 5646 sections 4.1/4.5).
#[test]
fn variant_preserves_prefix_order() {
    let tag = NormalTag::parse("sl-rozaj-biske").unwrap();
    assert_eq!(tag.to_string(), "sl-rozaj-biske");
    assert_eq!(tag.variants()[0].as_str(), "rozaj");
    assert_eq!(tag.variants()[1].as_str(), "biske");

    // Different variant orders produce distinct values.
    let tag2 = NormalTag::parse("sl-biske-rozaj").unwrap();
    assert_eq!(tag2.to_string(), "sl-biske-rozaj");
    assert_ne!(tag, tag2);
}

/// Extensions are sorted by singleton, preserving internal subtag order.
#[test]
fn canonicalize_extension_order() {
    // Digit singletons sort before letters.
    let tag = NormalTag::parse("en-b-bbb-a-aaa-0-zzz").unwrap();
    assert_eq!(tag.to_string(), "en-0-zzz-a-aaa-b-bbb");
    assert_eq!(
        tag.extensions()
            .iter()
            .map(Extension::singleton)
            .collect::<Vec<_>>(),
        ['0', 'a', 'b']
    );
    // Relative order within each extension is preserved.
    let tag = NormalTag::parse("en-u-ca-buddhist-nu-latn").unwrap();
    assert_eq!(tag.to_string(), "en-u-ca-buddhist-nu-latn");

    // Multiple extensions with multiple subtags.
    let tag = NormalTag::parse("en-t-foo-u-bar-a-baz").unwrap();
    assert_eq!(tag.to_string(), "en-a-baz-t-foo-u-bar");
}

/// Display and parse round-trip with idempotent normalization.
#[test]
fn roundtrip_idempotent() {
    for input in [
        "zh",
        "zh-Hans",
        "zh-Hans-CN",
        "yue-Hant",
        "sr-Latn-RS",
        "es-419",
        "en-1996-POSIX",
        "en-a-bbb-aaa-x-note",
        "zh-Hans-CN-u-ca-buddhist-t-en-us-hybrid-x-note",
        "x-pig-latin",
        "X-PIG-LATIN",
        "ZH-hAnS-cN",
    ] {
        let tag = Tag::parse(input).unwrap();
        let canonical = tag.to_string();
        let reparsed = Tag::parse(&canonical).unwrap();
        assert_eq!(reparsed, tag, "{input} failed to round-trip");
        assert_eq!(
            reparsed.to_string(),
            canonical,
            "{input} normalization is not idempotent"
        );
    }
}

/// Both entry points accept normal tags; only Tag::parse accepts private-use-only tags.
#[test]
fn entry_point_domains() {
    let normal = NormalTag::parse("zh-Hans-CN").unwrap();
    let Tag::Normal(full) = Tag::parse("zh-Hans-CN").unwrap() else {
        panic!("normal tags should map to Tag::Normal")
    };
    assert_eq!(normal, full);

    // NormalTag rejects the single-letter private-use prefix as InvalidLanguage.
    assert_eq!(
        NormalTag::parse("x-pig-latin"),
        Err(ParseError::InvalidLanguage { position: 0 })
    );
    assert!(Tag::parse("x-pig-latin").is_ok());
}

/// Reject POSIX syntax, unsupported grandfathered forms, invalid characters, and duplicates.
#[test]
fn reject_negative_cases() {
    let cases: &[(&str, ParseError)] = &[
        // POSIX locale separators are invalid characters.
        ("zh_CN", ParseError::InvalidCharacter { position: 2 }),
        ("en_US.UTF-8", ParseError::InvalidCharacter { position: 2 }),
        ("de-CH@euro", ParseError::InvalidCharacter { position: 5 }),
        ("zh CN", ParseError::InvalidCharacter { position: 2 }),
        // Byte offset of an invalid multibyte character.
        ("zh-Hans中", ParseError::InvalidCharacter { position: 7 }),
        // Empty subtags.
        ("zh--CN", ParseError::EmptySubtag { position: 3 }),
        ("-zh", ParseError::EmptySubtag { position: 0 }),
        ("zh-", ParseError::EmptySubtag { position: 3 }),
        // Overlong subtags (nine characters).
        (
            "abcdefghi",
            ParseError::SubtagTooLong {
                position: 0,
                length: 9,
            },
        ),
        (
            "zh-abcdefghi",
            ParseError::SubtagTooLong {
                position: 3,
                length: 9,
            },
        ),
        // Invalid languages: single-letter, reserved four-letter, and numeric forms.
        ("i", ParseError::InvalidLanguage { position: 0 }),
        ("i-klingon", ParseError::InvalidLanguage { position: 0 }),
        ("abcd", ParseError::InvalidLanguage { position: 0 }),
        ("123", ParseError::InvalidLanguage { position: 0 }),
        ("1", ParseError::InvalidLanguage { position: 0 }),
        // Mixed four-character subtag in the script slot.
        ("zh-Han1", ParseError::InvalidScript { position: 3 }),
        // Mixed two- and three-character subtags in the region slot.
        ("zh-4k", ParseError::InvalidRegion { position: 3 }),
        ("zh-4k9", ParseError::InvalidRegion { position: 3 }),
        // Duplicate variants, including mixed casing.
        ("en-1996-1996", ParseError::DuplicateVariant { position: 8 }),
        (
            "en-POSIX-1996-posix",
            ParseError::DuplicateVariant { position: 14 },
        ),
        (
            "en-POSIX-posix",
            ParseError::DuplicateVariant { position: 9 },
        ),
        // Invalid extension prefixes and payloads.
        ("en-u", ParseError::InvalidExtension { position: 3 }),
        ("en-u-x-foo", ParseError::InvalidExtension { position: 3 }),
        ("en-u-u-ca", ParseError::InvalidExtension { position: 3 }),
        ("en-u-ca-U-ca", ParseError::InvalidExtension { position: 8 }),
        // Private-use prefix without a payload.
        ("en-x", ParseError::InvalidPrivateUse { position: 3 }),
        // Context-sensitive rejection of extlang and leftover subtags.
        ("zh-yue-Hant", ParseError::UnexpectedSubtag { position: 3 }),
        ("zh-yue", ParseError::UnexpectedSubtag { position: 3 }),
        ("en-US-GB", ParseError::UnexpectedSubtag { position: 6 }),
        ("en-Hans-Latn", ParseError::UnexpectedSubtag { position: 8 }),
        // Invalid leftovers after the region slot are UnexpectedSubtag.
        ("en-US-12", ParseError::UnexpectedSubtag { position: 6 }),
        ("en-US-a1bc", ParseError::UnexpectedSubtag { position: 6 }),
    ];
    for &(input, ref expected) in cases {
        let actual = Tag::parse(input).unwrap_err();
        assert_eq!(&actual, expected, "unexpected error for {input}: {actual}");
        let actual_normal = NormalTag::parse(input).unwrap_err();
        assert_eq!(
            &actual_normal, expected,
            "unexpected NormalTag error for {input}"
        );
    }

    // Bare "x" is a missing private-use payload for Tag, but an invalid language for NormalTag.
    assert_eq!(
        Tag::parse("x").unwrap_err(),
        ParseError::InvalidPrivateUse { position: 0 }
    );
    assert_eq!(
        NormalTag::parse("x").unwrap_err(),
        ParseError::InvalidLanguage { position: 0 }
    );

    // Empty input.
    assert_eq!(Tag::parse(""), Err(ParseError::Empty));

    // Well-formed tags longer than 255 bytes parse and round-trip.
    let mut long_tag = String::from("en-x");
    for _ in 0..35 {
        long_tag.push_str("-abcdefgh");
    }
    assert!(long_tag.len() > 255);
    let parsed = Tag::parse(&long_tag).unwrap();
    assert_eq!(parsed.to_string(), long_tag);
    let reparsed = Tag::parse(&parsed.to_string()).unwrap();
    assert_eq!(parsed, reparsed);

    // Exactly 255 bytes: en-x-aa plus 124 one-character private-use subtags.
    let mut valid_255 = String::from("en-x-aa");
    for _ in 0..124 {
        valid_255.push_str("-a");
    }
    assert_eq!(valid_255.len(), 255);
    assert!(
        NormalTag::parse(&valid_255).is_ok(),
        "a well-formed 255-byte tag should parse successfully"
    );

    // A 255-byte tag with a trailing extlang-shaped subtag is structurally invalid.
    let mut edge = String::from("abcdefgh");
    for i in 1..28 {
        edge.push('-');
        edge.push_str(&format!("v{i:07}"));
    }
    edge.push_str("-abc");
    assert_eq!(edge.len(), 255);
    assert_eq!(
        NormalTag::parse(&edge).unwrap_err(),
        ParseError::UnexpectedSubtag { position: 252 }
    );
}

/// Grandfathered tags fitting the supported syntax are parsed literally.
#[test]
fn grandfathered_literal_compatibility() {
    let tag = NormalTag::parse("art-lojban").unwrap();
    assert_eq!(tag.language().as_str(), "art");
    assert_eq!(tag.variants()[0].as_str(), "lojban");
    let tag = NormalTag::parse("cel-gaulish").unwrap();
    assert_eq!(tag.language().as_str(), "cel");
    assert_eq!(tag.variants()[0].as_str(), "gaulish");
}

/// FromStr and TryFrom<&str> delegate to parsing.
#[test]
fn from_str_and_try_from() {
    let tag: NormalTag = "zh-Hans".parse().unwrap();
    assert_eq!(tag.to_string(), "zh-Hans");
    assert_eq!(tag.language().as_str(), "zh");

    let tag = <Tag as TryFrom<&str>>::try_from("x-a").unwrap();
    assert!(tag.is_private_use());
    let tag = <NormalTag as TryFrom<&str>>::try_from("zh").unwrap();
    assert_eq!(tag.language().as_str(), "zh");

    assert!(NormalTag::from_str("zh_CN").is_err());
    assert!(<Tag as TryFrom<&str>>::try_from("").is_err());
}

/// Serde uses normalized strings and validates deserialized input.
#[cfg(feature = "serde")]
#[test]
fn serde_roundtrip() {
    let tag = NormalTag::parse("ZH-hans-cn").unwrap();
    let text = serde_json::to_string(&tag).unwrap();
    assert_eq!(text, "\"zh-Hans-CN\"");
    let back: NormalTag = serde_json::from_str(&text).unwrap();
    assert_eq!(back, tag);

    let tag = Tag::parse("x-pig-latin").unwrap();
    assert_eq!(serde_json::to_string(&tag).unwrap(), "\"x-pig-latin\"");
    let back: Tag = serde_json::from_str("\"x-pig-latin\"").unwrap();
    assert_eq!(back, tag);

    let priv_tag = PrivateUseTag::parse("x-pig-latin").unwrap();
    let p_text = serde_json::to_string(&priv_tag).unwrap();
    assert_eq!(p_text, "\"x-pig-latin\"");
    let p_back: PrivateUseTag = serde_json::from_str(&p_text).unwrap();
    assert_eq!(p_back, priv_tag);

    // Serde for atomic subtags and Extension.
    let lang = Language::parse("zh").unwrap();
    let l_text = serde_json::to_string(&lang).unwrap();
    assert_eq!(l_text, "\"zh\"");
    assert_eq!(serde_json::from_str::<Language>(&l_text).unwrap(), lang);

    let ext = Extension::parse("u-ca-buddhist").unwrap();
    let e_text = serde_json::to_string(&ext).unwrap();
    assert_eq!(e_text, "\"u-ca-buddhist\"");
    assert_eq!(serde_json::from_str::<Extension>(&e_text).unwrap(), ext);

    assert!(serde_json::from_str::<NormalTag>("\"zh_CN\"").is_err());
}

/// Inline subtags are Copy; Script occupies four bytes.
#[test]
fn subtag_types_are_copy_and_compact() {
    fn assert_copy<T: Copy + Clone>() {}
    assert_copy::<Language>();
    assert_copy::<Script>();
    assert_copy::<Region>();
    assert_copy::<Variant>();
    assert_copy::<Subtag>();

    assert_eq!(size_of::<Script>(), 4);
}

/// Minimum lengths of extension and private-use subtags.
#[test]
fn extension_and_private_subtag_bounds() {
    assert_eq!(
        NormalTag::parse("en-u-a").unwrap_err(),
        ParseError::InvalidExtension { position: 3 }
    );
    let tag = Tag::parse("x-a").unwrap();
    assert_eq!(tag.as_private_use().unwrap().subtags().len(), 1);
    assert!(NormalTag::parse("en-u-ca").is_ok());
    assert!(NormalTag::parse("en-u-abcdefgh").is_ok());
}

/// Standalone subtag construction, comparison, ordering, and standard traits.
#[test]
fn subtag_standalone_and_traits() {
    let lang = Language::parse("ZH").unwrap();
    assert_eq!(lang, Language::parse("zh").unwrap());
    assert_eq!(lang.as_str(), "zh");

    let script = Script::parse("HANS").unwrap();
    assert_eq!(script, Script::parse("Hans").unwrap());
    assert_eq!(script.as_str(), "Hans");

    let region = Region::parse("cn").unwrap();
    assert_eq!(region, Region::parse("CN").unwrap());
    assert_eq!(region.as_str(), "CN");

    let variant = Variant::parse("1996").unwrap();
    assert_eq!(variant, Variant::parse("1996").unwrap());
    assert_eq!(variant.as_str(), "1996");

    let subtag = Subtag::parse("foo").unwrap();
    assert_eq!(subtag, Subtag::parse("foo").unwrap());
    assert_eq!(subtag.as_str(), "foo");

    // PartialOrd / Ord
    let mut langs = [
        Language::parse("zh").unwrap(),
        Language::parse("en").unwrap(),
        Language::parse("ja").unwrap(),
    ];
    langs.sort();
    assert_eq!(
        langs,
        [
            Language::parse("en").unwrap(),
            Language::parse("ja").unwrap(),
            Language::parse("zh").unwrap(),
        ]
    );

    // Typed collection lookups and literal comparisons.
    let mut set = std::collections::HashSet::new();
    set.insert(lang);
    assert!(set.contains(&lang), "typed set lookup via &Language");
    assert_eq!(lang.as_str(), "zh");

    let mut map = std::collections::HashMap::new();
    map.insert(region, 100);
    assert_eq!(map.get(&region), Some(&100), "typed map lookup via &Region");
    assert_eq!(region.as_str(), "CN");
}

/// Tag convenience methods and conversions.
#[test]
fn tag_ergonomics_and_conversions() {
    let normal = NormalTag::parse("zh-Hans-CN-variant-u-ca-buddhist-x-note").unwrap();
    let tag: Tag = normal.clone().into();
    assert!(tag.is_normal());
    assert!(!tag.is_private_use());
    assert_eq!(tag.language().unwrap().as_str(), "zh");
    assert_eq!(tag.script().unwrap().as_str(), "Hans");
    assert_eq!(tag.region().unwrap().as_str(), "CN");
    assert_eq!(tag.variants().len(), 1);
    assert_eq!(tag.variants()[0].as_str(), "variant");
    assert_eq!(tag.extensions().len(), 1);
    assert_eq!(tag.extensions()[0].singleton(), 'u');
    assert_eq!(tag.private_use().len(), 1);
    assert_eq!(tag.private_use()[0].as_str(), "note");
    assert_eq!(tag.as_normal(), Some(&normal));
    assert_eq!(tag.as_private_use(), None);

    let extracted: NormalTag = tag.try_into().unwrap();
    assert_eq!(extracted, normal);

    let priv_tag = Tag::parse("x-custom-note").unwrap();
    assert!(priv_tag.is_private_use());
    assert_eq!(priv_tag.language(), None);
    assert_eq!(priv_tag.script(), None);
    assert_eq!(priv_tag.region(), None);
    assert!(priv_tag.variants().is_empty());
    assert!(priv_tag.extensions().is_empty());
    assert_eq!(priv_tag.private_use().len(), 2);
    assert_eq!(priv_tag.private_use()[0].as_str(), "custom");
    assert_eq!(priv_tag.private_use()[1].as_str(), "note");
    assert!(NormalTag::try_from(priv_tag).is_err());
}

/// NormalTag component construction and transformations.
#[test]
fn normal_tag_from_parts_and_derivation() {
    let lang = Language::parse("zh").unwrap();
    let region = Region::parse("TW").unwrap();
    let base = NormalTag::from_parts(lang, None, Some(region), vec![], vec![], vec![]).unwrap();
    assert_eq!(base.to_string(), "zh-TW");

    // with_script -> zh-Hant-TW
    let hant = Script::parse("Hant").unwrap();
    let with_script = base.clone().with_script(Some(hant));
    assert_eq!(with_script.to_string(), "zh-Hant-TW");

    // without_region -> zh-Hant
    let without_reg = with_script.clone().without_region();
    assert_eq!(without_reg.to_string(), "zh-Hant");

    // without_script -> zh
    let lang_only = without_reg.without_script();
    assert_eq!(lang_only.to_string(), "zh");

    // with_language -> en
    let en_tag = lang_only.with_language(Language::parse("en").unwrap());
    assert_eq!(en_tag.to_string(), "en");

    // Decompose using into_parts.
    let (l, s, r, v, e, p) = base.into_parts();
    assert_eq!(l.as_str(), "zh");
    assert_eq!(s, None);
    assert_eq!(r.unwrap().as_str(), "TW");
    assert!(v.is_empty());
    assert!(e.is_empty());
    assert!(p.is_empty());

    // from_parts rejects duplicate variants.
    let v1 = Variant::parse("1996").unwrap();
    assert_eq!(
        NormalTag::from_parts(lang, None, None, vec![v1, v1], vec![], vec![]).unwrap_err(),
        BuildError::DuplicateVariant
    );

    // from_parts rejects duplicate extension singletons.
    let ext1 = Extension::new('u', vec![Subtag::parse("ca").unwrap()]).unwrap();
    let ext2 = Extension::new('u', vec![Subtag::parse("nu").unwrap()]).unwrap();
    assert_eq!(
        NormalTag::from_parts(lang, None, None, vec![], vec![ext1, ext2], vec![]).unwrap_err(),
        BuildError::DuplicateExtensionSingleton { singleton: 'u' }
    );
}

/// Hash and Eq are consistent.
#[test]
fn hash_and_eq_consistency() {
    fn hash_val<T: Hash>(item: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        item.hash(&mut hasher);
        hasher.finish()
    }

    let a = NormalTag::parse("zh-Hans-CN").unwrap();
    let b = NormalTag::parse("ZH-HANS-cn").unwrap();
    assert_eq!(a, b);
    assert_eq!(hash_val(&a), hash_val(&b));

    let tag_a: Tag = a.into();
    let tag_b: Tag = b.into();
    assert_eq!(tag_a, tag_b);
    assert_eq!(hash_val(&tag_a), hash_val(&tag_b));
}

/// Adaptive deduplication preserves order, using BTreeSet above 16 variants.
#[test]
fn variant_large_sequence_and_order_preservation() {
    // Construct a tag with 20 distinct variants.
    let mut parts = vec!["zh".to_string()];
    for i in 0..20 {
        parts.push(format!("v{:07}", i));
    }
    let input = parts.join("-");
    let tag = NormalTag::parse(&input).expect("20 valid variants should parse successfully");
    assert_eq!(tag.variants().len(), 20);
    // Preserve input order exactly.
    for (i, v) in tag.variants().iter().enumerate() {
        assert_eq!(v.as_str(), format!("v{:07}", i));
    }

    // Replace the item at index 18 with the item at index 2.
    let mut dup_parts = parts.clone();
    dup_parts[18] = dup_parts[2].clone(); // "v0000001"
    let dup_input = dup_parts.join("-");
    // Compute the exact byte offset of the item at index 18.
    let expected_pos: usize = dup_parts[..18].iter().map(|p| p.len() + 1).sum::<usize>();
    assert_eq!(
        NormalTag::parse(&dup_input).unwrap_err(),
        ParseError::DuplicateVariant {
            position: expected_pos
        }
    );

    // from_parts rejects duplicates in long variant sequences.
    let vars: Vec<Variant> = (0..20)
        .map(|i| Variant::parse(&format!("v{:07}", i)).unwrap())
        .collect();
    let lang = Language::parse("zh").unwrap();
    let from_parts_tag =
        NormalTag::from_parts(lang, None, None, vars.clone(), vec![], vec![]).unwrap();
    assert_eq!(from_parts_tag.variants().len(), 20);

    let mut dup_vars = vars.clone();
    dup_vars[18] = dup_vars[2];
    assert_eq!(
        NormalTag::from_parts(lang, None, None, dup_vars, vec![], vec![]).unwrap_err(),
        BuildError::DuplicateVariant
    );
}

/// Extension::parse returns precise error categories and byte offsets.
#[test]
fn extension_parse_detailed_errors() {
    // InvalidCharacter points to the offending character.
    assert_eq!(
        Extension::parse("u-a_"),
        Err(ParseError::InvalidCharacter { position: 3 })
    );

    // SubtagTooLong includes the offending length.
    assert_eq!(
        Extension::parse("u-abcdefghi"),
        Err(ParseError::SubtagTooLong {
            position: 2,
            length: 9
        })
    );

    // UnexpectedSubtag points to a one-character subtag.
    assert_eq!(
        Extension::parse("u-aa-1"),
        Err(ParseError::UnexpectedSubtag { position: 5 })
    );

    // Missing payload or invalid singleton: InvalidExtension at byte 0.
    assert_eq!(
        Extension::parse("u"),
        Err(ParseError::InvalidExtension { position: 0 })
    );
    assert_eq!(
        Extension::parse("x-aa"),
        Err(ParseError::InvalidExtension { position: 0 })
    );
    assert_eq!(Extension::parse(""), Err(ParseError::Empty));
    assert_eq!(
        Extension::parse("u--aa"),
        Err(ParseError::EmptySubtag { position: 2 })
    );
    // A trailing empty subtag starts at input.len().
    assert_eq!(
        Extension::parse("u-"),
        Err(ParseError::EmptySubtag { position: 2 })
    );
    assert_eq!(
        Extension::parse("u-aa-"),
        Err(ParseError::EmptySubtag { position: 5 })
    );
    // Lexical error precedence matches whole-tag parsing.
    assert_eq!(
        Extension::parse("u-a_-"),
        Err(ParseError::InvalidCharacter { position: 3 })
    );

    // Successful parsing.
    let ext = Extension::parse("u-ca-buddhist").unwrap();
    assert_eq!(ext.singleton(), 'u');
    assert_eq!(ext.singleton_byte(), b'u');
    assert_eq!(ext.to_string(), "u-ca-buddhist");
}

/// Atomic parse and accessors work in const contexts.
#[test]
fn const_evaluation_and_subtag_constants() {
    // Evaluate the same parse interface in const contexts.
    const ZH_RES: Result<Language, ParseError> = Language::parse("zh");
    assert!(ZH_RES.is_ok());

    const ZH: Language = match Language::parse("zh") {
        Ok(l) => l,
        Err(_) => panic!("invalid language"),
    };
    const EN: Language = match Language::parse("en") {
        Ok(l) => l,
        Err(_) => panic!("invalid language"),
    };
    const HANS: Script = match Script::parse("Hans") {
        Ok(s) => s,
        Err(_) => panic!("invalid script"),
    };
    const CN: Region = match Region::parse("CN") {
        Ok(r) => r,
        Err(_) => panic!("invalid region"),
    };
    const POSIX: Variant = match Variant::parse("posix") {
        Ok(v) => v,
        Err(_) => panic!("invalid variant"),
    };
    const CA: Subtag = match Subtag::parse("ca") {
        Ok(s) => s,
        Err(_) => panic!("invalid generic subtag"),
    };

    // Error positions are available in const contexts.
    const ERR_POS: usize = match Language::parse("zh_") {
        Ok(_) => 0,
        Err(e) => e.position(),
    };
    assert_eq!(ERR_POS, 0);

    const ZH_STR: &str = ZH.as_str();
    const HANS_STR: &str = HANS.as_str();
    const CN_STR: &str = CN.as_str();
    const POSIX_STR: &str = POSIX.as_str();
    const CA_STR: &str = CA.as_str();

    assert_eq!(ZH_STR, "zh");
    assert_eq!(EN.as_str(), "en");
    assert_eq!(HANS_STR, "Hans");
    assert_eq!(CN_STR, "CN");
    assert_eq!(POSIX_STR, "posix");
    assert_eq!(CA_STR, "ca");

    assert_eq!(ZH.len(), 2);
    assert_eq!(HANS.len(), 4);
    assert_eq!(CN.len(), 2);
    assert_eq!(POSIX.len(), 5);
    assert_eq!(CA.len(), 2);
}

/// Extension::new validation and failure paths.
#[test]
fn extension_new_validation() {
    let sub = Subtag::parse("buddhist").unwrap();
    let short_sub = Subtag::parse("a").unwrap();

    // Successful construction with lowercase normalization.
    let ext = Extension::new('U', vec![sub]).unwrap();
    assert_eq!(ext.singleton(), 'u');
    assert_eq!(ext.singleton_byte(), b'u');
    assert_eq!(ext.subtags().len(), 1);

    // Reject x and X, reserved for private use.
    assert_eq!(
        Extension::new('x', vec![sub]),
        Err(BuildError::InvalidSingleton { singleton: 'x' })
    );
    assert_eq!(
        Extension::new('X', vec![sub]),
        Err(BuildError::InvalidSingleton { singleton: 'X' })
    );

    // Reject non-ASCII or invalid singletons.
    assert_eq!(
        Extension::new('中', vec![sub]),
        Err(BuildError::InvalidSingleton { singleton: '中' })
    );
    assert_eq!(
        Extension::new('-', vec![sub]),
        Err(BuildError::InvalidSingleton { singleton: '-' })
    );

    // Reject an empty payload.
    assert_eq!(Extension::new('u', vec![]), Err(BuildError::EmptyExtension));

    // Reject one-character payload subtags.
    assert_eq!(
        Extension::new('u', vec![short_sub]),
        Err(BuildError::InvalidExtensionSubtag { index: 0 })
    );
}

/// Atomic subtags reject empty input and distinguish syntax errors.
#[test]
fn subtag_empty_and_fine_grained_errors() {
    assert_eq!(Language::parse(""), Err(ParseError::Empty));
    assert_eq!(Script::parse(""), Err(ParseError::Empty));
    assert_eq!(Region::parse(""), Err(ParseError::Empty));
    assert_eq!(Variant::parse(""), Err(ParseError::Empty));
    assert_eq!(Subtag::parse(""), Err(ParseError::Empty));

    // Invalid variant shapes produce InvalidVariant.
    assert_eq!(
        Variant::parse("abc"),
        Err(ParseError::InvalidVariant { position: 0 })
    );
    assert_eq!(
        Variant::parse("123"),
        Err(ParseError::InvalidVariant { position: 0 })
    );
    assert_eq!(
        Variant::parse("toolongvariant"),
        Err(ParseError::InvalidVariant { position: 0 })
    );

    // Generic subtag lexical checks.
    assert_eq!(
        Subtag::parse("toolongsubtag"),
        Err(ParseError::SubtagTooLong {
            position: 0,
            length: 13,
        })
    );
    assert_eq!(
        Subtag::parse("ab_cd"),
        Err(ParseError::InvalidCharacter { position: 2 })
    );
}
