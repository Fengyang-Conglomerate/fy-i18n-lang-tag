use core::cmp::Ordering;

use fy_i18n_lang_tag::normal::extensions::{is_singleton, validate_singleton, validate_subtag};
use fy_i18n_lang_tag::normal::language::{self, LanguageView};
use fy_i18n_lang_tag::normal::matching::{MatchQuality, eval_match_quality};
use fy_i18n_lang_tag::normal::region::{self, RegionView};
use fy_i18n_lang_tag::normal::script::{self, ScriptView};
use fy_i18n_lang_tag::normal::view::NormalTagView;
use fy_i18n_lang_tag::storage::{Fixed, Storage};
use fy_i18n_lang_tag::{ParseError, PrivateUseTag, PrivateUseTagView};

#[cfg(feature = "alloc")]
use fy_i18n_lang_tag::Grow;

#[test]
fn test_storage_traits_and_fixed_ord_hash() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // 1. Fixed Eq, Ord, Hash, len, is_empty
    let mut f1 = Fixed::<16>::default();
    let mut f2 = Fixed::<16>::default();
    assert_eq!(f1, f2);
    assert_eq!(f1.partial_cmp(&f2), Some(Ordering::Equal));
    assert_eq!(f1.cmp(&f2), Ordering::Equal);

    let mut h1 = DefaultHasher::new();
    let mut h2 = DefaultHasher::new();
    f1.hash(&mut h1);
    f2.hash(&mut h2);
    assert_eq!(h1.finish(), h2.finish());

    f1.write_bytes(b"abc").unwrap();
    f2.write_bytes(b"abd").unwrap();
    assert!(f1 < f2);
    assert_eq!(f1.len(), 3);
    assert!(!f1.is_empty());

    // 3. Grow push_byte overflow
    #[cfg(feature = "alloc")]
    {
        let mut g = Grow::<1>::default();
        assert!(g.push_byte(b'a').is_ok());
        assert!(g.push_byte(b'b').is_err());
    }
}

#[test]
fn test_language_script_region_standalone_validators_and_eq() {
    // Language
    assert!(language::is_valid("zh"));
    assert!(!language::is_valid("12"));
    assert!(language::validate("zh").is_ok());
    assert_eq!(language::validate(""), Err(ParseError::Empty));
    assert_eq!(
        language::validate("invalid-lang"),
        Err(ParseError::InvalidLanguage)
    );

    let lv = LanguageView::parse("zh").unwrap();
    assert!(!lv.is_empty());
    assert_eq!(lv.as_str(), "zh");
    assert_eq!(lv, LanguageView::parse("zh").unwrap());
    let lv_upper = LanguageView::parse("ZH").unwrap();
    assert_eq!(lv, lv_upper);

    // Script
    assert!(script::is_valid("Hans"));
    assert!(!script::is_valid("Han1"));
    assert!(script::validate("Hans").is_ok());
    assert_eq!(script::validate(""), Err(ParseError::Empty));
    assert_eq!(script::validate("Han"), Err(ParseError::InvalidScript));

    let sv = ScriptView::parse("Hans").unwrap();
    assert!(!sv.is_empty());
    assert_eq!(sv.as_str(), "Hans");
    assert_eq!(sv, ScriptView::parse("Hans").unwrap());
    let sv_lower = ScriptView::parse("hans").unwrap();
    assert_eq!(sv, sv_lower);

    // Region
    assert!(region::is_valid("CN"));
    assert!(region::is_valid("419"));
    assert!(!region::is_valid("C1"));
    assert!(region::validate("CN").is_ok());
    assert_eq!(region::validate(""), Err(ParseError::Empty));
    assert_eq!(region::validate("C"), Err(ParseError::InvalidRegion));

    let rv = RegionView::parse("CN").unwrap();
    assert!(!rv.is_empty());
    assert_eq!(rv.as_str(), "CN");
    assert_eq!(rv, RegionView::parse("CN").unwrap());
    let rv_lower = RegionView::parse("cn").unwrap();
    assert_eq!(rv, rv_lower);
}

#[test]
fn test_extensions_standalone_validators() {
    assert!(is_singleton(b'a'));
    assert!(!is_singleton(b'x'));
    assert!(!is_singleton(b'-'));

    assert!(validate_singleton(b'a').is_ok());
    assert_eq!(validate_singleton(b'x'), Err(ParseError::InvalidExtension));

    assert!(validate_subtag("subtag").is_ok());
    assert_eq!(validate_subtag(""), Err(ParseError::EmptySubtag));
    assert_eq!(validate_subtag("a"), Err(ParseError::InvalidExtension));
    assert_eq!(
        validate_subtag("toolongsubtag"),
        Err(ParseError::SubtagTooLong)
    );
    assert_eq!(validate_subtag("f@o"), Err(ParseError::InvalidCharacter));
}

#[test]
fn test_normal_tag_view_eq_and_cmp_all_branches() {
    // 1. Language mismatch
    let v_en = NormalTagView::parse("en").unwrap();
    let v_zh = NormalTagView::parse("zh").unwrap();
    assert_ne!(v_en, v_zh);
    assert_eq!(v_en.cmp(&v_zh), Ordering::Less);

    // 2. Script mismatch & presence
    let v_hans = NormalTagView::parse("zh-Hans").unwrap();
    let v_hant = NormalTagView::parse("zh-Hant").unwrap();
    let v_no_script = NormalTagView::parse("zh").unwrap();
    assert_ne!(v_hans, v_hant);
    assert_ne!(v_hans, v_no_script);
    assert_eq!(v_hans.cmp(&v_hant), Ordering::Less); // "Hans" < "Hant"
    assert_eq!(v_hans.cmp(&v_no_script), Ordering::Greater);
    assert_eq!(v_no_script.cmp(&v_hans), Ordering::Less);
    assert_eq!(v_no_script.script_view(), None);

    // 3. Region mismatch & presence
    let v_cn = NormalTagView::parse("zh-CN").unwrap();
    let v_tw = NormalTagView::parse("zh-TW").unwrap();
    let v_no_reg = NormalTagView::parse("zh").unwrap();
    assert_ne!(v_cn, v_tw);
    assert_ne!(v_cn, v_no_reg);
    assert_eq!(v_cn.cmp(&v_tw), Ordering::Less); // "CN" < "TW"
    assert_eq!(v_cn.cmp(&v_no_reg), Ordering::Greater);
    assert_eq!(v_no_reg.cmp(&v_cn), Ordering::Less);
    assert_eq!(v_no_reg.region_view(), None);

    // 4. Variants mismatch & presence
    let v_var1 = NormalTagView::parse("sl-rozaj").unwrap();
    let v_var2 = NormalTagView::parse("sl-nedis").unwrap();
    let v_no_var = NormalTagView::parse("sl").unwrap();
    assert_ne!(v_var1, v_var2);
    assert_ne!(v_var1, v_no_var);
    assert_eq!(v_var2.cmp(&v_var1), Ordering::Less); // "nedis" < "rozaj"
    assert_eq!(v_var1.cmp(&v_no_var), Ordering::Greater);
    assert_eq!(v_no_var.cmp(&v_var1), Ordering::Less);

    // 5. Extensions mismatch (u, t, other)
    let v_u1 = NormalTagView::parse("en-u-ca-gregory").unwrap();
    let v_u2 = NormalTagView::parse("en-u-ca-buddhist").unwrap();
    let v_no_u = NormalTagView::parse("en").unwrap();
    assert_ne!(v_u1, v_u2);
    assert_ne!(v_u1, v_no_u);

    let v_t1 = NormalTagView::parse("en-t-it").unwrap();
    let v_t2 = NormalTagView::parse("en-t-de").unwrap();
    assert_ne!(v_t1, v_t2);

    let v_o1 = NormalTagView::parse("en-a-foo").unwrap();
    let v_o2 = NormalTagView::parse("en-a-bar").unwrap();
    let v_o3 = NormalTagView::parse("en-b-foo").unwrap();
    let v_o_multi = NormalTagView::parse("en-a-foo-b-bar").unwrap();
    assert_ne!(v_o1, v_o2);
    assert_ne!(v_o1, v_o3);
    assert_ne!(v_o1, v_o_multi);
    assert_eq!(v_o2.cmp(&v_o1), Ordering::Less); // "bar" < "foo"
    assert_eq!(v_o1.cmp(&v_o3), Ordering::Less); // 'a' < 'b'
    assert_eq!(v_o1.cmp(&v_o_multi), Ordering::Less); // 1 block < 2 blocks

    // 6. Private use mismatch
    let v_p1 = NormalTagView::parse("en-x-aaa").unwrap();
    let v_p2 = NormalTagView::parse("en-x-bbb").unwrap();
    let v_no_p = NormalTagView::parse("en").unwrap();
    assert_ne!(v_p1, v_p2);
    assert_ne!(v_p1, v_no_p);
    assert_eq!(v_p1.cmp(&v_p2), Ordering::Less);
    assert_eq!(v_p1.cmp(&v_no_p), Ordering::Greater);
    assert_eq!(v_no_p.cmp(&v_p1), Ordering::Less);
}

#[test]
fn test_private_use_view_and_tag_edge_cases() {
    let pv = PrivateUseTagView::parse("x-subtag").unwrap();
    assert!(!pv.is_empty());
    assert_eq!(pv.len(), 6);
    assert_eq!(pv.subtags().count(), 1);

    // PrivateUseTag with_storage & from_subtags
    let p_owned: PrivateUseTag<Fixed<32>> = PrivateUseTag::from_subtags("subtag").unwrap();
    assert_eq!(p_owned.to_string(), "x-subtag");
    assert_eq!(p_owned.as_view(), pv);
    assert_eq!(p_owned, pv);
    assert_eq!(pv, p_owned);
    assert_eq!(p_owned.cmp(&p_owned), Ordering::Equal);
    assert_eq!(pv.cmp(&pv), Ordering::Equal);
}

#[test]
fn test_matching_edge_cases() {
    // Candidate has script while request does not
    let req_bare = NormalTagView::parse("zh").unwrap();
    let cand_script = NormalTagView::parse("zh-Hans").unwrap();
    assert_eq!(
        eval_match_quality(&req_bare, &cand_script),
        MatchQuality::Language
    );

    // Candidate has region while request does not
    let cand_region = NormalTagView::parse("zh-CN").unwrap();
    assert_eq!(
        eval_match_quality(&req_bare, &cand_region),
        MatchQuality::Language
    );

    // Variant difference does not affect core matching
    let req_var = NormalTagView::parse("sl-rozaj").unwrap();
    let cand_var_diff = NormalTagView::parse("sl-nedis").unwrap();
    assert_eq!(
        eval_match_quality(&req_var, &cand_var_diff),
        MatchQuality::CoreExact
    );

    // Primary language mismatch yields NoMatch
    assert_eq!(
        eval_match_quality(&req_bare, &req_var),
        MatchQuality::NoMatch
    );
}

#[test]
fn test_exhaustive_remaining_coverage() {
    use fy_i18n_lang_tag::normal::extensions::parse_and_validate_extensions_sequence;
    use fy_i18n_lang_tag::normal::{t, u, variant};
    use fy_i18n_lang_tag::{NormalTag, TExtensionView, UExtensionView};
    use std::collections::hash_map::DefaultHasher;
    use std::hash::Hash;

    // 1. ScriptView & RegionView len
    let sv = ScriptView::parse("Hans").unwrap();
    assert_eq!(sv.len(), 4);
    let rv = RegionView::parse("CN").unwrap();
    assert_eq!(rv.len(), 2);

    // 2. NormalTagView extensions() & has_extensions() & try_from
    let v_bare = NormalTagView::parse("zh").unwrap();
    assert!(!v_bare.has_extensions());
    assert_eq!(v_bare.extensions(), None);

    let v_ext = NormalTagView::try_from("zh-Hans-CN-a-foo-b-bar-x-priv").unwrap();
    assert!(v_ext.has_extensions());
    assert!(v_ext.extensions().is_some());

    // NormalTagView Hash with variants & private use
    let mut h = DefaultHasher::new();
    NormalTagView::parse("sl-rozaj").unwrap().hash(&mut h);
    NormalTagView::parse("en-x-priv").unwrap().hash(&mut h);
    NormalTagView::parse("en-a-foo").unwrap().hash(&mut h);

    // NormalTagView PartialOrd
    assert_eq!(v_bare.partial_cmp(&v_ext), Some(Ordering::Less));

    // NormalTagView cmp with prefix variants of different lengths
    let v_v1 = NormalTagView::parse("sl-rozaj").unwrap();
    let v_v2 = NormalTagView::parse("sl-rozaj-biske").unwrap();
    assert_eq!(v_v1.cmp(&v_v2), Ordering::Less);

    // NormalTagView cmp with u, t, other extension interleaving and swapping
    let v_ut1 = NormalTagView::parse("zh-u-ca-buddhist-t-it-a-foo").unwrap();
    let v_ut2 = NormalTagView::parse("zh-a-foo-t-it-u-ca-buddhist").unwrap();
    assert_eq!(v_ut1.cmp(&v_ut2), Ordering::Equal);
    assert_eq!(v_ut1, v_ut2);

    let v_t_only = NormalTagView::parse("zh-t-it").unwrap();
    let v_u_only = NormalTagView::parse("zh-u-ca-buddhist").unwrap();
    assert_eq!(v_t_only.cmp(&v_u_only), Ordering::Less); // 't' < 'u'

    // has_other equality check
    let v_other = NormalTagView::parse("zh-a-foo").unwrap();
    assert_ne!(v_other, v_bare);

    // 3. NormalTag methods & PartialOrd cross-storage
    let nt = NormalTag::<Fixed<64>>::parse("zh").unwrap();
    let nt2 = nt.with_script(Some("Hans")).unwrap();
    assert_eq!(nt2.script(), Some("Hans"));
    let nt3 = nt2.with_region(Some("CN")).unwrap();
    assert_eq!(nt3.region(), Some("CN"));
    let nt4 = nt3.with_script_view(None);
    assert_eq!(nt4.script(), None);

    let nt_fixed: NormalTag<Fixed<32>> = NormalTag::parse("zh").unwrap();
    let nt_fixed_other: NormalTag<Fixed<64>> = NormalTag::parse("en").unwrap();
    assert_eq!(
        nt_fixed.partial_cmp(&nt_fixed_other),
        Some(Ordering::Greater)
    );

    // 4. PrivateUseTag & PrivateUseTagView
    let pt = PrivateUseTag::<Fixed<32>>::parse("x-subtag").unwrap();
    assert_eq!(pt.max_capacity(), 32);
    assert_eq!(pt.as_ref(), "subtag");
    assert_eq!(&*pt, "subtag");

    let pt2: PrivateUseTag<Fixed<64>> = PrivateUseTag::parse("x-other").unwrap();
    assert_eq!(pt.partial_cmp(&pt2), Some(Ordering::Greater));

    let pv1 = PrivateUseTagView::parse("x-aaa").unwrap();
    let pv2 = PrivateUseTagView::parse("x-bbb").unwrap();
    assert_eq!(pv1.partial_cmp(&pv2), Some(Ordering::Less));

    assert!(!fy_i18n_lang_tag::private_use::is_subtag_bytes(b""));
    assert!(!fy_i18n_lang_tag::private_use::is_subtag_bytes(
        b"123456789"
    ));
    assert!(!fy_i18n_lang_tag::private_use::is_subtag_bytes(b"ab_c"));
    assert!(PrivateUseTagView::parse("x-abc-").is_err());

    // 5. Variant subtag
    assert!(!variant::is_valid_bytes(b"1@34"));
    assert!(variant::is_valid_bytes(b"1234"));

    // 6. Extensions parsing errors
    assert!(matches!(
        parse_and_validate_extensions_sequence(b""),
        Err(ParseError::EmptySubtag)
    ));
    assert!(matches!(
        parse_and_validate_extensions_sequence(b"a-bbb-"),
        Err(ParseError::TrailingSeparator)
    ));
    assert!(matches!(
        parse_and_validate_extensions_sequence(b"x"),
        Err(ParseError::InvalidPrivateUse)
    ));
    assert!(matches!(
        parse_and_validate_extensions_sequence(b"#"),
        Err(ParseError::InvalidCharacter)
    ));
    assert!(matches!(
        parse_and_validate_extensions_sequence(b"a-u"),
        Err(ParseError::InvalidExtension)
    ));
    assert!(matches!(
        parse_and_validate_extensions_sequence(b"a-123456789-b-foo"),
        Err(ParseError::SubtagTooLong)
    ));
    assert!(matches!(
        parse_and_validate_extensions_sequence(b"a-foo-x"),
        Err(ParseError::InvalidPrivateUse)
    ));
    assert!(matches!(
        parse_and_validate_extensions_sequence(b"a-foo-#"),
        Err(ParseError::InvalidCharacter)
    ));
    assert!(matches!(
        parse_and_validate_extensions_sequence(b"x-123456789"),
        Err(ParseError::SubtagTooLong)
    ));
    assert!(matches!(
        parse_and_validate_extensions_sequence(b"x-f@o"),
        Err(ParseError::InvalidPrivateUse)
    ));
    assert!(matches!(
        parse_and_validate_extensions_sequence(b"a-foo--bar"),
        Err(ParseError::EmptySubtag)
    ));

    // Extension and variant differences only affect Exact equality
    let req_ut1 = NormalTagView::parse("sl-rozaj-u-ca-gregory").unwrap();
    let cand_ut2 = NormalTagView::parse("sl-rozaj-u-ca-buddhist").unwrap();
    assert_eq!(
        eval_match_quality(&req_ut1, &cand_ut2),
        MatchQuality::CoreExact
    );

    let req_sl = NormalTagView::parse("sl").unwrap();
    let cand_sl_var = NormalTagView::parse("sl-rozaj").unwrap();
    assert_eq!(
        eval_match_quality(&req_sl, &cand_sl_var),
        MatchQuality::CoreExact
    );

    // NormalTagView block swapping with 'z' singleton
    let v_swap1 = NormalTagView::parse("zh-z-foo-t-it").unwrap();
    let v_swap2 = NormalTagView::parse("zh-t-it-z-foo").unwrap();
    assert_eq!(v_swap1.cmp(&v_swap2), Ordering::Equal);

    // 7. u extension validators & methods
    assert!(u::is_key("ca"));
    assert!(!u::is_key("c"));
    assert!(!u::is_key("caa"));
    assert!(u::is_key_bytes(b"ca"));
    assert!(!u::is_key_bytes(b"c"));
    assert!(!u::is_key_bytes(b"caa"));
    assert!(!u::is_alphanum_3_8_bytes(b"ab"));
    assert!(!u::is_alphanum_3_8_bytes(b"123456789"));
    assert!(!u::is_alphanum_3_8_bytes(b"ab_c"));
    assert!(u::is_attribute("foobar"));
    assert!(u::is_type("buddhist"));
    assert!(u::validate("ca-buddhist").is_ok());
    assert!(u::validate("u-ca-buddhist").is_ok());
    assert_eq!(
        u::validate_subtags_bytes(b"ca--buddhist"),
        Err(ParseError::EmptySubtag)
    );
    assert_eq!(
        u::validate_subtags_bytes(b"toolongattribute"),
        Err(ParseError::SubtagTooLong)
    );

    let uv1 = UExtensionView::parse("ca-buddhist").unwrap();
    let uv2 = UExtensionView::from_subtags("ca-buddhist").unwrap();
    assert_eq!(uv1, uv2);
    assert_eq!(uv1.partial_cmp(&uv2), Some(Ordering::Equal));
    assert_eq!(uv1.keyword_types("ca").count(), 1);
    assert_eq!(uv1.keyword_types("none").count(), 0);
    assert!(UExtensionView::from_subtags("ca--buddhist").is_err());

    let uv_attr = UExtensionView::parse("u-attr1-attr2-ca-gregory").unwrap();
    for _ in uv_attr.attributes() {}
    for _ in uv_attr.keyword_types("ca") {}

    let uv_pure_attr = UExtensionView::parse("u-attr1-attr2").unwrap();
    let attrs: Vec<_> = uv_pure_attr.attributes().collect();
    assert_eq!(attrs, vec!["attr1", "attr2"]);
    for _ in uv_pure_attr.keywords() {}

    let uv_no_type = UExtensionView::parse("u-ca").unwrap();
    let kws: Vec<_> = uv_no_type.keywords().collect();
    assert_eq!(kws, vec![("ca", None)]);

    // 8. t extension validators & methods
    assert!(t::is_fsep("m0"));
    assert!(!t::is_fsep("m"));
    assert!(!t::is_fsep("m00"));
    assert!(t::is_field_value("ungegn"));
    assert!(!t::is_field_value("ab"));
    assert!(!t::is_field_value("toolongfieldval"));
    assert!(!t::is_field_value("un_egn"));
    assert!(t::validate("it-m0-ungegn").is_ok());
    assert!(t::validate("t-it-m0-ungegn").is_ok());
    assert!(t::validate("t-it-419-m0-ungegn").is_ok());
    assert_eq!(t::validate_subtags_bytes(b""), Err(ParseError::EmptySubtag));
    assert_eq!(
        t::validate_subtags_bytes(b"it-"),
        Err(ParseError::TrailingSeparator)
    );
    assert_eq!(
        t::validate_subtags_bytes(b"--m0-ungegn"),
        Err(ParseError::EmptySubtag)
    );
    assert_eq!(
        t::validate_subtags_bytes(b"it--m0-ungegn"),
        Err(ParseError::EmptySubtag)
    );
    assert_eq!(
        t::validate_subtags_bytes(b"m0--ungegn"),
        Err(ParseError::EmptySubtag)
    );
    assert_eq!(
        t::validate_subtags_bytes(b"it-nedis-nedis"),
        Err(ParseError::DuplicateVariant)
    );
    assert_eq!(
        t::validate_subtags_bytes(b"it-nedis-rozaj-rozaj"),
        Err(ParseError::DuplicateVariant)
    );
    assert_eq!(
        t::validate_subtags_bytes(b"it-nedis-12345678-nedis"),
        Err(ParseError::DuplicateVariant)
    );
    assert!(t::validate_subtags_bytes(b"it-Latn-CN-nedis-m0-ungegn").is_ok());
    assert_eq!(
        t::validate_subtags_bytes(b"it-m0"),
        Err(ParseError::InvalidTField)
    );
    assert_eq!(
        t::validate_subtags_bytes(b"it-m0-toolongfieldval"),
        Err(ParseError::SubtagTooLong)
    );
    assert_eq!(
        t::validate_subtags_bytes(b"it-m0-f@o"),
        Err(ParseError::InvalidTField)
    );
    assert_eq!(
        t::validate_subtags_bytes(b"it-m0-ab"),
        Err(ParseError::InvalidTField)
    );

    let tv1 = TExtensionView::parse("it-m0-ungegn").unwrap();
    let tv2 = TExtensionView::from_subtags("it-m0-ungegn").unwrap();

    assert_eq!(tv1, tv2);
    assert_eq!(tv1.partial_cmp(&tv2), Some(Ordering::Equal));
    assert!(TExtensionView::from_subtags("m0--ungegn").is_err());

    let tv_bare = TExtensionView::parse("t-m0-ungegn").unwrap();
    assert_eq!(tv_bare.source_language(), None);
    let tv_with_lang = TExtensionView::parse("t-it-Latn-m0-ungegn").unwrap();
    assert_eq!(tv_with_lang.source_language(), Some("it-Latn"));

    let tv_fields = TExtensionView::parse("t-m0-ungegn-k0-test").unwrap();
    for _ in tv_fields.fields() {}
    for _ in tv_fields.field_values("m0") {}

    assert_eq!(tv1.field_values("m0").count(), 1);
    assert_eq!(tv1.field_values("none").count(), 0);

    // Iterators are constructed from validated parent views.
    let parent = NormalTagView::parse("en-a-foo-u-ca-buddhist-b-bar").unwrap();
    let exts: Vec<_> = parent.other_extensions().map(|e| e.singleton()).collect();
    assert_eq!(exts, vec!['a', 'b']);

    // 9. Fixed push_byte overflow
    let mut f_small = Fixed::<1>::default();
    assert!(f_small.push_byte(b'a').is_ok());
    assert!(f_small.push_byte(b'b').is_err());

    // 10. Grow max_capacity
    #[cfg(feature = "alloc")]
    {
        let nt_grow = NormalTag::<Grow<128>>::parse("zh").unwrap();
        assert_eq!(nt_grow.extensions_max_capacity(), 128);
        let pt_grow = PrivateUseTag::<Grow<128>>::parse("x-priv").unwrap();
        assert_eq!(pt_grow.max_capacity(), 128);
        let g = Grow::<128>::default();
        assert_eq!(Storage::max_capacity(&g), 128);
    }
}
