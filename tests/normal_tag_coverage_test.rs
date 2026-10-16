//! Comprehensive tests for NormalTag, NormalTagView, U, T, Variant, and Storage.

use core::cmp::Ordering;
use core::str::FromStr;

use fy_i18n_lang_tag::normal::extensions::is_extension_subtag;
use fy_i18n_lang_tag::normal::variant::{is_valid, is_valid_bytes, validate};
use fy_i18n_lang_tag::{
    Fixed, LanguageView, NormalTag, PrivateUseTag, RegionView, ScriptView, Storage, TExtensionView,
    UExtensionView,
};

#[cfg(feature = "alloc")]
use fy_i18n_lang_tag::Grow;

#[test]
fn test_normal_tag_derivation_and_component_constructors() {
    let lang = LanguageView::parse("en").unwrap();
    let script = ScriptView::parse("Latn").unwrap();
    let region = RegionView::parse("US").unwrap();

    // 1. from_components with empty strings treated as None
    let tag: NormalTag = NormalTag::from_components(lang, Some(""), Some(""), Some("")).unwrap();
    assert_eq!(tag.to_string(), "en");
    assert_eq!(tag.script(), None);
    assert_eq!(tag.region(), None);
    assert_eq!(tag.extensions(), None);

    // 2. from_components with full parts
    let tag_full: NormalTag =
        NormalTag::from_components(lang, Some("Latn"), Some("US"), Some("u-ca-gregory")).unwrap();
    assert_eq!(tag_full.to_string(), "en-Latn-US-u-ca-gregory");
    assert_eq!(tag_full.script_view(), Some(script));
    assert_eq!(tag_full.region_view(), Some(region));
    assert_eq!(tag_full.language_str(), "en");

    // 3. with_language & with_language_str
    let tag_zh = tag_full.with_language_str("zh").unwrap();
    assert_eq!(tag_zh.language_str(), "zh");
    assert_eq!(tag_zh.to_string(), "zh-Latn-US-u-ca-gregory");
    assert!(tag_full.with_language_str("invalid_lang").is_err());

    // 4. with_script & with_script_view
    let tag_no_script = tag_full.with_script(None).unwrap();
    assert_eq!(tag_no_script.script(), None);
    let tag_empty_script = tag_full.with_script(Some("")).unwrap();
    assert_eq!(tag_empty_script.script(), None);
    let tag_hans = tag_no_script.with_script_view(Some(ScriptView::parse("Hans").unwrap()));
    assert_eq!(tag_hans.script(), Some("Hans"));
    let tag_cleared_script = tag_hans.with_script_view(None);
    assert_eq!(tag_cleared_script.script(), None);
    assert!(tag_full.with_script(Some("toolongscript")).is_err());

    // 5. with_region & with_region_view
    let tag_no_region = tag_full.with_region(None).unwrap();
    assert_eq!(tag_no_region.region(), None);
    let tag_empty_region = tag_full.with_region(Some("")).unwrap();
    assert_eq!(tag_empty_region.region(), None);
    let tag_cn = tag_no_region.with_region_view(Some(RegionView::parse("CN").unwrap()));
    assert_eq!(tag_cn.region(), Some("CN"));
    let tag_cleared_region = tag_cn.with_region_view(None);
    assert_eq!(tag_cleared_region.region(), None);
    assert!(tag_full.with_region(Some("INVALID_REG")).is_err());

    // 6. without_extensions & with_extensions
    let tag_stripped = tag_full.without_extensions();
    assert_eq!(tag_stripped.to_string(), "en-Latn-US");
    assert_eq!(tag_stripped.extensions(), None);
    assert!(!tag_stripped.has_extensions());

    let tag_new_ext = tag_stripped.with_extensions(Some("a-custom")).unwrap();
    assert_eq!(tag_new_ext.to_string(), "en-Latn-US-a-custom");
    assert_eq!(tag_new_ext.extensions(), Some("a-custom"));
    assert!(tag_new_ext.has_extensions());

    let tag_ext_empty = tag_new_ext.with_extensions(Some("")).unwrap();
    assert_eq!(tag_ext_empty.extensions(), None);
    let tag_ext_none = tag_new_ext.with_extensions(None).unwrap();
    assert_eq!(tag_ext_none.extensions(), None);
    assert!(tag_new_ext.with_extensions(Some("invalid--ext")).is_err());

    // 7. Storage introspection
    assert_eq!(tag_full.extensions_max_capacity(), 32);
    assert_eq!(tag_full.extensions(), Some("u-ca-gregory"));
}

#[test]
fn test_normal_tag_ord_and_partial_ord() {
    let t1: NormalTag = NormalTag::parse("en-US").unwrap();
    let t2: NormalTag = NormalTag::parse("en-GB").unwrap();
    let t3: NormalTag = NormalTag::parse("zh-CN").unwrap();

    // GB < US
    assert_eq!(t2.cmp(&t1), Ordering::Less);
    assert_eq!(t1.cmp(&t2), Ordering::Greater);
    assert_eq!(t1.cmp(&t1), Ordering::Equal);
    assert_eq!(t1.cmp(&t3), Ordering::Less); // "en" < "zh"

    // View and Tag ordering
    let v1 = t1.as_view();
    let v2 = t2.as_view();
    assert_eq!(t1.partial_cmp(&v2), Some(Ordering::Greater));
    assert_eq!(v1.partial_cmp(&t2), Some(Ordering::Greater));
    assert_eq!(v1.partial_cmp(&t1), Some(Ordering::Equal));

    // FromStr and TryFrom
    let parsed_from_str: NormalTag = NormalTag::from_str("en-US").unwrap();
    assert_eq!(parsed_from_str, t1);
    let parsed_try_from: NormalTag = NormalTag::try_from("en-US").unwrap();
    assert_eq!(parsed_try_from, t1);
}

#[test]
fn test_u_extension_view_full_features() {
    let u_str = "u-attr-ca-gregory-nu-latn";
    let u_view = UExtensionView::parse(u_str).unwrap();

    assert_eq!(u_view.as_str(), "attr-ca-gregory-nu-latn");
    assert_eq!(u_view.to_string(), "u-attr-ca-gregory-nu-latn");

    // has_attribute & attributes iterator
    assert!(u_view.has_attribute("attr"));
    assert!(!u_view.has_attribute("missing"));
    let attrs: Vec<_> = u_view.attributes().collect();
    assert_eq!(attrs, vec!["attr"]);

    // has_key & keyword_type
    assert!(u_view.has_key("ca"));
    assert_eq!(u_view.keyword_type("ca"), Some("gregory"));
    assert!(u_view.has_key("nu"));
    assert_eq!(u_view.keyword_type("nu"), Some("latn"));
    assert_eq!(u_view.keyword_type("missing"), None);

    // keyword_types iterator
    let kw_types: Vec<_> = u_view.keyword_types("ca").collect();
    assert_eq!(kw_types, vec!["gregory"]);

    // keywords iterator
    let kws: Vec<_> = u_view.keywords().collect();
    assert_eq!(kws, vec![("ca", Some("gregory")), ("nu", Some("latn"))]);

    // TryFrom, Ord, PartialOrd
    let u_view2 = UExtensionView::try_from(u_str).unwrap();
    assert_eq!(u_view, u_view2);
    assert_eq!(u_view.cmp(&u_view2), Ordering::Equal);

    let u_view3 = UExtensionView::parse("u-ca-buddhist").unwrap();
    assert_eq!(u_view3.cmp(&u_view), Ordering::Greater); // "ca-buddhist" > "attr-ca-gregory-nu-latn"

    // Invalid syntax validation
    assert!(UExtensionView::parse("").is_err());
    assert!(UExtensionView::parse("u").is_err());
    assert!(UExtensionView::parse("u-").is_err());
    assert!(UExtensionView::parse("x-ca-gregory").is_err());
    assert!(UExtensionView::parse("u-ca-gregory-ca-duplicate").is_err());
}

#[test]
fn test_t_extension_view_full_features() {
    let t_str = "t-it-m0-ungegn-k0-dvorak";
    let t_view = TExtensionView::parse(t_str).unwrap();

    assert_eq!(t_view.as_str(), "it-m0-ungegn-k0-dvorak");
    assert_eq!(t_view.to_string(), "t-it-m0-ungegn-k0-dvorak");

    // tlang / source_language
    assert_eq!(t_view.source_language(), Some("it"));

    // has_field & field_value
    assert!(t_view.has_field("m0"));
    assert_eq!(t_view.field_value("m0"), Some("ungegn"));
    assert!(t_view.has_field("k0"));
    assert_eq!(t_view.field_value("k0"), Some("dvorak"));
    assert_eq!(t_view.field_value("missing"), None);

    // field_values iterator
    let f_vals: Vec<_> = t_view.field_values("m0").collect();
    assert_eq!(f_vals, vec!["ungegn"]);

    // fields iterator
    let fields: Vec<_> = t_view.fields().collect();
    assert_eq!(fields, vec![("m0", "ungegn"), ("k0", "dvorak")]);

    // TryFrom, Ord, PartialOrd
    let t_view2 = TExtensionView::try_from(t_str).unwrap();
    assert_eq!(t_view, t_view2);
    assert_eq!(t_view.cmp(&t_view2), Ordering::Equal);

    let t_view3 = TExtensionView::parse("t-en").unwrap();
    assert_eq!(t_view3.cmp(&t_view), Ordering::Less); // "en" < "it"

    // Invalid syntax validation
    assert!(TExtensionView::parse("").is_err());
    assert!(TExtensionView::parse("t").is_err());
    assert!(TExtensionView::parse("t-").is_err());
    assert!(TExtensionView::parse("x-it").is_err());
    assert!(TExtensionView::parse("t-it-m0-ungegn-m0-duplicate").is_err());
}

#[test]
fn test_variant_validation_and_extensions_helpers() {
    assert!(validate("rozaj").is_ok());
    assert!(validate("1996").is_ok());
    assert!(validate("123").is_err()); // starts with digit but shorter than 4 chars
    assert!(validate("abc").is_err()); // starts with letter but shorter than 5 chars
    assert!(validate("toolongvariantname").is_err()); // exceeds 8 chars
    assert!(validate("").is_err());
    assert!(is_valid("rozaj"));

    assert!(is_valid_bytes(b"rozaj"));
    assert!(!is_valid_bytes(b""));
    assert!(!is_valid_bytes(b"bad_char"));

    assert!(is_extension_subtag("subtag"));
    assert!(!is_extension_subtag("a")); // length < 2
    assert!(!is_extension_subtag("toolongsubtag")); // length > 8
}

#[test]
fn test_private_use_tag_full_methods() {
    let put: PrivateUseTag = PrivateUseTag::parse("x-my-data").unwrap();
    assert_eq!(put.to_string(), "x-my-data");
    assert_eq!(put.as_str(), "my-data");

    let subs: Vec<_> = put.subtags().collect();
    assert_eq!(subs, vec!["my", "data"]);

    // FromStr and TryFrom
    let put2: PrivateUseTag = PrivateUseTag::from_str("x-my-data").unwrap();
    assert_eq!(put, put2);
    let put3: PrivateUseTag = PrivateUseTag::try_from("x-my-data").unwrap();
    assert_eq!(put, put3);

    // parse_with
    let put_custom = PrivateUseTag::parse_with("x-my-data", Fixed::<64>::default()).unwrap();
    assert_eq!(put_custom.as_str(), "my-data");

    assert_eq!(put.max_capacity(), 32);
}

#[test]
fn test_storage_backends_edge_cases() {
    // 1. Fixed Storage
    let mut fixed = Fixed::<8>::default();
    assert_eq!(fixed.len(), 0);
    assert!(fixed.is_empty());
    assert_eq!(fixed.max_capacity(), 8);

    assert!(fixed.push_byte(b'a').is_ok());
    assert_eq!(fixed.len(), 1);
    assert!(!fixed.is_empty());
    assert_eq!(fixed.as_bytes(), b"a");

    assert!(fixed.push_bytes(b"bcde").is_ok());
    assert_eq!(fixed.as_bytes(), b"abcde");
    assert_eq!(fixed.len(), 5);

    // Overflow
    assert!(fixed.push_bytes(b"1234").is_err()); // 5 + 4 = 9 > 8
    fixed.clear();
    assert_eq!(fixed.len(), 0);
    assert!(fixed.is_empty());

    // 2. Grow Storage
    #[cfg(feature = "alloc")]
    {
        let mut grow = Grow::<16>::default();
        assert_eq!(grow.len(), 0);
        assert!(grow.is_empty());
        assert_eq!(grow.max_capacity(), 16);

        assert!(grow.push_byte(b'x').is_ok());
        assert_eq!(grow.as_bytes(), b"x");
        assert!(grow.push_bytes(b"1234567890").is_ok());
        assert_eq!(grow.len(), 11);

        // Overflow Grow
        assert!(grow.push_bytes(b"123456").is_err()); // 11 + 6 = 17 > 16
        grow.clear();
        assert_eq!(grow.len(), 0);
    }
}
