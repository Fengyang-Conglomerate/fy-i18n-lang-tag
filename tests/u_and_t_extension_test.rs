//! Integration tests for u-extension (RFC 6067) and t-extension (RFC 6497) validation.

use fy_i18n_lang_tag::normal::error::ParseError;
use fy_i18n_lang_tag::normal::{NormalTag, NormalTagView, t, u};

#[test]
fn test_u_extension_valid_cases() {
    // Attributes and keywords
    assert_eq!(u::validate("u-attr-co-phonebk"), Ok(()));
    assert_eq!(u::validate_subtags("attr-co-phonebk"), Ok(()));

    // Pure keywords
    assert_eq!(u::validate("u-ca-gregory"), Ok(()));
    assert_eq!(u::validate_subtags("ca-gregory"), Ok(()));

    // Multiple keywords
    assert_eq!(u::validate("u-ca-gregory-nu-latn-co-phonebk"), Ok(()));

    // Key only without type
    assert_eq!(u::validate("u-ca"), Ok(()));
    assert_eq!(u::validate("u-ca-nu-latn"), Ok(()));

    // Single key with multiple types
    assert_eq!(u::validate("u-ca-gregory-nu-latn-deva"), Ok(()));

    // Attributes only
    assert_eq!(u::validate("u-attr"), Ok(()));
    assert_eq!(u::validate("u-attr1-attr2"), Ok(()));

    // Mixed case
    assert_eq!(u::validate("U-CA-gregory"), Ok(()));
}

#[test]
fn test_u_extension_duplicate_key() {
    // Identical key duplication
    assert_eq!(
        u::validate("u-ca-gregory-ca-japanese"),
        Err(ParseError::DuplicateUKey)
    );

    // Case-insensitive key duplication
    assert_eq!(
        u::validate("u-ca-gregory-CA-japanese"),
        Err(ParseError::DuplicateUKey)
    );
    assert_eq!(
        u::validate_subtags("nu-latn-NU-arab"),
        Err(ParseError::DuplicateUKey)
    );

    // Key-only duplication
    assert_eq!(u::validate("u-ca-co-ca"), Err(ParseError::DuplicateUKey));
}

#[test]
fn test_u_extension_invalid_syntax() {
    // Empty or trailing hyphen
    assert_eq!(u::validate_subtags(""), Err(ParseError::EmptySubtag));
    assert_eq!(u::validate("u-"), Err(ParseError::EmptySubtag));
    assert_eq!(u::validate("u-ca-"), Err(ParseError::TrailingSeparator));

    // Subtag too long
    assert_eq!(
        u::validate("u-ca-toolongsubtag"),
        Err(ParseError::SubtagTooLong)
    );

    // Invalid character
    assert_eq!(
        u::validate("u-ca-greg@ry"),
        Err(ParseError::InvalidCharacter)
    );

    // Single character subtag (invalid within u-extension)
    assert_eq!(u::validate("u-ca-a-gregory"), Err(ParseError::InvalidUType));
}

#[test]
fn test_t_extension_valid_cases() {
    // Mode A: Simple source language (2-3 chars and 4ALPHA like root)
    assert_eq!(t::validate("t-it"), Ok(()));
    assert_eq!(t::validate("t-root"), Ok(()));
    assert_eq!(t::validate_subtags("it"), Ok(()));

    // Source language with script and region
    assert_eq!(t::validate("t-und-cyrl"), Ok(()));
    assert_eq!(t::validate("t-zh-hans-cn"), Ok(()));

    // Source language with variants
    assert_eq!(t::validate("t-en-fonipa"), Ok(()));

    // Source language followed by fields
    assert_eq!(t::validate("t-und-latn-m0-ungegn-2007"), Ok(()));
    assert_eq!(t::validate("t-ja-m0-ungegn-t0-und"), Ok(()));

    // Mode B: No source language, direct fields
    assert_eq!(t::validate("t-m0-ungegn"), Ok(()));
    assert_eq!(t::validate("t-m0-ungegn-t0-und"), Ok(()));
    assert_eq!(t::validate("t-m0-ungegn-2007-approved"), Ok(()));

    // Mixed case
    assert_eq!(t::validate("T-ZH-HANS-M0-UNGEGN"), Ok(()));
}

#[test]
fn test_t_extension_duplicate_field_separator() {
    // Duplicate fsep
    assert_eq!(
        t::validate("t-m0-ungegn-m0-loc"),
        Err(ParseError::DuplicateTField)
    );

    // Case-insensitive duplicate fsep
    assert_eq!(
        t::validate("t-m0-ungegn-M0-loc"),
        Err(ParseError::DuplicateTField)
    );
    assert_eq!(
        t::validate_subtags("it-m0-ungegn-M0-other"),
        Err(ParseError::DuplicateTField)
    );
}

#[test]
fn test_t_extension_invalid_syntax() {
    // Empty or trailing hyphen
    assert_eq!(t::validate_subtags(""), Err(ParseError::EmptySubtag));
    assert_eq!(t::validate("t-"), Err(ParseError::EmptySubtag));

    // fsep missing values
    assert_eq!(t::validate("t-m0"), Err(ParseError::InvalidTField));
    assert_eq!(t::validate("t-m0-t0-value"), Err(ParseError::InvalidTField));
    assert_eq!(t::validate("t-it-m0"), Err(ParseError::InvalidTField));

    // Invalid source language
    assert_eq!(t::validate("t-a"), Err(ParseError::InvalidTLanguage));
    assert_eq!(t::validate("t-12"), Err(ParseError::InvalidTLanguage));

    // Subtag too long
    assert_eq!(
        t::validate("t-m0-toolongfieldvalue"),
        Err(ParseError::SubtagTooLong)
    );

    // Invalid characters
    assert_eq!(t::validate("t-m0-ung#gn"), Err(ParseError::InvalidTField));
}

#[test]
fn test_normal_tag_integration_with_u_and_t() {
    // Valid tag with u-extension
    let tag: NormalTag = NormalTag::parse("zh-Hans-CN-u-ca-gregory").unwrap();
    assert_eq!(tag.language().as_str(), "zh");
    assert_eq!(tag.script(), Some("Hans"));
    assert_eq!(tag.region(), Some("CN"));
    assert_eq!(tag.extensions(), Some("u-ca-gregory"));

    // Valid tag with t-extension
    let tag2: NormalTag = NormalTag::parse("ja-Kana-JP-t-it-m0-ungegn").unwrap();
    assert_eq!(tag2.language().as_str(), "ja");
    assert_eq!(tag2.script(), Some("Kana"));
    assert_eq!(tag2.region(), Some("JP"));
    assert_eq!(tag2.extensions(), Some("t-it-m0-ungegn"));

    // Both u-extension and t-extension (normalized in ASCII order t-it-u-ca-gregory)
    let tag3: NormalTag = NormalTag::parse("zh-u-ca-gregory-t-it").unwrap();
    assert_eq!(tag3.language().as_str(), "zh");
    assert_eq!(tag3.extensions(), Some("t-it-u-ca-gregory"));

    // Reject u-extension with duplicate keys
    assert_eq!(
        NormalTagView::parse("zh-u-ca-gregory-ca-japanese"),
        Err(ParseError::DuplicateUKey)
    );

    // Reject t-extension with duplicate fsep
    assert_eq!(
        NormalTagView::parse("ja-t-it-m0-ungegn-m0-other"),
        Err(ParseError::DuplicateTField)
    );

    // Reject duplicate singletons (e.g. duplicate u extensions)
    assert_eq!(
        NormalTagView::parse("zh-u-ca-gregory-u-nu-latn"),
        Err(ParseError::DuplicateSingleton)
    );

    // Reject duplicate variants
    assert_eq!(
        NormalTagView::parse("sl-rozaj-rozaj"),
        Err(ParseError::DuplicateVariant)
    );
}

#[test]
fn test_structured_views_api() {
    use fy_i18n_lang_tag::{Fixed, NormalTag, PrivateUseTagView};

    let tag = NormalTag::<Fixed<64>>::parse(
        "zh-Hans-CN-rozaj-a-exta-u-attr-ca-gregory-t-it-m0-ungegn-x-mydata",
    )
    .unwrap();

    // Core subtags
    assert_eq!(tag.language().as_str(), "zh");
    assert_eq!(tag.script(), Some("Hans"));
    assert_eq!(tag.region(), Some("CN"));

    // Variants
    assert_eq!(tag.variants(), Some("rozaj"));

    // u-extension view
    let u_view = tag.u().expect("must contain u extension");
    assert_eq!(u_view.keyword_type("ca"), Some("gregory"));
    assert!(u_view.has_key("ca"));
    assert!(!u_view.has_key("nu"));
    assert!(u_view.has_attribute("attr"));
    assert!(!u_view.has_attribute("phonebk"));
    assert_eq!(u_view.to_string(), "u-attr-ca-gregory");

    // t-extension view
    let t_view = tag.t().expect("must contain t extension");
    assert_eq!(t_view.source_language(), Some("it"));
    assert_eq!(t_view.field_value("m0"), Some("ungegn"));
    assert!(t_view.has_field("m0"));
    assert!(!t_view.has_field("t0"));
    assert_eq!(t_view.to_string(), "t-it-m0-ungegn");

    // Other single-letter extension views (a-)
    assert!(tag.has_other_extensions());
    let ext_a = tag.other_extension('a').expect("must contain extension a");
    assert_eq!(ext_a.singleton(), 'a');
    assert_eq!(ext_a.singleton_byte(), b'a');
    assert_eq!(ext_a.subtags(), "exta");
    assert_eq!(ext_a.as_str(), "exta");
    assert_eq!(ext_a.to_string(), "a-exta");
    assert_eq!(tag.other_extension('b'), None);
    let all_others: Vec<_> = tag.other_extensions().collect();
    assert_eq!(all_others.len(), 1);
    assert_eq!(all_others[0], ext_a);

    // Private use extension (x-), reusing PrivateUseTagView
    let pu_view = tag
        .private_use()
        .expect("must contain private use extension");
    assert_eq!(pu_view.as_str(), "mydata");
    assert_eq!(pu_view.to_string(), "x-mydata");

    // Equivalent to independent PrivateUseTagView::parse
    let independent_pu = PrivateUseTagView::parse("x-mydata").unwrap();
    assert_eq!(pu_view, independent_pu);
}
