//! Public API contracts for constructors, transformations, traits, and Serde.

use core::cmp::Ordering;
use core::hash::{Hash, Hasher};
use core::str::FromStr;
use std::hash::DefaultHasher;

use fy_i18n_lang_tag::{
    BuildError, Extension, Language, NormalTag, ParseError, PrivateUseTag, Region, Script, Subtag,
    Tag, Variant,
};

fn hash<T: Hash>(value: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

#[test]
fn component_construction_errors_preserve_context_and_precedence() {
    let valid = Subtag::parse("aa").unwrap();
    let short = Subtag::parse("b").unwrap();
    let invalid: Result<Extension, BuildError> = Extension::new('X', vec![]);
    assert_eq!(
        invalid,
        Err(BuildError::InvalidSingleton { singleton: 'X' })
    );
    assert_eq!(Extension::new('u', vec![]), Err(BuildError::EmptyExtension));
    assert_eq!(
        Extension::new('u', vec![valid, short, short]),
        Err(BuildError::InvalidExtensionSubtag { index: 1 })
    );
    let private: Result<PrivateUseTag, BuildError> = PrivateUseTag::new(vec![]);
    assert_eq!(private, Err(BuildError::EmptyPrivateUse));
    let cases = [
        (
            BuildError::InvalidSingleton { singleton: 'X' },
            "cannot build extension: invalid singleton 'X'",
        ),
        (
            BuildError::EmptyExtension,
            "cannot build extension: empty subtag sequence",
        ),
        (
            BuildError::InvalidExtensionSubtag { index: 1 },
            "cannot build extension: subtag at index 1 must contain at least 2 characters",
        ),
        (
            BuildError::EmptyPrivateUse,
            "cannot build private-use tag: empty subtag sequence",
        ),
    ];
    for (error, message) in cases {
        assert_eq!(error.to_string(), message);
        let error: &dyn core::error::Error = &error;
        assert!(error.source().is_none());
    }
    assert_eq!(
        BuildError::InvalidSingleton { singleton: '\n' }.to_string(),
        "cannot build extension: invalid singleton '\\n'"
    );
    assert_eq!(
        Extension::parse("x-aa"),
        Err(ParseError::InvalidExtension { position: 0 })
    );
    assert_eq!(
        PrivateUseTag::parse("x"),
        Err(ParseError::InvalidPrivateUse { position: 0 })
    );
}

#[test]
fn atomic_subtag_traits_use_normalized_text() {
    macro_rules! check {
        ($ty:ty, $input:literal, $normalized:literal, $greater:literal) => {{
            let value = <$ty>::from_str($input).unwrap();
            let equivalent = <$ty>::try_from($normalized).unwrap();
            let greater = <$ty>::parse($greater).unwrap();
            assert_eq!(value, equivalent);
            assert_eq!(value.to_string(), $normalized);
            assert_eq!(AsRef::<str>::as_ref(&value), $normalized);
            assert_eq!(value.as_bytes(), $normalized.as_bytes());
            assert_eq!(value.len(), $normalized.len());
            assert_eq!(hash(&value), hash(&equivalent));
            assert_eq!(value.cmp(&equivalent), Ordering::Equal);
            assert_eq!(value.partial_cmp(&greater), Some(Ordering::Less));
            assert_eq!(greater.cmp(&value), Ordering::Greater);
            assert_eq!(<$ty>::from_str(""), Err(ParseError::Empty));
            assert_eq!(<$ty>::try_from(""), Err(ParseError::Empty));
        }};
    }
    check!(Language, "EN", "en", "zh");
    check!(Script, "HANS", "Hans", "Latn");
    check!(Region, "cn", "CN", "US");
    check!(Variant, "POSIX", "posix", "rozaj");
    check!(Subtag, "AA", "aa", "bb");
}

#[test]
fn atomic_subtags_reject_invalid_shapes_and_characters() {
    for input in ["e", "abcd", "abcdefghi", "e1", "en_", "\u{e9}"] {
        assert_eq!(
            Language::parse(input),
            Err(ParseError::InvalidLanguage { position: 0 })
        );
    }
    for input in ["abc", "abcde", "Lat1", "1atn", "\u{e9}ab"] {
        assert_eq!(
            Script::parse(input),
            Err(ParseError::InvalidScript { position: 0 })
        );
    }
    for input in ["a", "abcd", "1A", "A1", "a12", "1a2", "12a"] {
        assert_eq!(
            Region::parse(input),
            Err(ParseError::InvalidRegion { position: 0 })
        );
    }
    for input in ["abcd", "1ab_", "abc_e", "abcdefghi"] {
        assert_eq!(
            Variant::parse(input),
            Err(ParseError::InvalidVariant { position: 0 })
        );
    }
    for input in ["0", "999", "us"] {
        assert_eq!(Region::parse(input).is_ok(), input.len() != 1);
    }
    for input in ["a", "ABCDEFGH", "01234567"] {
        let value = Subtag::parse(input).unwrap();
        assert_eq!(value.as_str(), input.to_ascii_lowercase());
    }
    assert_eq!(
        Subtag::parse("a-"),
        Err(ParseError::InvalidCharacter { position: 1 })
    );
}

#[test]
fn private_use_entry_points_and_conversions_preserve_values() {
    let private = PrivateUseTag::from_str("X-CUSTOM-1").unwrap();
    assert_eq!(PrivateUseTag::try_from("x-custom-1").unwrap(), private);
    let tag = Tag::from(private.clone());
    assert_eq!(Tag::from_str("X-CUSTOM-1").unwrap(), tag);
    assert_eq!(tag.as_normal(), None);
    assert_eq!(tag.as_private_use(), Some(&private));
    assert_eq!(PrivateUseTag::try_from(tag.clone()), Ok(private.clone()));
    assert_eq!(NormalTag::try_from(tag.clone()), Err(tag));

    let normal = Tag::from_str("en-US").unwrap();
    assert_eq!(PrivateUseTag::try_from(normal.clone()), Err(normal));
    for input in ["en", "en-x-note", "xx-aa"] {
        assert_eq!(
            PrivateUseTag::parse(input),
            Err(ParseError::MissingPrivateUsePrefix { position: 0 })
        );
    }
    assert_eq!(PrivateUseTag::from_str(""), Err(ParseError::Empty));
    assert_eq!(
        PrivateUseTag::try_from("x"),
        Err(ParseError::InvalidPrivateUse { position: 0 })
    );
    assert_eq!(
        hash(&private),
        hash(&PrivateUseTag::parse("x-custom-1").unwrap())
    );
}

#[test]
fn extension_traits_and_singleton_boundaries() {
    let extension = Extension::from_str("U-CA-BUDDHIST").unwrap();
    assert_eq!(extension, Extension::try_from("u-ca-buddhist").unwrap());
    assert_eq!(extension.to_string(), "u-ca-buddhist");
    assert_eq!(
        hash(&extension),
        hash(&Extension::parse("u-ca-buddhist").unwrap())
    );
    assert_eq!(
        Extension::parse("uu-aa"),
        Err(ParseError::InvalidExtension { position: 0 })
    );
    assert_eq!(Extension::from_str(""), Err(ParseError::Empty));
    assert_eq!(
        Extension::try_from("x-aa"),
        Err(ParseError::InvalidExtension { position: 0 })
    );
    for singleton in "0123456789abcdefghijklmnopqrstuvwyzABCDEFGHIJKLMNOPQRSTUVWXYZ".chars() {
        let result = Extension::new(singleton, vec![Subtag::parse("AB").unwrap()]);
        if singleton.eq_ignore_ascii_case(&'x') {
            assert_eq!(result, Err(BuildError::InvalidSingleton { singleton }));
        } else {
            let value = result.unwrap();
            assert_eq!(value.singleton(), singleton.to_ascii_lowercase());
            assert_eq!(Extension::parse(&value.to_string()).unwrap(), value);
        }
    }
    for singleton in ['\u{e9}', '\0', '-', ' '] {
        assert_eq!(
            Extension::new(singleton, vec![Subtag::parse("aa").unwrap()]),
            Err(BuildError::InvalidSingleton { singleton })
        );
    }
}

fn assert_ordered<T: Ord + core::fmt::Debug>(values: &[T]) {
    for (i, left) in values.iter().enumerate() {
        for (j, right) in values.iter().enumerate() {
            assert_eq!(left.cmp(right), i.cmp(&j), "{left:?} versus {right:?}");
            assert_eq!(left.partial_cmp(right), Some(i.cmp(&j)));
        }
    }
}

#[test]
fn ordering_is_structural_and_consistent_across_tag_kinds() {
    let normal: Vec<_> = [
        "en", "en-x-aa", "en-x-bb", "en-a-aa", "en-a-bb", "en-b-aa", "en-posix", "en-rozaj",
        "en-CN", "en-US", "en-Hans", "en-Latn", "zh",
    ]
    .into_iter()
    .map(|s| NormalTag::parse(s).unwrap())
    .collect();
    assert_ordered(&normal);
    let private: Vec<_> = ["x-aa", "x-aa-bb", "x-bb"]
        .into_iter()
        .map(|s| PrivateUseTag::parse(s).unwrap())
        .collect();
    assert_ordered(&private);
    let tags: Vec<_> = normal
        .into_iter()
        .map(Tag::from)
        .chain(private.into_iter().map(Tag::from))
        .collect();
    assert_ordered(&tags);
    let extensions: Vec<_> = ["0-aa", "a-aa", "a-aa-bb", "a-bb", "u-aa"]
        .into_iter()
        .map(|s| Extension::parse(s).unwrap())
        .collect();
    assert_ordered(&extensions);
}

#[test]
fn transformations_preserve_unmodified_components_and_round_trip() {
    let base = NormalTag::parse("en-Latn-US-posix-u-ca-buddhist-x-note").unwrap();
    let cases = [
        (
            base.clone().with_region(Some(Region::parse("CN").unwrap())),
            "en-Latn-CN-posix-u-ca-buddhist-x-note",
        ),
        (
            base.clone().with_region(None),
            "en-Latn-posix-u-ca-buddhist-x-note",
        ),
        (
            base.clone().with_script(None),
            "en-US-posix-u-ca-buddhist-x-note",
        ),
        (
            base.clone().without_variants(),
            "en-Latn-US-u-ca-buddhist-x-note",
        ),
        (base.clone().without_extensions(), "en-Latn-US-posix-x-note"),
        (
            base.clone().without_private_use(),
            "en-Latn-US-posix-u-ca-buddhist",
        ),
    ];
    for (value, expected) in cases {
        assert_eq!(value.to_string(), expected);
        assert_eq!(NormalTag::parse(expected).unwrap(), value);
    }
    assert_eq!(
        base.clone()
            .without_variants()
            .without_extensions()
            .without_private_use()
            .to_string(),
        "en-Latn-US"
    );
    let (language, script, region, variants, mut extensions, private_use) =
        base.clone().into_parts();
    extensions.push(Extension::parse("a-foo").unwrap());
    extensions.reverse();
    let rebuilt =
        NormalTag::from_parts(language, script, region, variants, extensions, private_use).unwrap();
    assert_eq!(
        rebuilt.to_string(),
        "en-Latn-US-posix-a-foo-u-ca-buddhist-x-note"
    );
    assert_eq!(rebuilt.without_extensions(), base.without_extensions());
}

#[test]
fn lexical_errors_take_precedence_over_structural_errors() {
    for (input, expected) in [
        (
            "en-abcdefghi-u-aa",
            ParseError::SubtagTooLong {
                position: 3,
                length: 9,
            },
        ),
        (
            "abcdefghi-en",
            ParseError::SubtagTooLong {
                position: 0,
                length: 9,
            },
        ),
        (
            "en-a_-abcdefghi",
            ParseError::InvalidCharacter { position: 4 },
        ),
        ("en--abcdefghi", ParseError::EmptySubtag { position: 3 }),
    ] {
        assert_eq!(Tag::parse(input), Err(expected.clone()));
        assert_eq!(NormalTag::parse(input), Err(expected));
    }
    assert_eq!(
        NormalTag::parse("en-abc"),
        Err(ParseError::UnexpectedSubtag { position: 3 })
    );
}

#[test]
fn diagnostics_follow_lexical_scan_order_and_grammar_context() {
    for (input, expected) in [
        (
            "toolongsub-en_US",
            ParseError::SubtagTooLong {
                position: 0,
                length: 10,
            },
        ),
        ("en--US_x", ParseError::EmptySubtag { position: 3 }),
        ("en-a1bc", ParseError::InvalidScript { position: 3 }),
        ("en-US-a1bc", ParseError::UnexpectedSubtag { position: 6 }),
        ("en-1a", ParseError::InvalidRegion { position: 3 }),
        ("en-US-1a", ParseError::UnexpectedSubtag { position: 6 }),
        ("en-a1b", ParseError::InvalidRegion { position: 3 }),
        ("en-US-a1b", ParseError::UnexpectedSubtag { position: 6 }),
        ("en-u-ca-a", ParseError::InvalidExtension { position: 8 }),
    ] {
        assert_eq!(NormalTag::parse(input), Err(expected.clone()));
        assert_eq!(Tag::parse(input), Err(expected));
    }
    assert_eq!(
        Extension::parse("u-ca-a"),
        Err(ParseError::UnexpectedSubtag { position: 5 })
    );
    for input in ["en", "xx-aa"] {
        let expected = ParseError::MissingPrivateUsePrefix { position: 0 };
        assert_eq!(PrivateUseTag::parse(input), Err(expected.clone()));
        assert_eq!(PrivateUseTag::from_str(input), Err(expected.clone()));
        assert_eq!(PrivateUseTag::try_from(input), Err(expected));
    }
    assert_eq!(
        PrivateUseTag::parse("en_"),
        Err(ParseError::InvalidCharacter { position: 2 })
    );
    assert_eq!(PrivateUseTag::parse("X-aa").unwrap().to_string(), "x-aa");
}

#[cfg(feature = "serde")]
#[test]
fn serde_round_trips_and_rejects_invalid_input_for_every_type() {
    fn check<T>(input: &str, normalized: &str, invalid: &str)
    where
        T: serde::Serialize + serde::de::DeserializeOwned + FromStr + Eq + core::fmt::Debug,
        T::Err: core::fmt::Debug,
    {
        let value = T::from_str(input).unwrap();
        let json = serde_json::to_string(&value).unwrap();
        assert_eq!(json, serde_json::to_string(normalized).unwrap());
        assert_eq!(serde_json::from_str::<T>(&json).unwrap(), value);
        assert_eq!(
            serde_json::from_str::<T>(&serde_json::to_string(input).unwrap()).unwrap(),
            value
        );
        assert!(serde_json::from_str::<T>(&serde_json::to_string(invalid).unwrap()).is_err());
        for json in ["null", "true", "42", "[]", "{}", "\"\""] {
            assert!(serde_json::from_str::<T>(json).is_err(), "accepted {json}");
        }
    }
    check::<Language>("EN", "en", "abcd");
    check::<Script>("LATN", "Latn", "Lat1");
    check::<Region>("us", "US", "1a2");
    check::<Variant>("POSIX", "posix", "abc");
    check::<Subtag>("NOTE", "note", "a_b");
    check::<Extension>("U-CA-BUDDHIST", "u-ca-buddhist", "u-a");
    check::<NormalTag>(
        "EN-latn-us-B-FOO-A-BAR-X-NOTE",
        "en-Latn-US-a-bar-b-foo-x-note",
        "x-note",
    );
    check::<PrivateUseTag>("X-NOTE", "x-note", "en");
    check::<Tag>("X-NOTE", "x-note", "en_US");
    check::<Tag>("EN-US", "en-US", "en-u");
}
