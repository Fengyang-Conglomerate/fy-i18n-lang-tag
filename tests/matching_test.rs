use fy_i18n_lang_tag::{
    LanguageMatch, MatchQuality, MatchResult, NormalTag, match_language, match_languages,
};

#[test]
fn test_match_quality_exact_and_core_exact() {
    let req: NormalTag = NormalTag::parse("zh-Hans-CN-u-ca-gregory").unwrap();
    let exact: NormalTag = NormalTag::parse("ZH-HANS-CN-U-CA-GREGORY").unwrap();
    let core: NormalTag = NormalTag::parse("zh-Hans-CN").unwrap();

    assert_eq!(req.match_quality(&exact), MatchQuality::Exact);
    assert_eq!(req.match_quality(&core), MatchQuality::CoreExact);
    assert!(req.matches(&exact));
    assert!(req.matches(&core));

    let view = req.as_view();
    assert_eq!(view.match_quality(&exact.as_view()), MatchQuality::Exact);
    assert_eq!(view.match_quality(&core.as_view()), MatchQuality::CoreExact);
}

#[test]
fn test_match_quality_script_and_region_and_language() {
    let req: NormalTag = NormalTag::parse("zh-Hans-CN").unwrap();

    let lang_script: NormalTag = NormalTag::parse("zh-Hans").unwrap();
    let lang_region: NormalTag = NormalTag::parse("zh-CN").unwrap();
    let lang_only: NormalTag = NormalTag::parse("zh").unwrap();

    assert_eq!(
        req.match_quality(&lang_script),
        MatchQuality::LanguageScript
    );
    assert_eq!(
        req.match_quality(&lang_region),
        MatchQuality::LanguageRegion
    );
    assert_eq!(req.match_quality(&lang_only), MatchQuality::Language);

    assert!(req.matches(&lang_script));
    assert!(req.matches(&lang_region));
    assert!(req.matches(&lang_only));
}

#[test]
fn test_match_quality_no_match_conflicts() {
    let req: NormalTag = NormalTag::parse("zh-Hans-CN").unwrap();

    // Different languages
    let en: NormalTag = NormalTag::parse("en-US").unwrap();
    assert_eq!(req.match_quality(&en), MatchQuality::NoMatch);
    assert!(!req.matches(&en));

    // Conflicting script (Hans vs Hant)
    let hant: NormalTag = NormalTag::parse("zh-Hant-CN").unwrap();
    assert_eq!(req.match_quality(&hant), MatchQuality::NoMatch);
    assert!(!req.matches(&hant));

    // Conflicting region (CN vs TW)
    let tw: NormalTag = NormalTag::parse("zh-Hans-TW").unwrap();
    assert_eq!(req.match_quality(&tw), MatchQuality::NoMatch);
    assert!(!req.matches(&tw));

    // Variant differences only affect Exact equality, core subtags remain identical
    let req_bare: NormalTag = NormalTag::parse("sl-Rozaj").unwrap();
    let cand_more_variants: NormalTag = NormalTag::parse("sl-Rozaj-biske").unwrap();
    assert_eq!(
        req_bare.match_quality(&cand_more_variants),
        MatchQuality::CoreExact
    );

    // Variant difference
    let req_var1: NormalTag = NormalTag::parse("sl-rozaj").unwrap();
    let req_var2: NormalTag = NormalTag::parse("sl-nedis").unwrap();
    assert_eq!(req_var1.match_quality(&req_var2), MatchQuality::CoreExact);

    // Same variant
    assert_eq!(req_var1.match_quality(&req_var1), MatchQuality::Exact);
}

#[test]
fn test_match_str_and_normal_tag_view_interactions() {
    let req: NormalTag = NormalTag::parse("zh-Hans-CN").unwrap();

    // String match implementation: Exact when identical
    assert_eq!(req.match_quality("zh-Hans-CN"), MatchQuality::Exact);
    assert_eq!(req.match_quality(&"zh-Hans-CN"), MatchQuality::Exact);
    assert_eq!(req.match_quality("invalid--tag"), MatchQuality::NoMatch);
    assert!(req.matches("zh-Hans-CN"));
    assert!(!req.matches("en-US"));

    // Request with extensions matches pure core candidate as CoreExact
    let req_ext: NormalTag = NormalTag::parse("zh-Hans-CN-u-ca-gregory").unwrap();
    assert_eq!(req_ext.match_quality("zh-Hans-CN"), MatchQuality::CoreExact);

    let view = req.as_view();
    assert_eq!(view.match_quality("zh-Hans-CN"), MatchQuality::Exact);
    assert_eq!(view.match_quality(&"zh-Hans-CN"), MatchQuality::Exact);
    assert_eq!(view.match_quality("invalid--tag"), MatchQuality::NoMatch);
    assert!(view.matches("zh-Hans-CN"));
    assert!(!view.matches("invalid--tag"));

    // View and Tag mutual matching
    assert_eq!(view.match_quality(&req), MatchQuality::Exact);
    assert_eq!(req.match_quality(&view), MatchQuality::Exact);
}

#[test]
fn test_match_language_function() {
    let req: NormalTag = NormalTag::parse("zh-Hans-CN-u-ca-gregory").unwrap();

    let available: [NormalTag; 4] = [
        NormalTag::parse("en-US").unwrap(),
        NormalTag::parse("zh").unwrap(),
        NormalTag::parse("zh-Hans").unwrap(),
        NormalTag::parse("zh-Hans-CN").unwrap(),
    ];

    let (idx, matched) = match_language(&req, &available).unwrap();
    assert_eq!(idx, 3);
    assert_eq!(matched.to_string(), "zh-Hans-CN");

    // Falls back to script match when exact region is missing
    let available2: [NormalTag; 3] = [
        NormalTag::parse("en-US").unwrap(),
        NormalTag::parse("zh").unwrap(),
        NormalTag::parse("zh-Hans").unwrap(),
    ];
    let (idx2, matched2) = match_language(&req, &available2).unwrap();
    assert_eq!(idx2, 2);
    assert_eq!(matched2.to_string(), "zh-Hans");

    // Falls back to language match when script is also missing
    let available3: [NormalTag; 2] = [
        NormalTag::parse("en-US").unwrap(),
        NormalTag::parse("zh").unwrap(),
    ];
    let (idx3, matched3) = match_language(&req, &available3).unwrap();
    assert_eq!(idx3, 1);
    assert_eq!(matched3.to_string(), "zh");

    // No matching candidates
    let available_none: [NormalTag; 2] = [
        NormalTag::parse("en-US").unwrap(),
        NormalTag::parse("ja-JP").unwrap(),
    ];
    assert!(match_language(&req, &available_none).is_none());
}

#[test]
fn test_structural_fallback_sequence() {
    let tag: NormalTag<fy_i18n_lang_tag::Fixed<64>> =
        NormalTag::parse("zh-Hans-CN-rozaj-u-ca-gregory-t-it-a-foo-x-priv").unwrap();
    let view = tag.as_view();

    let chain: Vec<String> = tag.structural_fallback().map(|v| v.to_string()).collect();
    assert_eq!(
        chain,
        vec![
            "zh-Hans-CN-rozaj-a-foo-t-it-u-ca-gregory-x-priv",
            "zh-Hans-CN-rozaj",
            "zh-Hans-CN",
            "zh-Hans",
            "zh"
        ]
    );

    let chain_view: Vec<String> = view.structural_fallback().map(|v| v.to_string()).collect();
    assert_eq!(chain, chain_view);

    // Simple fallback chain (language and region only)
    let tag2: NormalTag = NormalTag::parse("en-US").unwrap();
    let chain2: Vec<String> = tag2.structural_fallback().map(|v| v.to_string()).collect();
    assert_eq!(chain2, vec!["en-US", "en"]);

    // Pure language tag
    let tag3: NormalTag = NormalTag::parse("ja").unwrap();
    let chain3: Vec<String> = tag3.structural_fallback().map(|v| v.to_string()).collect();
    assert_eq!(chain3, vec!["ja"]);
}

#[test]
fn test_structural_match_methods_on_tag_and_view() {
    let t1: NormalTag = NormalTag::parse("zh-Hans-CN").unwrap();
    let t2: NormalTag = NormalTag::parse("zh-Hans").unwrap();

    assert!(t1.structural_matches(&t2));
    assert_eq!(
        t1.structural_match_quality(&t2),
        MatchQuality::LanguageScript
    );

    let v1 = t1.as_view();
    let v2 = t2.as_view();
    assert!(v1.structural_matches(&v2));
    assert_eq!(
        v1.structural_match_quality(&v2),
        MatchQuality::LanguageScript
    );
}

#[test]
fn test_match_languages_multi_requests_priority() {
    let requests: [NormalTag; 3] = [
        NormalTag::parse("zh-Hans").unwrap(), // Priority 1: Simplified
        NormalTag::parse("zh-Hant").unwrap(), // Priority 2: Traditional
        NormalTag::parse("en").unwrap(),      // Priority 3: English
    ];

    let available: [NormalTag; 2] = [
        NormalTag::parse("en-US").unwrap(),      // Matches Priority 3
        NormalTag::parse("zh-Hant-TW").unwrap(), // Matches Priority 2
    ];

    let matched: MatchResult<'_, NormalTag> = match_languages(&requests, &available).unwrap();
    assert_eq!(matched.request_index(), 1); // Matched zh-Hant
    assert_eq!(matched.candidate_index(), 1); // Matches available[1] zh-Hant-TW
    assert_eq!(matched.candidate().to_string(), "zh-Hant-TW");
    assert_eq!(matched.quality(), MatchQuality::LanguageScript);

    let (req_idx, cand_idx, cand) = matched.into_tuple();
    assert_eq!(req_idx, 1);
    assert_eq!(cand_idx, 1);
    assert_eq!(cand.to_string(), "zh-Hant-TW");

    // No matching candidates
    let available_none: [NormalTag; 2] = [
        NormalTag::parse("ja-JP").unwrap(),
        NormalTag::parse("ko-KR").unwrap(),
    ];
    assert!(match_languages(&requests, &available_none).is_none());

    // Empty requests or candidates
    let empty_requests: [NormalTag; 0] = [];
    assert!(match_languages(&empty_requests, &available).is_none());
    let empty_available: [NormalTag; 0] = [];
    assert!(match_languages(&requests, &empty_available).is_none());
}

#[test]
fn test_match_zh_hans_with_available_private_use() {
    // When candidates contain zh-Hans-x-abc, requesting zh-Hans must succeed
    let req: NormalTag = NormalTag::parse("zh-Hans").unwrap();
    let cand_with_priv: NormalTag = NormalTag::parse("zh-Hans-x-abc").unwrap();
    let cand_exact: NormalTag = NormalTag::parse("zh-Hans").unwrap();

    // 1. Match quality: base request matching candidate with private-use extension is CoreExact
    assert_eq!(req.match_quality(&cand_with_priv), MatchQuality::CoreExact);
    assert!(req.matches(&cand_with_priv));

    // 2. Candidate list containing only zh-Hans-x-abc successfully matches
    let available_only_priv = [cand_with_priv.clone()];
    let (idx, matched) = match_language(&req, &available_only_priv).unwrap();
    assert_eq!(idx, 0);
    assert_eq!(matched.to_string(), "zh-Hans-x-abc");

    // 3. When both exact zh-Hans and zh-Hans-x-abc are available, prefer Exact
    let available_both = [cand_with_priv.clone(), cand_exact.clone()];
    let (idx2, matched2) = match_language(&req, &available_both).unwrap();
    assert_eq!(idx2, 1);
    assert_eq!(matched2.to_string(), "zh-Hans");

    // 4. Multi-request match_languages also succeeds
    let requests = [req];
    let res = match_languages(&requests, &available_only_priv).unwrap();
    assert_eq!(res.request_index, 0);
    assert_eq!(res.candidate_index, 0);
    assert_eq!(res.quality, MatchQuality::CoreExact);
    assert_eq!(res.candidate.to_string(), "zh-Hans-x-abc");
}

#[test]
fn test_private_use_differences_only_affect_exact() {
    // Private-use differences only affect Exact equality
    let req_abc: NormalTag = NormalTag::parse("zh-Hans-x-abc").unwrap();
    let cand_def: NormalTag = NormalTag::parse("zh-Hans-x-def").unwrap();
    let cand_abc: NormalTag = NormalTag::parse("zh-Hans-x-abc").unwrap();

    assert_eq!(req_abc.match_quality(&cand_def), MatchQuality::CoreExact);
    assert!(req_abc.matches(&cand_def));

    assert_eq!(req_abc.match_quality(&cand_abc), MatchQuality::Exact);
    assert!(req_abc.matches(&cand_abc));
}

#[test]
fn test_str_and_slice_combinations_for_match_languages() {
    let requests = ["zh-Hans", "zh-Hant", "en"];
    let available = ["en-US", "zh-Hans-CN"];

    // Pure str slice vs pure str slice
    let matched = match_languages(&requests, &available).unwrap();
    assert_eq!(matched.request_index, 0); // Matched zh-Hans
    assert_eq!(matched.candidate_index, 1); // Matches zh-Hans-CN
    assert_eq!(*matched.candidate, "zh-Hans-CN");
    assert_eq!(matched.quality, MatchQuality::LanguageScript);
}

#[test]
fn test_str_matching_owned_tag_and_best_quality_upgrade() {
    let tag: NormalTag = NormalTag::parse("zh-Hans-CN").unwrap();
    // str matching NormalTag
    assert_eq!("zh-Hans-CN".match_quality(&tag), MatchQuality::Exact);
    assert_eq!("invalid--tag".match_quality(&tag), MatchQuality::NoMatch);

    // str matching &str
    assert_eq!(
        "zh-Hans-CN".match_quality(&"zh-Hans-CN"),
        MatchQuality::Exact
    );
    assert_eq!(
        "zh-Hans-CN".match_quality(&"invalid--tag"),
        MatchQuality::NoMatch
    );

    // Quality upgrade in match_languages (best_quality < quality)
    let requests = ["zh-Hans-CN"];
    let available = ["zh", "zh-Hans", "zh-Hans-CN"];
    let matched = match_languages(&requests, &available).unwrap();
    assert_eq!(matched.candidate_index, 2);
    assert_eq!(matched.quality, MatchQuality::Exact);

    // Skip invalid tags in requests
    let requests_with_invalid = ["invalid--tag", "zh-Hans"];
    let matched_valid = match_languages(&requests_with_invalid, &available).unwrap();
    assert_eq!(matched_valid.request_index, 1);
    assert_eq!(matched_valid.candidate_index, 1);
    assert_eq!(*matched_valid.candidate, "zh-Hans");

    // str matching NormalTagView
    let view = tag.as_view();
    assert_eq!("zh-Hans-CN".match_quality(&view), MatchQuality::Exact);
    assert_eq!("invalid--tag".match_quality(&view), MatchQuality::NoMatch);

    // Variant difference branches
    let tag_v1: NormalTag = NormalTag::parse("sl-rozaj").unwrap();
    let tag_v2: NormalTag = NormalTag::parse("sl-nedis").unwrap();
    let tag_no_v: NormalTag = NormalTag::parse("sl").unwrap();
    assert_eq!(tag_v1.match_quality(&tag_v2), MatchQuality::CoreExact);
    assert_eq!(tag_no_v.match_quality(&tag_v1), MatchQuality::CoreExact);
}

#[test]
fn test_fixed_matching_contract_table_across_representations() {
    use fy_i18n_lang_tag::{Fixed, NormalTagView};

    // Symmetric behavior across strings, owned tags, and borrowed views
    let cases = [
        (
            "zh-Hans-CN-u-ca-gregory",
            "ZH-HANS-CN-U-CA-GREGORY",
            MatchQuality::Exact,
        ),
        ("en-a-foo-b-bar", "EN-B-BAR-A-FOO", MatchQuality::Exact),
        (
            "zh-Hans-CN-u-ca-gregory",
            "zh-Hans-CN",
            MatchQuality::CoreExact,
        ),
        (
            "zh-Hans-CN-u-ca-gregory",
            "zh-Hans",
            MatchQuality::LanguageScript,
        ),
        (
            "zh-Hans-CN-u-ca-gregory",
            "zh-CN",
            MatchQuality::LanguageRegion,
        ),
        ("zh-Hans-CN-u-ca-gregory", "zh", MatchQuality::Language),
        (
            "zh-Hans-CN-u-ca-gregory",
            "zh-Hant-CN",
            MatchQuality::NoMatch,
        ),
        (
            "zh-Hans-CN-u-ca-gregory",
            "zh-Hans-TW",
            MatchQuality::NoMatch,
        ),
        ("zh-Hans-CN-u-ca-gregory", "en", MatchQuality::NoMatch),
        ("zh-Hans", "zh-Hans-x-abc", MatchQuality::CoreExact),
        ("zh-Hans-x-abc", "zh-Hans-x-def", MatchQuality::CoreExact),
        ("sl-rozaj", "sl-nedis", MatchQuality::CoreExact),
        ("sl", "sl-rozaj-biske", MatchQuality::CoreExact),
        (
            "en-u-ca-gregory",
            "en-u-ca-buddhist",
            MatchQuality::CoreExact,
        ),
        ("en-t-en", "en-t-it", MatchQuality::CoreExact),
        ("en-a-foo", "en-a-bar", MatchQuality::CoreExact),
        ("zh-Hans", "zh-Hans-CN", MatchQuality::LanguageScript),
        ("zh-TW", "zh-Hant", MatchQuality::Language),
        ("zh", "zh-Hans-CN", MatchQuality::Language),
        ("en-x-a", "en-Latn-US-x-a", MatchQuality::Language),
        ("en-Latn-US-x-a", "en-Cyrl-US-x-b", MatchQuality::NoMatch),
        ("en-Latn-US-x-a", "en-Latn-GB-x-b", MatchQuality::NoMatch),
    ];
    for (request, candidate, expected) in cases {
        let r = NormalTagView::parse(request).unwrap();
        let c = NormalTagView::parse(candidate).unwrap();
        let ro: NormalTag = NormalTag::parse(request).unwrap();
        let co: NormalTag<Fixed<64>> = NormalTag::parse(candidate).unwrap();
        assert_eq!(r.match_quality(&c), expected, "{request} vs {candidate}");
        assert_eq!(c.match_quality(&r), expected);
        assert_eq!(request.match_quality(candidate), expected);
        assert_eq!(candidate.match_quality(request), expected);
        assert_eq!(ro.match_quality(&co), expected);
        assert_eq!(co.match_quality(&ro), expected);
        assert_eq!(ro.match_quality(&c), expected);
        assert_eq!(r.match_quality(&co), expected);
        assert_eq!(ro.match_quality(candidate), expected);
        assert_eq!(request.match_quality(&co), expected);
        assert_eq!(r.matches(&c), expected > MatchQuality::NoMatch);
        assert_eq!(r == c, expected == MatchQuality::Exact);
    }
}

#[test]
fn test_fixed_candidate_order_and_request_priority() {
    // Candidates ordered from weakest to strongest match
    let candidates = [
        "en",
        "zh",
        "zh-CN",
        "zh-Hans",
        "zh-Hans-CN",
        "zh-Hans-CN-u-ca-gregory",
    ];
    let request = "zh-Hans-CN-u-ca-gregory";
    assert_eq!(match_language(request, &candidates[..1]), None);
    for length in 2..=candidates.len() {
        assert_eq!(
            match_language(request, &candidates[..length]),
            Some((length - 1, &candidates[length - 1]))
        );
        let result = match_languages(&[request], &candidates[..length]).unwrap();
        assert_eq!(result.candidate_index, length - 1);
        assert_eq!(result.request_index, 0);
    }

    let tied = ["zh-Hans-x-b", "zh-Hans-x-a"];
    assert_eq!(match_language("zh-Hans", &tied), Some((0, &tied[0])));
    let result = match_languages(&["zh-Hans"], &tied).unwrap();
    assert_eq!(result.candidate_index, 0);
    assert_eq!(result.quality, MatchQuality::CoreExact);

    // First request's Language match takes precedence over second request's Exact match
    let candidates = ["en-US", "zh"];
    let result = match_languages(&["zh-Hans-CN", "en-US"], &candidates).unwrap();
    assert_eq!(result.request_index, 0);
    assert_eq!(result.candidate_index, 1);
    assert_eq!(result.quality, MatchQuality::Language);

    // Proceed to next request only when earlier request fails to match
    let candidates = ["zh-Hant", "en-US"];
    let result = match_languages(&["zh-Hans", "en-US"], &candidates).unwrap();
    assert_eq!(result.request_index, 1);
    assert_eq!(result.candidate_index, 1);
    assert_eq!(result.quality, MatchQuality::Exact);

    let empty: [&str; 0] = [];
    assert_eq!(match_language("zh", &empty), None);
    assert!(match_languages(&empty, &candidates).is_none());
    assert!(match_languages(&["zh"], &empty).is_none());
}

#[test]
fn test_ignored_extensions_still_require_well_formed_tags() {
    for invalid in [
        "en-a-foo-",
        "en-u-ca-gregory-ca-latn",
        "en-t-m0",
        "en-x-",
        "x-a",
    ] {
        assert_eq!("en".match_quality(invalid), MatchQuality::NoMatch);
        assert_eq!(invalid.match_quality("en"), MatchQuality::NoMatch);
    }
}
