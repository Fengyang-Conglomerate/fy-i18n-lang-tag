//! Regression tests for review fixes.

use core::hash::{Hash, Hasher};
use fy_i18n_lang_tag::normal::{ParseError, RegionView, ScriptView, UExtensionView};
use fy_i18n_lang_tag::{
    Fixed, LanguageView, NormalTag, NormalTagView, PrivateUseTag, PrivateUseTagView, Storage,
    TagError,
};

/// Helper hash calculation
fn calculate_hash<T: Hash>(t: &T) -> u64 {
    struct SimpleHasher(u64);
    impl Hasher for SimpleHasher {
        fn finish(&self) -> u64 {
            self.0
        }
        fn write(&mut self, bytes: &[u8]) {
            for &b in bytes {
                self.0 = self.0.rotate_left(5) ^ (b as u64);
            }
        }
    }
    let mut s = SimpleHasher(0);
    t.hash(&mut s);
    s.finish()
}

#[test]
fn test_fix_p0_2_view_display_no_duplication_and_roundtrip() {
    // Composite tag previously triggering duplicate output: variants, other ext, u ext, t ext, private use
    let raw = "de-DE-1996-a-bb-t-m0-xyz-u-ca-gregory-x-priv";
    let view = NormalTagView::parse(raw).unwrap();
    let owned: NormalTag<Fixed<64>> = NormalTag::parse(raw).unwrap();

    let view_str = view.to_string();
    let owned_str = owned.to_string();

    // 1. View and owned output must be identical
    assert_eq!(view_str, owned_str);

    // 2. Output must be well-formed and roundtrip cleanly without DuplicateVariant or duplicate extensions
    let reparsed_view = NormalTagView::parse(&view_str).unwrap();
    let reparsed_owned: NormalTag<Fixed<64>> = NormalTag::parse(&owned_str).unwrap();
    assert_eq!(reparsed_view, view);
    assert_eq!(reparsed_owned, owned);

    // 3. Verify various extreme sample inputs
    let samples = [
        "en-fonipa-a-xy",
        "de-DE-a-bb-u-xx-yyy",
        "en-a-bb-x-q",
        "en-rozaj-a-foo-u-ca-gregory-x-bar",
    ];
    for s in samples {
        let v = NormalTagView::parse(s).unwrap();
        let o: NormalTag = NormalTag::parse(s).unwrap();
        assert_eq!(v.to_string(), o.to_string());
        assert!(NormalTagView::parse(&v.to_string()).is_ok());
    }
}

#[test]
fn test_fix_p1_1_parse_and_from_components_consistency() {
    let lang = LanguageView::parse("en").unwrap();
    let parsed: NormalTag = NormalTag::parse("en-t-it-u-ca").unwrap();
    let from_comp: NormalTag =
        NormalTag::from_components(lang, None, None, Some("u-ca-t-it")).unwrap();

    // Both paths must yield equivalent canonical output and matching hash
    assert_eq!(parsed, from_comp);
    assert_eq!(parsed.as_view(), from_comp.as_view());
    assert_eq!(calculate_hash(&parsed), calculate_hash(&from_comp));
    assert_eq!(parsed.to_string(), from_comp.to_string());

    // Storage with dirty data must be cleared
    let mut dirty_storage = Fixed::<32>::new();
    dirty_storage.write_bytes(b"garbage-data").unwrap();
    let clean: NormalTag =
        NormalTag::from_components_with(lang, None, None, None, dirty_storage).unwrap();
    assert_eq!(clean.extensions(), None);
    assert_eq!(clean.to_string(), "en");
}

#[test]
fn test_fix_p0_1_with_language_clears_old_bytes() {
    // 8-character long language tag
    let original: NormalTag = NormalTag::parse("abcdefgh-Hans-CN").unwrap();
    assert_eq!(original.language_str(), "abcdefgh");

    // Replace with 2-character short language "en"
    let short_lang = LanguageView::parse("en").unwrap();
    let derived = original.with_language(short_lang);

    // Verify derived tag is identical to parsed "en-Hans-CN" without leftover bytes
    let expected: NormalTag = NormalTag::parse("en-Hans-CN").unwrap();
    assert_eq!(derived, expected);
    assert_eq!(derived.language_str(), "en");
    assert_eq!(derived.script(), Some("Hans"));
    assert_eq!(derived.region(), Some("CN"));
    assert_eq!(calculate_hash(&derived), calculate_hash(&expected));
}

#[test]
fn test_fix_p1_3_t_extension_4alpha_language_and_stage_enforcement() {
    // 1. Supports 4ALPHA primary languages allowed by RFC 6497
    assert!(NormalTagView::parse("en-t-tzzz-m0-ungegn").is_ok());

    // 2. Reject script or region after variant phase in t-extension
    assert_eq!(
        NormalTagView::parse("en-t-sl-rozaj-Latn"),
        Err(ParseError::InvalidTLanguage)
    );
    assert_eq!(
        NormalTagView::parse("en-t-sl-rozaj-SI"),
        Err(ParseError::InvalidTLanguage)
    );

    // 3. Reject duplicate variants in t-extension
    assert_eq!(
        NormalTagView::parse("en-t-sl-rozaj-rozaj"),
        Err(ParseError::DuplicateVariant)
    );
}

#[test]
fn test_fix_p1_4_display_lowercase_normalization() {
    // Uppercase inputs must be normalized per specification
    let view = NormalTagView::parse("zh-HANS-cn-U-CA-GREGORY-T-M0-ABC").unwrap();
    assert_eq!(view.to_string(), "zh-Hans-CN-t-m0-abc-u-ca-gregory");

    let owned: NormalTag = NormalTag::parse("zh-HANS-cn-U-CA-GREGORY-T-M0-ABC").unwrap();
    assert_eq!(owned.to_string(), "zh-Hans-CN-t-m0-abc-u-ca-gregory");
}

#[test]
fn test_fix_p2_4_regular_grandfathered_behavior_boundary() {
    // Regular grandfathered tags matching primary language + variant (5-8 alphanum)
    let tag = NormalTagView::parse("art-lojban").unwrap();
    assert_eq!(tag.language_str(), "art");
    assert_eq!(tag.variants(), Some("lojban"));

    let tag2 = NormalTagView::parse("zh-guoyu").unwrap();
    assert_eq!(tag2.language_str(), "zh");
    assert_eq!(tag2.variants(), Some("guoyu"));

    // Grandfathered tags not conforming to modern syntax are rejected
    assert!(NormalTagView::parse("zh-min-nan").is_err());
    assert!(NormalTagView::parse("i-klingon").is_err());
    assert!(NormalTagView::parse("en-GB-oed").is_err());
}

#[cfg(feature = "serde")]
#[test]
fn test_fix_p1_2_serde_serialization_roundtrip() {
    use serde::Deserialize;
    use serde::de::value::{BorrowedStrDeserializer, StrDeserializer};

    let tag_str = "zh-Hans-CN-t-m0-abc-u-ca-gregory";

    // Owned deserialization
    let deserializer = StrDeserializer::<serde::de::value::Error>::new(tag_str);
    let tag: NormalTag = NormalTag::deserialize(deserializer).unwrap();
    assert_eq!(tag.to_string(), tag_str);

    // Borrowed deserialization
    let borrowed_de = BorrowedStrDeserializer::<serde::de::value::Error>::new(tag_str);
    let view = NormalTagView::deserialize(borrowed_de).unwrap();
    assert_eq!(view.to_string(), tag_str);

    // Borrowed deserializer rejects non-borrowed data sources
    struct NonBorrowedDeserializer<'a>(&'a str);
    impl<'de, 'a> serde::Deserializer<'de> for NonBorrowedDeserializer<'a> {
        type Error = serde::de::value::Error;
        fn deserialize_any<V: serde::de::Visitor<'de>>(
            self,
            visitor: V,
        ) -> Result<V::Value, Self::Error> {
            visitor.visit_str(self.0)
        }
        serde::forward_to_deserialize_any! {
            bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string
            bytes byte_buf option unit unit_struct newtype_struct seq tuple
            tuple_struct map struct enum identifier ignored_any
        }
    }

    let err = NormalTagView::deserialize(NonBorrowedDeserializer(tag_str)).unwrap_err();
    #[cfg(feature = "alloc")]
    assert!(
        err.to_string()
            .contains("borrowed views only support deserialization from zero-copy borrowed data")
    );
    #[cfg(not(feature = "alloc"))]
    let _ = err;
}

/// Round 2 P0-1: Verify Fixed storage residue does not affect PrivateUseTag equality or hash
#[test]
fn test_round2_fix_p0_storage_residue_isolation() {
    use fy_i18n_lang_tag::PrivateUseTag;

    let mut reused = Fixed::<32>::new();
    reused.write_bytes(b"longer-previous-content").unwrap();
    let a = PrivateUseTag::parse_with("x-a", reused).unwrap();
    let b = PrivateUseTag::<Fixed<32>>::parse("x-a").unwrap();

    // Equality, transitivity, and hash must remain completely consistent
    assert_eq!(a, b);
    assert_eq!(a.as_str(), b.as_str());
    assert_eq!(calculate_hash(&a), calculate_hash(&b));
    assert_eq!(a.cmp(&b), core::cmp::Ordering::Equal);
}

/// Round 2 P1-1: Verify OtherExtensionView pure subtags and truncation safety
#[test]
fn test_round2_fix_other_extension_pure_subtags_and_truncation_safety() {
    let input = "zh-u-ca-gregory-a-foo-t-it-m0-ungegn-b-bar-x-priv";
    let view = NormalTagView::parse(input).unwrap();
    assert!(view.has_other_extensions());

    // Query extension 'a': must be pure slice without 'a-' prefix
    let ext_a = view.other_extension('a').expect("must contain extension a");
    assert_eq!(ext_a.singleton(), 'a');
    assert_eq!(ext_a.singleton_byte(), b'a');
    assert_eq!(ext_a.subtags(), "foo");
    assert_eq!(ext_a.as_str(), "foo");
    assert_eq!(ext_a.to_string(), "a-foo");

    // Query extension 'b': must be pure slice
    let ext_b = view.other_extension('b').expect("must contain extension b");
    assert_eq!(ext_b.singleton(), 'b');
    assert_eq!(ext_b.subtags(), "bar");
    assert_eq!(ext_b.to_string(), "b-bar");

    // Non-ASCII character query: no u8 truncation collision
    assert_eq!(view.other_extension('š'), None);
    assert_eq!(view.other_extension('é'), None);
}

/// Round 2 P1-2: Verify all single-letter extensions sorted in ascending ASCII order per RFC 5646 §4.5
#[test]
fn test_round2_fix_extension_full_ascii_canonical_sort() {
    // 1. Extension y comes after u ('u' 0x75 < 'y' 0x79)
    let tag1 = NormalTagView::parse("en-u-ca-y-ab").unwrap();
    assert_eq!(tag1.to_string(), "en-u-ca-y-ab");
    let owned1 = NormalTag::<Fixed<64>>::from_view(tag1).unwrap();
    assert_eq!(owned1.to_string(), "en-u-ca-y-ab");

    // 2. Reverse input en-y-ab-u-ca normalized to en-u-ca-y-ab
    let tag2 = NormalTagView::parse("en-y-ab-u-ca").unwrap();
    assert_eq!(tag2.to_string(), "en-u-ca-y-ab");
    let owned2 = NormalTag::<Fixed<64>>::from_view(tag2).unwrap();
    assert_eq!(owned2.to_string(), "en-u-ca-y-ab");

    // 3. Generic extensions reverse order: b before a reordered to a before b
    let tag3 = NormalTagView::parse("en-b-bb-a-aa").unwrap();
    assert_eq!(tag3.to_string(), "en-a-aa-b-bb");
    let owned3 = NormalTag::<Fixed<64>>::from_view(tag3).unwrap();
    assert_eq!(owned3.to_string(), "en-a-aa-b-bb");

    // 4. Digit singleton and letter singleton: digit < letter ('1' 0x31 < '9' 0x39)
    let tag4 = NormalTagView::parse("en-9-aa-1-bb").unwrap();
    assert_eq!(tag4.to_string(), "en-1-bb-9-aa");

    // 5. Complex interleaved normalization
    let tag5 = NormalTagView::parse("en-w-ab-x2-t-m0-bcd-u-ca").unwrap();
    assert_eq!(tag5.to_string(), "en-t-m0-bcd-u-ca-w-ab-x2");
}

/// Round 2 P1-3: Verify 4ALPHA language production support and t-extension consistency
#[test]
fn test_round2_fix_4alpha_language_support_and_consistency() {
    // 4ALPHA primary language (e.g., CLDR root language, or reserved abcd)
    let lang = LanguageView::parse("root").unwrap();
    assert_eq!(lang.as_str(), "root");

    let tag = NormalTagView::parse("root-Latn-US-t-it-x-priv").unwrap();
    assert_eq!(tag.language_str(), "root");
    assert_eq!(tag.script(), Some("Latn"));
    assert_eq!(tag.region(), Some("US"));
    assert_eq!(tag.to_string(), "root-Latn-US-t-it-x-priv");
}

/// Round 3 P1-1: Verify equality and hash invariance regardless of input extension physical order
#[test]
fn test_round3_fix_other_extensions_order_independent_eq_and_hash() {
    let s1 = "en-b-bb-a-aa";
    let s2 = "en-a-aa-b-bb";

    let v1 = NormalTagView::parse(s1).unwrap();
    let v2 = NormalTagView::parse(s2).unwrap();

    // 1. View equality
    assert_eq!(v1, v2);
    assert_eq!(v2, v1);

    // 2. Hash consistency
    assert_eq!(calculate_hash(&v1), calculate_hash(&v2));

    // 3. Canonical Display equivalence
    assert_eq!(v1.to_string(), "en-a-aa-b-bb");
    assert_eq!(v2.to_string(), "en-a-aa-b-bb");

    // 4. Roundtrip parse fixed-point identity
    let v1_str = v1.to_string();
    let reparsed = NormalTagView::parse(&v1_str).unwrap();
    assert_eq!(v1, reparsed);

    // 5. Owned and borrowed cross-comparison
    let o1: NormalTag = NormalTag::parse(s1).unwrap();
    let o2: NormalTag = NormalTag::parse(s2).unwrap();
    assert_eq!(o1, o2);
    assert_eq!(o1, v1);
    assert_eq!(v1, o1);
    assert_eq!(o2, v2);
    assert_eq!(v2, o2);

    // 6. Complex multi-extension and digit singleton combinations
    let s_complex1 = "en-9-zz-1-aa-c-cc";
    let s_complex2 = "en-1-aa-c-cc-9-zz";
    let vc1 = NormalTagView::parse(s_complex1).unwrap();
    let vc2 = NormalTagView::parse(s_complex2).unwrap();
    assert_eq!(vc1, vc2);
    assert_eq!(calculate_hash(&vc1), calculate_hash(&vc2));
    assert_eq!(vc1.to_string(), "en-1-aa-9-zz-c-cc");
    let oc1: NormalTag<Fixed<64>> = NormalTag::parse(s_complex1).unwrap();
    assert_eq!(oc1, vc1);
}

/// Round 3 P2-3: Verify OtherExtensionView case-insensitive equality
#[test]
fn test_round3_fix_other_extension_case_insensitive_eq() {
    let v1 = NormalTagView::parse("en-A-FOO").unwrap();
    let v2 = NormalTagView::parse("en-a-foo").unwrap();

    let ext1 = v1.other_extension('a').unwrap();
    let ext2 = v2.other_extension('A').unwrap();

    assert_eq!(ext1, ext2);
    assert_eq!(calculate_hash(&ext1), calculate_hash(&ext2));
}

/// Round 3 P2-4: Verify PrivateUseTag ordering and view consistency
#[test]
fn test_round3_fix_private_use_tag_ord_consistency() {
    use fy_i18n_lang_tag::PrivateUseTag;

    let p1: PrivateUseTag = PrivateUseTag::parse("x-aaa").unwrap();
    let p2: PrivateUseTag = PrivateUseTag::parse("x-bbb").unwrap();

    let v1 = p1.as_view();
    let v2 = p2.as_view();

    assert_eq!(p1.cmp(&p2), v1.cmp(&v2));
    assert_eq!(p2.cmp(&p1), v2.cmp(&v1));
    assert_eq!(p1.cmp(&p1), core::cmp::Ordering::Equal);
}

/// Round 3: Explicit rejection of deprecated extlang syntax
#[test]
fn test_round3_fix_explicit_rejection_of_extlang() {
    // Extlang syntax is explicitly rejected in favor of primary languages
    assert!(NormalTagView::parse("zh-cmn").is_err());
    assert!(NormalTagView::parse("zh-cmn-Hans-CN").is_err());
    assert!(NormalTagView::parse("zh-yue-Hant-HK").is_err());
    assert!(NormalTagView::parse("ar-afb").is_err());
    assert!(NormalTagView::parse("sr-bsc").is_err());
}

/// Round 4: Verify without_extensions and with_extensions preserve storage configuration
#[test]
fn test_round4_fix_without_extensions_preserves_storage_instance() {
    let raw = "zh-Hans-CN-u-ca-gregory-t-zh-latn-x-priv";
    let tag = NormalTag::<Fixed<128>>::parse(raw).unwrap();

    let stripped = tag.without_extensions();
    assert_eq!(stripped.language_str(), "zh");
    assert_eq!(stripped.script(), Some("Hans"));
    assert_eq!(stripped.region(), Some("CN"));
    assert!(!stripped.has_extensions());

    let with_ext = stripped.with_extensions(Some("u-ca-chinese")).unwrap();
    assert_eq!(with_ext.to_string(), "zh-Hans-CN-u-ca-chinese");
    assert!(with_ext.has_extensions());
}

/// Round 4: Verify component Hash implementations write 0xFF delimiter
#[test]
fn test_round4_fix_component_hash_delimiter_distinction() {
    use fy_i18n_lang_tag::normal::{
        OtherExtensionView, RegionView, ScriptView, TExtensionView, UExtensionView,
    };
    use fy_i18n_lang_tag::{PrivateUseTag, PrivateUseTagView};

    struct ByteCaptureHasher {
        buf: [u8; 64],
        len: usize,
    }
    impl Hasher for ByteCaptureHasher {
        fn finish(&self) -> u64 {
            self.len as u64
        }
        fn write(&mut self, bytes: &[u8]) {
            for &b in bytes {
                if self.len < self.buf.len() {
                    self.buf[self.len] = b;
                    self.len += 1;
                }
            }
        }
    }

    let capture = |val: &dyn Fn(&mut ByteCaptureHasher)| -> ([u8; 64], usize) {
        let mut h = ByteCaptureHasher {
            buf: [0; 64],
            len: 0,
        };
        val(&mut h);
        (h.buf, h.len)
    };

    let lang = LanguageView::parse("zh").unwrap();
    let (buf, len) = capture(&|h| lang.hash(h));
    assert_eq!(&buf[..len], &[b'z', b'h', 0xFF]);

    let script = ScriptView::parse("Hans").unwrap();
    let (buf, len) = capture(&|h| script.hash(h));
    assert_eq!(&buf[..len], &[b'h', b'a', b'n', b's', 0xFF]);

    let region = RegionView::parse("CN").unwrap();
    let (buf, len) = capture(&|h| region.hash(h));
    assert_eq!(&buf[..len], &[b'c', b'n', 0xFF]);

    let priv_view = PrivateUseTagView::parse("x-priv").unwrap();
    let (buf, len) = capture(&|h| priv_view.hash(h));
    assert_eq!(buf[len - 1], 0xFF);

    // Verify owned PrivateUseTag delegates to view and writes 0xFF
    let priv_tag: PrivateUseTag = PrivateUseTag::parse("x-priv").unwrap();
    let (buf_tag, len_tag) = capture(&|h| priv_tag.hash(h));
    assert_eq!(buf_tag[len_tag - 1], 0xFF);
    assert_eq!(&buf[..len], &buf_tag[..len_tag]);

    let u_ext = UExtensionView::parse("u-ca-gregory").unwrap();
    let (buf, len) = capture(&|h| u_ext.hash(h));
    assert_eq!(buf[len - 1], 0xFF);

    let t_ext = TExtensionView::parse("t-zh-latn").unwrap();
    let (buf, len) = capture(&|h| t_ext.hash(h));
    assert_eq!(buf[len - 1], 0xFF);

    let other = OtherExtensionView::parse("a-foo").unwrap();
    let (buf, len) = capture(&|h| other.hash(h));
    assert_eq!(buf[len - 1], 0xFF);
}

#[test]
fn test_round5_fix_f02_capacity_error_required_total_length() {
    use fy_i18n_lang_tag::TagError;

    // Variants "1996" (4B) + u ext "u-ca-gregory" (12B) + private use "x-private" (9B)
    // Written to storage:
    // "1996" (4B)
    // + "-" + "u-ca-gregory" (1 + 12 = 13B)
    // + "-" + "x-private" (1 + 9 = 10B)
    // Total: 4 + 13 + 10 = 27 bytes
    let raw = "en-1996-u-ca-gregory-x-private";
    let view = NormalTagView::parse(raw).unwrap();

    // Storage capacity 16 bytes:
    let res = NormalTag::<Fixed<16>>::from_view(view);
    match res {
        Err(err) => {
            assert_eq!(
                err.required, 27,
                "CapacityError.required must be total required bytes"
            );
            assert_eq!(err.max_capacity, 16);
        }
        Ok(_) => panic!("should fail due to capacity"),
    }

    // Tested via NormalTag::parse_with
    let err = NormalTag::<Fixed<10>>::parse_with(raw, Fixed::<10>::default()).unwrap_err();
    match err {
        TagError::Capacity(c) => {
            assert_eq!(c.required, 27);
            assert_eq!(c.max_capacity, 10);
        }
        _ => panic!("should return TagError::Capacity"),
    }

    // Private use tag: "x-very-long-private-data" (without x- is 22 bytes)
    let priv_raw = "x-very-long-private-data";
    let priv_view = PrivateUseTagView::parse(priv_raw).unwrap();
    let priv_err = PrivateUseTag::<Fixed<8>>::from_view(priv_view).unwrap_err();
    assert_eq!(priv_err.required, 22);
    assert_eq!(priv_err.max_capacity, 8);
}

#[test]
fn test_debug_format_does_not_leak_internal_state() {
    let tag: NormalTag = NormalTag::parse("zh-Hans-CN-u-ca-gregory").unwrap();
    let dbg = format!("{tag:?}");
    assert_eq!(dbg, "NormalTag(\"zh-Hans-CN-u-ca-gregory\")");
    assert!(!dbg.contains("core"));
    assert!(!dbg.contains("indices"));
    assert!(!dbg.contains("Fixed"));

    let priv_tag: PrivateUseTag = PrivateUseTag::parse("x-my-data").unwrap();
    let priv_dbg = format!("{priv_tag:?}");
    assert_eq!(priv_dbg, "PrivateUseTag(\"x-my-data\")");
    assert!(!priv_dbg.contains("storage"));
    assert!(!priv_dbg.contains("Fixed"));
}

#[test]
fn test_from_components_consistency_and_validation() {
    // Normal case
    let lang = LanguageView::parse("zh").unwrap();
    let tag: NormalTag =
        NormalTag::from_components(lang, Some("Hans"), Some("CN"), Some("u-ca-gregory")).unwrap();
    assert_eq!(tag.language().as_str(), "zh");
    assert_eq!(tag.script(), Some("Hans"));
    assert_eq!(tag.region(), Some("CN"));
    assert!(tag.u().is_some());
    assert_eq!(tag.extensions(), Some("u-ca-gregory"));

    // Error case: invalid extension sequence
    let lang_en = LanguageView::parse("en").unwrap();
    let err = NormalTag::<Fixed<32>>::from_components(lang_en, None, None, Some("invalid--ext"))
        .unwrap_err();
    assert!(matches!(err, TagError::Parse(_)));
}

#[test]
fn test_hash_order_invariance_with_multiple_other_extensions() {
    // Multiple other extensions in different order
    let v1 = NormalTagView::parse("en-b-foo-a-bar-c-baz").unwrap();
    let v2 = NormalTagView::parse("en-c-baz-a-bar-b-foo").unwrap();
    assert_eq!(calculate_hash(&v1), calculate_hash(&v2));

    // Owned consistency
    let t1: NormalTag = NormalTag::parse("en-b-foo-a-bar-c-baz").unwrap();
    let t2: NormalTag = NormalTag::parse("en-c-baz-a-bar-b-foo").unwrap();
    assert_eq!(calculate_hash(&t1), calculate_hash(&t2));
    assert_eq!(calculate_hash(&t1), calculate_hash(&v1));
}

/// Regression test R2-01: Tiny storage capacity precheck reports full required length
#[test]
fn test_capacity_precheck_reports_full_length_even_with_tiny_storage() {
    let tag_str = "en-fonipa-a-foo-b-bar-x-priv";
    let view = NormalTagView::parse(tag_str).unwrap();

    // Fixed<3>: smaller than variant length 6
    let err_fixed3 = NormalTag::<Fixed<3>>::from_view(view).unwrap_err();
    assert_eq!(err_fixed3.max_capacity, 3);
    assert_eq!(err_fixed3.required, 25);

    // Fixed<0>: zero capacity
    let err_fixed0 = NormalTag::<Fixed<0>>::from_view(view).unwrap_err();
    assert_eq!(err_fixed0.max_capacity, 0);
    assert_eq!(err_fixed0.required, 25);

    // from_view_with consistency
    let err_with = NormalTag::from_view_with(view, Fixed::<3>::new()).unwrap_err();
    assert_eq!(err_with.required, 25);
    assert_eq!(err_with.max_capacity, 3);
}

/// Regression test R2-02: NormalTagView::hash private-use segment prefix-freedom
#[test]
fn test_hash_prefix_freedom_between_tag_with_and_without_private_use() {
    struct ByteRecordingHasher(std::vec::Vec<u8>);
    impl Hasher for ByteRecordingHasher {
        fn finish(&self) -> u64 {
            0
        }
        fn write(&mut self, bytes: &[u8]) {
            self.0.extend_from_slice(bytes);
        }
        fn write_u8(&mut self, i: u8) {
            self.0.push(i);
        }
    }

    let tag_without = NormalTagView::parse("en").unwrap();
    let tag_with = NormalTagView::parse("en-x-ab").unwrap();

    let mut h1 = ByteRecordingHasher(std::vec::Vec::new());
    tag_without.hash(&mut h1);

    let mut h2 = ByteRecordingHasher(std::vec::Vec::new());
    tag_with.hash(&mut h2);

    // h1 ends with [0xFF, 0x00]
    let len1 = h1.0.len();
    assert!(len1 >= 2);
    assert_eq!(h1.0[len1 - 2], 0xFF);
    assert_eq!(h1.0[len1 - 1], 0x00);

    // h2 ends with [0xFF, 0x01, ...]
    assert!(h2.0.len() > len1);
    assert_eq!(h2.0[len1 - 2], 0xFF);
    assert_eq!(h2.0[len1 - 1], 0x01);

    // Verify h1 byte sequence is never a strict prefix of h2
    assert_ne!(&h1.0[..], &h2.0[..len1]);
}

/// Regression test R2-04: Open-addressing variant deduplication and t-extension variant detection
#[test]
fn test_open_addressing_variant_deduplication_and_t_extension_variants() {
    // 1. Multiple valid variants without duplicates
    let multi_var = NormalTagView::parse("sl-rozaj-biske-1994").unwrap();
    assert_eq!(multi_var.variants(), Some("rozaj-biske-1994"));

    // 2. Duplicate variant returns DuplicateVariant error
    assert_eq!(
        NormalTagView::parse("sl-rozaj-biske-rozaj"),
        Err(ParseError::DuplicateVariant)
    );

    // 3. Variants following script and region inside t-extension
    let t_ok = NormalTagView::parse("en-t-it-Latn-IT-1abc").unwrap();
    assert!(t_ok.t().is_some());

    // 4. Duplicate variant inside t-extension returns DuplicateVariant error
    assert_eq!(
        NormalTagView::parse("en-t-it-Latn-IT-1abc-1abc"),
        Err(ParseError::DuplicateVariant)
    );
}

/// Regression test R3-01: Data-driven verification of 19 error cases from error mapping table
#[test]
fn test_specification_error_mapping_table_consistency() {
    // 1. Full tag entry NormalTagView::parse
    assert_eq!(NormalTagView::parse(""), Err(ParseError::Empty));
    assert_eq!(NormalTagView::parse("zh--CN"), Err(ParseError::EmptySubtag));
    assert_eq!(
        NormalTagView::parse("zh-"),
        Err(ParseError::TrailingSeparator)
    );
    assert_eq!(
        NormalTagView::parse("en-x-"),
        Err(ParseError::TrailingSeparator)
    );
    assert_eq!(
        NormalTagView::parse("en-a-123456789"),
        Err(ParseError::SubtagTooLong)
    );
    assert_eq!(
        NormalTagView::parse("en-!"),
        Err(ParseError::InvalidCharacter)
    );
    assert_eq!(
        NormalTagView::parse("-en"),
        Err(ParseError::InvalidLanguage)
    );
    assert_eq!(
        NormalTagView::parse("--en"),
        Err(ParseError::InvalidLanguage)
    );
    assert_eq!(
        NormalTagView::parse("i-klingon"),
        Err(ParseError::InvalidLanguage)
    );
    assert_eq!(
        NormalTagView::parse("zh-cmn"),
        Err(ParseError::InvalidVariant)
    );
    assert_eq!(
        NormalTagView::parse("zh-min-nan"),
        Err(ParseError::InvalidVariant)
    );
    assert_eq!(
        NormalTagView::parse("en-GB-oed"),
        Err(ParseError::InvalidVariant)
    );
    assert_eq!(
        NormalTagView::parse("en-fonipa-fonipa"),
        Err(ParseError::DuplicateVariant)
    );
    assert_eq!(
        NormalTagView::parse("en-a"),
        Err(ParseError::InvalidExtension)
    );
    assert_eq!(
        NormalTagView::parse("en-a-b"),
        Err(ParseError::InvalidExtension)
    );
    assert_eq!(
        NormalTagView::parse("en-a-foo-a-bar"),
        Err(ParseError::DuplicateSingleton)
    );
    assert_eq!(
        NormalTagView::parse("en-x"),
        Err(ParseError::InvalidPrivateUse)
    );
    assert_eq!(
        NormalTagView::parse("en-u-ca-gregory-ca-buddhist"),
        Err(ParseError::DuplicateUKey)
    );
    assert_eq!(
        NormalTagView::parse("en-t-m0"),
        Err(ParseError::InvalidTField)
    );
    assert_eq!(
        NormalTagView::parse("en-t-m0-ab"),
        Err(ParseError::InvalidTField)
    );
    assert_eq!(
        NormalTagView::parse("en-t-m0-abc-m0-def"),
        Err(ParseError::DuplicateTField)
    );
    assert_eq!(
        NormalTagView::parse("en-t-1a"),
        Err(ParseError::InvalidTLanguage)
    );

    // 2. Component and subtag entries
    assert_eq!(ScriptView::parse("Lat"), Err(ParseError::InvalidScript));
    assert_eq!(RegionView::parse("1"), Err(ParseError::InvalidRegion));
    assert_eq!(RegionView::parse("USA"), Err(ParseError::InvalidRegion));
    assert_eq!(
        UExtensionView::from_subtags("ca-a"),
        Err(ParseError::InvalidUType)
    );
    assert_eq!(
        UExtensionView::from_subtags("a"),
        Err(ParseError::InvalidUAttribute)
    );
}

/// Regression test R3-02: Hash slot dispersion and full-table bypass for same-prefix variants
#[test]
fn test_variant_open_addressing_hash_dispersion_and_bypass() {
    use std::collections::BTreeSet;

    // 128 valid variants starting with 'v': v0000000 to v0000127
    let mut slots = BTreeSet::new();
    let mut tag_parts = std::vec::Vec::new();
    tag_parts.push("en".to_string());

    for i in 0..128 {
        let v = format!("v{:07}", i);
        tag_parts.push(v.clone());

        let mut buf = [0u8; 8];
        let bytes = v.as_bytes();
        let limit = bytes.len().min(8);
        buf[..limit].copy_from_slice(&bytes[..limit]);
        let packed = u64::from_ne_bytes(buf);

        let mut h = (packed ^ 0xcbf2_9ce4_8422_2325).wrapping_mul(0x517c_c1b7_2722_0a95);
        h ^= h >> 32;
        h = h.wrapping_mul(0x9e37_79b9_7f4a_7c15);
        h ^= h >> 28;
        let slot = (h as usize) % 128;
        slots.insert(slot);
    }

    // Verify slot distribution dispersion covers at least 60 distinct slots
    assert!(
        slots.len() >= 60,
        "variant slots should be well dispersed, got only {} distinct slots",
        slots.len()
    );

    // Verify long tag with 128 variants parses successfully
    let big_tag = tag_parts.join("-");
    let view = NormalTagView::parse(&big_tag).expect("128 variants tag must parse cleanly");
    assert!(view.variants().is_some());

    // Verify adding 129th variant succeeds via linear bypass, while duplicate is intercepted
    let mut tag_129 = big_tag.clone();
    tag_129.push_str("-v0000128");
    assert!(NormalTagView::parse(&tag_129).is_ok());

    let mut dup_tag = tag_129;
    dup_tag.push_str("-v0000000"); // Duplicate of 1st variant
    assert_eq!(
        NormalTagView::parse(&dup_tag),
        Err(ParseError::DuplicateVariant)
    );
}
