#[cfg(feature = "alloc")]
use fy_i18n_lang_tag::Grow;
use fy_i18n_lang_tag::private_use::{ParseError, TagError};
use fy_i18n_lang_tag::{Fixed, PrivateUseTag, PrivateUseTagView};
use std::str::FromStr;

#[test]
fn test_private_use_tag_fixed_default() {
    // Defaults to Fixed<32>
    let tag: PrivateUseTag = PrivateUseTag::parse("X-Custom-Env").unwrap();
    // Normalized to lowercase
    assert_eq!(tag.as_str(), "custom-env");
    assert_eq!(tag.to_string(), "x-custom-env");

    // Iterate subtags
    let subtags: Vec<&str> = tag.subtags().collect();
    assert_eq!(subtags, vec!["custom", "env"]);

    // FromStr test
    let tag_from_str: PrivateUseTag = PrivateUseTag::from_str("x-custom-env").unwrap();
    assert_eq!(tag, tag_from_str);

    // Compare with borrowed view
    let view = PrivateUseTagView::parse("X-CUSTOM-ENV").unwrap();
    assert_eq!(tag, view);
    assert_eq!(view, tag);

    // as_view conversion
    let view2 = tag.as_view();
    assert_eq!(view2.as_str(), "custom-env");
}

#[test]
fn test_private_use_tag_fixed_explicit() {
    // Explicitly specify Fixed default 32 bytes
    let tag = PrivateUseTag::<Fixed>::parse("x-hello-world").unwrap();
    assert_eq!(tag.as_str(), "hello-world");
    assert_eq!(tag.max_capacity(), 32);
}

#[test]
fn test_private_use_tag_fixed_capacity_limit() {
    // Fixed with 8 bytes capacity
    type SmallTag = PrivateUseTag<Fixed<8>>;

    // "a-b-c" takes 5 bytes <= 8 bytes, succeeds
    let tag = SmallTag::parse("x-a-b-c").unwrap();
    assert_eq!(tag.as_str(), "a-b-c");

    // "very-long" takes 9 bytes > 8 bytes, triggers capacity error
    let err = SmallTag::parse("x-very-long").unwrap_err();
    match err {
        TagError::Capacity(cap_err) => {
            assert_eq!(cap_err.required, 9);
            assert_eq!(cap_err.max_capacity, 8);
        }
        TagError::Parse(_) => panic!("expected capacity error"),
    }
}

#[test]
fn test_private_use_tag_syntax_error() {
    let err = PrivateUseTag::<Fixed>::parse("invalid-prefix").unwrap_err();
    match err {
        TagError::Parse(parse_err) => {
            assert_eq!(parse_err, ParseError::InvalidPrefix);
        }
        TagError::Capacity(_) => panic!("expected parse error"),
    }
}

#[test]
fn test_private_use_tag_from_subtags() {
    let tag: PrivateUseTag = PrivateUseTag::from_subtags("ALPHA-BETA").unwrap();
    assert_eq!(tag.as_str(), "alpha-beta");
    assert_eq!(tag.to_string(), "x-alpha-beta");
}

#[cfg(feature = "alloc")]
#[test]
fn test_private_use_tag_grow_backend() {
    // Grow backend (default usize::MAX capacity)
    type DynamicTag = PrivateUseTag<Grow>;

    let long_subtags = "subtag1-subtag2-subtag3-subtag4-subtag5-subtag6";
    let input = format!("x-{}", long_subtags);

    let tag = DynamicTag::parse(&input).unwrap();
    assert_eq!(tag.as_str(), long_subtags);
    assert_eq!(tag.to_string(), input);
    assert_eq!(tag.max_capacity(), usize::MAX);

    // Test Grow<20> with custom bound
    type LimitedGrowTag = PrivateUseTag<Grow<20>>;
    let err = LimitedGrowTag::parse(&input).unwrap_err();
    match err {
        TagError::Capacity(cap) => {
            assert_eq!(cap.max_capacity, 20);
        }
        TagError::Parse(_) => panic!("expected capacity error"),
    }
}
