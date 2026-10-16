use fy_i18n_lang_tag::normal::{ParseError, TagError};
use fy_i18n_lang_tag::{Fixed, NormalTag, NormalTagView};

#[cfg(feature = "alloc")]
use fy_i18n_lang_tag::Grow;

#[test]
fn test_normal_tag_view_const_and_basic() {
    const C_TAG: NormalTagView<'static> = match NormalTagView::parse("zh-Hans-CN-u-ca-gregory") {
        Ok(t) => t,
        Err(_) => unreachable!(),
    };
    assert_eq!(C_TAG.language().as_str(), "zh");
    assert_eq!(C_TAG.script(), Some("Hans"));
    assert_eq!(C_TAG.region(), Some("CN"));
    assert_eq!(C_TAG.u().unwrap().to_string(), "u-ca-gregory");
    assert_eq!(C_TAG.u().unwrap().as_str(), "ca-gregory");
    assert_eq!(C_TAG.to_string(), "zh-Hans-CN-u-ca-gregory");
}

#[test]
fn test_normal_tag_owned_default_fixed() {
    let tag: NormalTag = NormalTag::parse("zh-hans-cn").unwrap();
    assert_eq!(tag.language().as_str(), "zh");
    assert_eq!(tag.script(), Some("Hans"));
    assert_eq!(tag.region(), Some("CN"));
    assert_eq!(tag.extensions(), None);
    assert!(!tag.has_extensions());
    // Normalized output
    assert_eq!(tag.to_string(), "zh-Hans-CN");
}

#[test]
fn test_extensions_capacity_isolation() {
    // Critical architecture verification: MAX only measures extension subtags!
    // Core prefix "zh-Hans-CN" consumes 10 bytes, but does not use Fixed<8> extension capacity!
    type SmallExtTag = NormalTag<Fixed<8>>;

    // Extension "u-ca" (4 bytes <= 8 bytes) succeeds
    let tag = SmallExtTag::parse("zh-Hans-CN-u-ca").unwrap();
    assert_eq!(tag.extensions(), Some("u-ca"));
    assert_eq!(tag.to_string(), "zh-Hans-CN-u-ca");

    // Extension "u-ca-gregory" (12 bytes > 8 bytes) triggers capacity error
    let err = SmallExtTag::parse("zh-Hans-CN-u-ca-gregory").unwrap_err();
    match err {
        TagError::Capacity(cap) => {
            assert_eq!(cap.required, 12);
            assert_eq!(cap.max_capacity, 8);
        }
        TagError::Parse(_) => panic!("expected extension capacity error"),
    }
}

#[test]
fn test_case_insensitive_and_display() {
    let t1 = NormalTag::<Fixed>::parse("ZH-hAnS-cn-U-ca-gregory").unwrap();
    let t2 = NormalTag::<Fixed>::parse("zh-Hans-CN-u-ca-gregory").unwrap();
    let view = NormalTagView::parse("zh-HANS-cn-u-ca-GREGORY").unwrap();

    assert_eq!(t1, t2);
    assert_eq!(t1, view);
    assert_eq!(view, t2);

    // Display verifies RFC 5646 recommended formatting:
    // Language lowercase, script Titlecase, region uppercase, extensions lowercase
    assert_eq!(t1.to_string(), "zh-Hans-CN-u-ca-gregory");
}

#[test]
fn test_number_region() {
    // UN M.49 3-digit region
    let tag: NormalTag = NormalTag::parse("es-419").unwrap();
    assert_eq!(tag.language().as_str(), "es");
    assert_eq!(tag.script(), None);
    assert_eq!(tag.region(), Some("419"));
    assert_eq!(tag.to_string(), "es-419");
}

#[cfg(feature = "alloc")]
#[test]
fn test_normal_tag_grow_backend() {
    type DynTag = NormalTag<Grow>;
    let long_ext = "u-ca-gregory-co-pinyin-ka-shifted-kf-upper-kn-true";
    let input = format!("zh-Hans-CN-{}", long_ext);

    let tag = DynTag::parse(&input).unwrap();
    assert_eq!(tag.language().as_str(), "zh");
    assert_eq!(tag.script(), Some("Hans"));
    assert_eq!(tag.region(), Some("CN"));
    assert_eq!(tag.extensions(), Some(long_ext));
    assert_eq!(tag.to_string(), input);
}

#[test]
fn test_invalid_syntax() {
    // Empty input
    assert_eq!(NormalTagView::parse(""), Err(ParseError::Empty));

    // 4ALPHA primary language valid per RFC 5646 reserved grammar
    assert!(NormalTagView::parse("abcd-US").is_ok());

    // 1-character primary language rejected
    assert_eq!(
        NormalTagView::parse("a-US"),
        Err(ParseError::InvalidLanguage)
    );

    // Trailing hyphens
    assert_eq!(
        NormalTagView::parse("zh-"),
        Err(ParseError::TrailingSeparator)
    );
    assert_eq!(
        NormalTagView::parse("zh-Hans-"),
        Err(ParseError::TrailingSeparator)
    );

    // Extension containing empty subtag
    assert_eq!(
        NormalTagView::parse("zh-Hans--u"),
        Err(ParseError::EmptySubtag)
    );

    // Invalid variant (e.g. 3-character "abc")
    assert_eq!(
        NormalTagView::parse("zh-Hans-CN-abc"),
        Err(ParseError::InvalidVariant)
    );

    // Extension singleton missing subsequent subtags
    assert_eq!(
        NormalTagView::parse("zh-Hans-CN-u"),
        Err(ParseError::InvalidExtension)
    );
}

#[test]
fn test_subtag_phases_combination() {
    // Scenario 1: variant + extension + privateuse strictly in RFC 5646 order
    // Extensions total 40 bytes, using Fixed<64>
    let tag = NormalTag::<Fixed<64>>::parse("sl-rozaj-1996-u-ca-gregory-x-private-subtag").unwrap();
    assert_eq!(tag.language().as_str(), "sl");
    assert_eq!(tag.script(), None);
    assert_eq!(tag.region(), None);
    assert_eq!(
        tag.extensions(),
        Some("rozaj-1996-u-ca-gregory-x-private-subtag")
    );
    assert_eq!(
        tag.to_string(),
        "sl-rozaj-1996-u-ca-gregory-x-private-subtag"
    );

    // Scenario 2: Language directly followed by private use
    let tag2: NormalTag = NormalTag::parse("en-x-custom").unwrap();
    assert_eq!(tag2.language().as_str(), "en");
    assert_eq!(tag2.extensions(), Some("x-custom"));
    assert_eq!(tag2.to_string(), "en-x-custom");
}

#[test]
fn test_reject_grandfathered_tags() {
    // Legacy grandfathered tags without modern syntax are rejected
    assert!(NormalTagView::parse("en-GB-oed").is_err());
    assert!(NormalTagView::parse("i-klingon").is_err());
    assert!(NormalTagView::parse("i-default").is_err());
    assert!(NormalTagView::parse("zh-min-nan").is_err());
    assert!(NormalTagView::parse("no-bok").is_err());
    assert!(NormalTagView::parse("sgn-BE-FR").is_err());
}

#[test]
fn test_owned_tag_clone_semantics() {
    let tag1: NormalTag = NormalTag::parse("zh-Hans-CN-u-ca-gregory").unwrap();
    let tag2 = tag1.clone();
    assert_eq!(tag1, tag2);

    // Verify ownership move (not implicit Copy)
    let tag3 = tag1;
    assert_eq!(tag3, tag2);
}

#[test]
fn test_language_view_standalone() {
    use fy_i18n_lang_tag::LanguageView;

    // Valid tests (2-3 chars, 4ALPHA, 5-8 chars)
    let l1 = LanguageView::parse("zh").unwrap();
    let l2 = LanguageView::parse("ZH").unwrap();
    assert_eq!(l1, l2);
    assert_eq!(l1.as_str(), "zh");
    assert_eq!(l2.as_str(), "ZH");
    assert_eq!(l1.len(), 2);

    let l3 = LanguageView::parse("cmn").unwrap();
    assert_eq!(l3.as_str(), "cmn");

    let l_4alpha = LanguageView::parse("zhxx").unwrap();
    assert_eq!(l_4alpha.as_str(), "zhxx");

    // Case transitivity
    let l_upper = LanguageView::parse("ZH").unwrap();
    let l_lower = LanguageView::parse("zh").unwrap();
    assert_eq!(l1, l_upper);
    assert_eq!(l_upper, l_lower);
    assert_eq!(l1, l_lower);

    // Invalid tests
    assert_eq!(LanguageView::parse(""), Err(ParseError::Empty));
    assert_eq!(LanguageView::parse("a"), Err(ParseError::InvalidLanguage));
    assert_eq!(
        LanguageView::parse("toolongsubtag"),
        Err(ParseError::InvalidLanguage)
    );
    assert_eq!(LanguageView::parse("z1"), Err(ParseError::InvalidLanguage));
}

#[test]
fn test_assemble_and_derive_tag_without_validation() {
    use fy_i18n_lang_tag::LanguageView;

    let tag: NormalTag = NormalTag::parse("zh-Hans-CN-u-ca-gregory").unwrap();

    // 1. Get strongly-typed language view
    let lang_view: LanguageView = tag.language();
    assert_eq!(lang_view.as_str(), "zh");

    // 2. Derive tag with LanguageView: zero validation checks
    let ja_view = LanguageView::parse("ja").unwrap();
    let new_tag = tag.with_language(ja_view);
    assert_eq!(new_tag.language().as_str(), "ja");
    assert_eq!(new_tag.script(), Some("Hans"));
    assert_eq!(new_tag.region(), Some("CN"));
    assert_eq!(new_tag.to_string(), "ja-Hans-CN-u-ca-gregory");

    // Original tag unchanged (value object semantics)
    assert_eq!(tag.language().as_str(), "zh");

    // 3. Extract LanguageView from existing tag to assemble another tag
    let en_tag: NormalTag = NormalTag::parse("en-US").unwrap();
    let derived = tag.with_language(en_tag.language());
    assert_eq!(derived.language().as_str(), "en");
    assert_eq!(derived.to_string(), "en-Hans-CN-u-ca-gregory");

    // 4. Construct using from_components with LanguageView
    let assembled: NormalTag = NormalTag::from_components(
        en_tag.language(),
        Some("Latn"),
        Some("GB"),
        Some("u-ca-gregory"),
    )
    .unwrap();
    assert_eq!(assembled.language().as_str(), "en");
    assert_eq!(assembled.script(), Some("Latn"));
    assert_eq!(assembled.region(), Some("GB"));
    assert_eq!(assembled.to_string(), "en-Latn-GB-u-ca-gregory");

    // 5. with_script / with_region derivation tests
    let mod_region = assembled.with_region(Some("US")).unwrap();
    assert_eq!(mod_region.to_string(), "en-Latn-US-u-ca-gregory");
    let no_script = mod_region.with_script(None).unwrap();
    assert_eq!(no_script.to_string(), "en-US-u-ca-gregory");
}
