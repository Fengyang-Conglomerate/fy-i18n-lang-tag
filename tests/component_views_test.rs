//! Component views (LanguageView, ScriptView, RegionView, OtherExtensionView) and error coverage tests.

use core::cmp::Ordering;
use core::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

use fy_i18n_lang_tag::{
    CapacityError, LanguageView, OtherExtensionView, ParseError, RegionView, ScriptView, TagError,
};

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

#[test]
fn test_language_view_full_traits() {
    let lv1 = LanguageView::parse("zh").unwrap();
    let lv2 = LanguageView::try_from("ZH").unwrap();
    let lv3 = LanguageView::parse("en").unwrap();

    assert_eq!(lv1, lv2);
    assert_eq!(calculate_hash(&lv1), calculate_hash(&lv2));
    assert_ne!(lv1, lv3);

    assert_eq!(lv1.as_str(), "zh");
    assert_eq!(lv1.as_ref(), "zh");
    assert_eq!(&*lv1, "zh");
    assert_eq!(lv1.to_string(), "zh");

    assert_eq!(lv1.partial_cmp(&lv2), Some(Ordering::Equal));
    assert_eq!(lv1.cmp(&lv2), Ordering::Equal);
    assert_eq!(lv3.cmp(&lv1), Ordering::Less); // "en" < "zh"

    // Invalid languages
    assert_eq!(LanguageView::parse(""), Err(ParseError::Empty));
    assert_eq!(LanguageView::parse("a"), Err(ParseError::InvalidLanguage));
    assert_eq!(
        LanguageView::parse("toolonglang"),
        Err(ParseError::InvalidLanguage)
    );
    assert_eq!(LanguageView::parse("z1"), Err(ParseError::InvalidLanguage));
    assert!(LanguageView::try_from("123").is_err());
}

#[test]
fn test_script_view_full_traits() {
    let sv1 = ScriptView::parse("Hans").unwrap();
    let sv2 = ScriptView::try_from("hans").unwrap();
    let sv3 = ScriptView::parse("Hant").unwrap();

    assert_eq!(sv1, sv2);
    assert_eq!(calculate_hash(&sv1), calculate_hash(&sv2));
    assert_ne!(sv1, sv3);

    assert_eq!(sv1.as_str(), "Hans");
    assert_eq!(sv1.as_ref(), "Hans");
    assert_eq!(&*sv1, "Hans");
    assert_eq!(sv1.to_string(), "Hans");

    assert_eq!(sv1.partial_cmp(&sv2), Some(Ordering::Equal));
    assert_eq!(sv1.cmp(&sv2), Ordering::Equal);
    assert_eq!(sv1.cmp(&sv3), Ordering::Less); // "hans" < "hant"

    // Invalid scripts
    assert_eq!(ScriptView::parse(""), Err(ParseError::Empty));
    assert_eq!(ScriptView::parse("Han"), Err(ParseError::InvalidScript));
    assert_eq!(ScriptView::parse("Hansi"), Err(ParseError::InvalidScript));
    assert_eq!(ScriptView::parse("Ha1s"), Err(ParseError::InvalidScript));
    assert!(ScriptView::try_from("1234").is_err());
}

#[test]
fn test_region_view_full_traits() {
    let rv1 = RegionView::parse("CN").unwrap();
    let rv2 = RegionView::try_from("cn").unwrap();
    let rv3 = RegionView::parse("419").unwrap();
    let rv4 = RegionView::parse("US").unwrap();

    assert_eq!(rv1, rv2);
    assert_eq!(calculate_hash(&rv1), calculate_hash(&rv2));
    assert_ne!(rv1, rv3);

    assert_eq!(rv1.as_str(), "CN");
    assert_eq!(rv1.as_ref(), "CN");
    assert_eq!(&*rv1, "CN");
    assert_eq!(rv1.to_string(), "CN");
    assert_eq!(rv3.to_string(), "419");

    assert_eq!(rv1.partial_cmp(&rv2), Some(Ordering::Equal));
    assert_eq!(rv1.cmp(&rv2), Ordering::Equal);
    assert_eq!(rv1.cmp(&rv4), Ordering::Less); // "cn" < "us"

    // Invalid regions
    assert_eq!(RegionView::parse(""), Err(ParseError::Empty));
    assert_eq!(RegionView::parse("C"), Err(ParseError::InvalidRegion));
    assert_eq!(RegionView::parse("CHN"), Err(ParseError::InvalidRegion));
    assert_eq!(RegionView::parse("41"), Err(ParseError::InvalidRegion));
    assert_eq!(RegionView::parse("4190"), Err(ParseError::InvalidRegion));
    assert_eq!(RegionView::parse("C1"), Err(ParseError::InvalidRegion));
    assert!(RegionView::try_from("###").is_err());
}

#[test]
fn test_other_extension_view_full_traits() {
    let o1 = OtherExtensionView::parse("a-foo-bar").unwrap();
    let o2 = OtherExtensionView::try_from("A-FOO-BAR").unwrap();
    let o3 = OtherExtensionView::parse("b-baz").unwrap();

    assert_eq!(o1, o2);
    assert_eq!(calculate_hash(&o1), calculate_hash(&o2));
    assert_ne!(o1, o3);

    assert_eq!(o1.singleton(), 'a');
    assert_eq!(o1.singleton_byte(), b'a');
    assert_eq!(o1.subtags(), "foo-bar");
    assert_eq!(o1.as_str(), "foo-bar");
    assert_eq!(o1.to_string(), "a-foo-bar");

    assert_eq!(o1.partial_cmp(&o2), Some(Ordering::Equal));
    assert_eq!(o1.cmp(&o2), Ordering::Equal);
    assert_eq!(o1.cmp(&o3), Ordering::Less); // 'a' < 'b'

    // Compare same singleton with different subtags
    let o4 = OtherExtensionView::parse("a-aaa").unwrap();
    assert_eq!(o4.cmp(&o1), Ordering::Less); // "aaa" < "foo-bar"

    // Invalid parses
    assert!(OtherExtensionView::parse("").is_err());
    assert!(OtherExtensionView::parse("a").is_err());
    assert!(OtherExtensionView::parse("a-").is_err());
    assert!(OtherExtensionView::parse("x-foo").is_err()); // x is not other
    assert!(OtherExtensionView::parse("a-f").is_err()); // subtag at least 2 chars
    assert!(OtherExtensionView::parse("a-toolongsubtag").is_err()); // subtag at most 8 chars
    assert!(OtherExtensionView::parse("a-f@o").is_err());
}

#[test]
fn test_error_displays_and_conversions() {
    // ParseError display
    let errors = [
        ParseError::Empty,
        ParseError::TrailingSeparator,
        ParseError::EmptySubtag,
        ParseError::SubtagTooLong,
        ParseError::InvalidCharacter,
        ParseError::InvalidLanguage,
        ParseError::InvalidScript,
        ParseError::InvalidRegion,
        ParseError::InvalidVariant,
        ParseError::DuplicateVariant,
        ParseError::InvalidExtension,
        ParseError::DuplicateSingleton,
        ParseError::InvalidPrivateUse,
        ParseError::DuplicateUKey,
        ParseError::InvalidUType,
        ParseError::InvalidUAttribute,
        ParseError::InvalidTField,
        ParseError::DuplicateTField,
        ParseError::InvalidTLanguage,
    ];

    for err in &errors {
        let s = format!("{}", err);
        assert!(!s.is_empty(), "Error display cannot be empty: {:?}", err);
    }

    // TagError display and From
    let tag_err_parse: TagError = ParseError::Empty.into();
    assert!(format!("{}", tag_err_parse).contains("empty"));

    let cap = CapacityError::new(10, 5);
    let tag_err_cap: TagError = cap.into();
    assert!(format!("{}", tag_err_cap).contains("capacity exceeded"));

    // PrivateUseTag ParseError
    use fy_i18n_lang_tag::private_use::ParseError as PuError;
    let pu_errors = [
        PuError::Empty,
        PuError::InvalidPrefix,
        PuError::EmptySubtag,
        PuError::TrailingSeparator,
        PuError::SubtagTooLong,
        PuError::InvalidCharacter,
    ];
    for perr in &pu_errors {
        let s = format!("{}", perr);
        assert!(!s.is_empty());
        let converted: ParseError = (*perr).into();
        assert!(!format!("{}", converted).is_empty());
    }

    // PrivateUseTag TagError
    use fy_i18n_lang_tag::private_use::TagError as PuTagError;
    let pu_tag_err: PuTagError = PuError::Empty.into();
    assert!(!format!("{}", pu_tag_err).is_empty());
    let pu_tag_cap: PuTagError = cap.into();
    assert!(!format!("{}", pu_tag_cap).is_empty());
}
