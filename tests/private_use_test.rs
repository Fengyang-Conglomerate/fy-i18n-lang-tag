use core::ops::Deref;
use fy_i18n_lang_tag::PrivateUseTagView;
use fy_i18n_lang_tag::private_use::ParseError;
use std::collections::HashSet;

#[test]
fn test_valid_private_use_tags() {
    let input = "x-a-b-c";
    let view = PrivateUseTagView::parse(input).expect("should parse successfully");
    // Verify stored slice is stripped of "x-" prefix
    assert_eq!(view.as_str(), "a-b-c");

    // Verify subtag iteration
    let subtags: Vec<&str> = view.subtags().collect();
    assert_eq!(subtags, vec!["a", "b", "c"]);

    // Verify Display
    assert_eq!(view.to_string(), "x-a-b-c");

    // Single subtag
    let view_single = PrivateUseTagView::parse("x-custom").unwrap();
    assert_eq!(view_single.as_str(), "custom");
    assert_eq!(view_single.to_string(), "x-custom");

    // Boundary length: each subtag 1 to 8 characters
    let view_max = PrivateUseTagView::parse("x-1-12345678-abc").unwrap();
    assert_eq!(view_max.as_str(), "1-12345678-abc");
}

#[test]
fn test_const_evaluation() {
    const C_TAG: PrivateUseTagView<'static> = match PrivateUseTagView::parse("x-const-tag") {
        Ok(t) => t,
        Err(_) => unreachable!(),
    };
    const C_SUBTAG: PrivateUseTagView<'static> =
        match PrivateUseTagView::from_subtags("const-subtag") {
            Ok(t) => t,
            Err(_) => unreachable!(),
        };

    assert_eq!(C_TAG.as_str(), "const-tag");
    assert_eq!(C_SUBTAG.as_str(), "const-subtag");
}

#[test]
fn test_display_lowercase_recommendation() {
    // RFC 5646 §2.1.1 recommends lowercase for private use tags
    let view = PrivateUseTagView::parse("X-HeLLo-WoRLD").unwrap();
    // as_str returns original slice
    assert_eq!(view.as_str(), "HeLLo-WoRLD");
    // Display normalizes to lowercase
    assert_eq!(view.to_string(), "x-hello-world");
}

#[test]
fn test_case_insensitive_eq_and_hash() {
    let v1 = PrivateUseTagView::parse("x-custom-TAG").unwrap();
    let v2 = PrivateUseTagView::parse("X-CUSTOM-tag").unwrap();
    let v3 = PrivateUseTagView::parse("x-other").unwrap();

    // Case-insensitive comparison
    assert_eq!(v1, v2);
    assert_ne!(v1, v3);

    // Hash consistency
    let mut set = HashSet::new();
    set.insert(v1);
    assert!(set.contains(&v2));

    // Ord ordering consistency
    assert_eq!(v1.cmp(&v2), core::cmp::Ordering::Equal);
}

#[test]
fn test_as_ref_and_deref_traits() {
    let view = PrivateUseTagView::parse("x-foo-bar").unwrap();

    // AsRef<str>
    let s_ref: &str = view.as_ref();
    assert_eq!(s_ref, "foo-bar");

    // Deref
    assert_eq!(view.deref(), "foo-bar");
    // Deref coercion
    assert!(view.starts_with("foo"));
}

#[test]
fn test_from_subtags_constructor() {
    // Construct from subtag string without x- prefix
    let view = PrivateUseTagView::from_subtags("alpha-beta-gamma").unwrap();
    assert_eq!(view.as_str(), "alpha-beta-gamma");
    assert_eq!(view.to_string(), "x-alpha-beta-gamma");

    // Invalid cases: empty input, consecutive hyphens, subtag too long
    assert_eq!(
        PrivateUseTagView::from_subtags(""),
        Err(ParseError::EmptySubtag)
    );
    assert_eq!(
        PrivateUseTagView::from_subtags("a--b"),
        Err(ParseError::EmptySubtag)
    );
    assert_eq!(
        PrivateUseTagView::from_subtags("a-"),
        Err(ParseError::TrailingSeparator)
    );
    assert_eq!(
        PrivateUseTagView::from_subtags("toolongsubtag"),
        Err(ParseError::SubtagTooLong)
    );
    assert_eq!(
        PrivateUseTagView::from_subtags("inv@lid"),
        Err(ParseError::InvalidCharacter)
    );
}

#[test]
fn test_try_from_trait() {
    let raw = "x-alpha-beta";
    let view: PrivateUseTagView = raw.try_into().expect("TryFrom should parse successfully");
    assert_eq!(view.as_str(), "alpha-beta");
}

#[test]
fn test_invalid_cases() {
    // Empty input
    assert_eq!(PrivateUseTagView::parse(""), Err(ParseError::Empty));

    // Invalid prefix
    assert_eq!(
        PrivateUseTagView::parse("x"),
        Err(ParseError::InvalidPrefix)
    );
    assert_eq!(
        PrivateUseTagView::parse("x-"),
        Err(ParseError::InvalidPrefix)
    );
    assert_eq!(
        PrivateUseTagView::parse("en-US"),
        Err(ParseError::InvalidPrefix)
    );
    assert_eq!(
        PrivateUseTagView::parse("y-custom"),
        Err(ParseError::InvalidPrefix)
    );

    // Consecutive hyphens
    assert_eq!(
        PrivateUseTagView::parse("x--a"),
        Err(ParseError::EmptySubtag)
    );
    assert_eq!(
        PrivateUseTagView::parse("x-a--b"),
        Err(ParseError::EmptySubtag)
    );

    // Trailing hyphens
    assert_eq!(
        PrivateUseTagView::parse("x-a-"),
        Err(ParseError::TrailingSeparator)
    );
    assert_eq!(
        PrivateUseTagView::parse("x-a-b-"),
        Err(ParseError::TrailingSeparator)
    );

    // Subtag too long (> 8 chars)
    assert_eq!(
        PrivateUseTagView::parse("x-123456789"),
        Err(ParseError::SubtagTooLong)
    );
    assert_eq!(
        PrivateUseTagView::parse("x-ok-toolongsubtag"),
        Err(ParseError::SubtagTooLong)
    );

    // Invalid characters (underscore, dot, space, non-ASCII)
    assert_eq!(
        PrivateUseTagView::parse("x-a_b"),
        Err(ParseError::InvalidCharacter)
    );
    assert_eq!(
        PrivateUseTagView::parse("x-a.b"),
        Err(ParseError::InvalidCharacter)
    );
    assert_eq!(
        PrivateUseTagView::parse("x-a b"),
        Err(ParseError::InvalidCharacter)
    );
}
