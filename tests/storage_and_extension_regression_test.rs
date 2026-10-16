//! Regressions for byte storage, long tag indices, and extension boundaries.

use fy_i18n_lang_tag::{
    Fixed, NormalTag, NormalTagView, OtherExtensionView, PrivateUseTag, Storage,
};

fn bytes_backend<S: Storage>() {
    let mut bytes = S::default();
    bytes.write_bytes(&[0xff, b'A', 0, 0x80]).unwrap();
    assert_eq!(bytes.as_bytes(), &[0xff, b'A', 0, 0x80]);
    bytes.push_byte(b'Z').unwrap();
    assert_eq!(bytes.as_bytes(), &[0xff, b'A', 0, 0x80, b'Z']);
    let normal = NormalTag::parse_with("EN-u-CA-GREGORY", bytes).unwrap();
    assert_eq!(normal.extensions(), Some("u-ca-gregory"));
    assert_eq!(normal.to_string(), "en-u-ca-gregory");
    let mut bytes = S::default();
    bytes.push_bytes(&[0xff, 0xfe]).unwrap();
    let private = PrivateUseTag::parse_with("X-MY-DATA", bytes).unwrap();
    assert_eq!(private.as_str(), "my-data");
}

#[test]
fn arbitrary_bytes_and_validated_tag_storage_are_separate() {
    bytes_backend::<Fixed<32>>();
    #[cfg(feature = "alloc")]
    bytes_backend::<fy_i18n_lang_tag::Grow>();
}

fn capacity_errors_preserve_bytes<S: Storage>() {
    let mut s = S::default();
    s.write_bytes(b"ABC").unwrap();
    assert!(s.push_bytes(b"DE").is_err());
    assert_eq!(s.as_bytes(), b"ABC");
    assert!(s.write_bytes(b"ABCDE").is_err());
    assert_eq!(s.as_bytes(), b"ABC");
    s.push_byte(0xff).unwrap();
    assert!(s.push_byte(0).is_err());
    assert_eq!(s.as_bytes(), &[b'A', b'B', b'C', 0xff]);
}

#[test]
fn byte_capacity_errors_do_not_partially_write() {
    capacity_errors_preserve_bytes::<Fixed<4>>();
    #[cfg(feature = "alloc")]
    capacity_errors_preserve_bytes::<fy_i18n_lang_tag::Grow<4>>();
}

#[test]
fn other_extension_rejects_reserved_singletons_and_empty_subtags() {
    for input in [
        "a-foo-",
        "a--foo",
        "a-foo--bar",
        "a-",
        "a",
        "u-ca",
        "U-ca",
        "t-en",
        "T-en",
        "x-a",
        "X-a",
    ] {
        assert!(
            OtherExtensionView::parse(input).is_err(),
            "should reject: {input}"
        );
    }
    for input in ["a-foo", "A-FOO-bar", "0-ab", "z-abcdefgh"] {
        let ext = OtherExtensionView::parse(input).unwrap();
        let full = format!("en-{ext}");
        assert!(NormalTagView::parse(&full).is_ok());
    }
    // Excluding u/t/x is only for OtherExtensionView, not affecting full tag parsing.
    assert!(NormalTagView::parse("en-t-en-u-ca-x-a").is_ok());
}

fn assert_long_roundtrip<S: Storage>(input: &str) {
    use std::hash::{DefaultHasher, Hash, Hasher};
    let original = NormalTagView::parse(input).unwrap();
    let owned = NormalTag::<S>::parse(input).unwrap();
    let projected = owned.as_view();
    assert_eq!(projected, original);
    assert_eq!(projected.cmp(&original), core::cmp::Ordering::Equal);
    assert_eq!(owned.to_string(), original.to_string());
    let mut a = DefaultHasher::new();
    let mut b = DefaultHasher::new();
    owned.hash(&mut a);
    original.hash(&mut b);
    assert_eq!(a.finish(), b.finish());
}

#[test]
fn long_indices_preserve_each_extension_and_offsets() {
    for len in [65533, 65534, 65535, 65536, 65537] {
        let mut payload = "a-".repeat(len / 2);
        if len % 2 == 1 {
            payload.push('a');
        } else {
            payload.pop();
            payload.push_str("ab");
        }
        let input = format!("en-x-{payload}");
        assert_long_roundtrip::<Fixed<70000>>(&input);
        #[cfg(feature = "alloc")]
        assert_long_roundtrip::<fy_i18n_lang_tag::Grow>(&input);
    }
    // Single u/t extension crossing 16-bit boundaries, and u/t/x start after long preceding extensions.
    let types = vec!["abcdefgh"; 7300].join("-");
    for input in [
        format!("en-u-ca-{types}"),
        format!("en-t-m0-{types}"),
        format!("en-a-{types}-t-en-u-ca-gregory-x-last"),
    ] {
        assert_long_roundtrip::<Fixed<70000>>(&input);
        #[cfg(feature = "alloc")]
        assert_long_roundtrip::<fy_i18n_lang_tag::Grow>(&input);
    }
    let variants = (0..8200)
        .map(|i| format!("v{i:07}"))
        .collect::<Vec<_>>()
        .join("-");
    assert_long_roundtrip::<Fixed<80000>>(&format!("en-{variants}"));
}
