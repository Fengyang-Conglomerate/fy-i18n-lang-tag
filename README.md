# fy-i18n-lang-tag

[![crates.io](https://img.shields.io/crates/v/fy-i18n-lang-tag)](https://crates.io/crates/fy-i18n-lang-tag)
[![docs.rs](https://docs.rs/fy-i18n-lang-tag/badge.svg)](https://docs.rs/fy-i18n-lang-tag)
[![CI](https://github.com/Fengyang-Conglomerate/fy-i18n-lang-tag/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/Fengyang-Conglomerate/fy-i18n-lang-tag/actions/workflows/ci.yml)
[![Coverage](https://coveralls.io/repos/github/Fengyang-Conglomerate/fy-i18n-lang-tag/badge.svg?branch=main)](https://coveralls.io/github/Fengyang-Conglomerate/fy-i18n-lang-tag?branch=main)
[![Rust 1.86+](https://img.shields.io/badge/rust-1.86%2B-blue)](https://www.rust-lang.org/)
[![no_std](https://img.shields.io/badge/no__std-compatible-green)](#features)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](#license)

English | [简体中文](README.zh-CN.md)

Lightweight, high-performance, and `no_std`-first BCP 47 language tag parser, normalizer, and matcher based on [RFC 5646](https://www.rfc-editor.org/rfc/rfc5646).

## Features

- **Zero-Allocation by Default**: Runs in bare-metal `no_std` environments with zero heap allocation. Powered by a compact 15-byte stack buffer `[u8; 15]` for core subtags and configurable stack storage ([`Fixed<32>`]) for dynamic extensions.
- **Zero-Cost Borrowed Views**: Zero-copy borrowed views ([`NormalTagView`], [`PrivateUseTagView`]) callable in `const` compile-time contexts.
- **Pre-Indexed Projections**: Owned tags maintain internal boundary indices, enabling instant zero-cost view projections without re-parsing.
- **Strongly-Typed Binary System**: Clear separation between normal tags ([`NormalTag`]) and pure private-use tags ([`PrivateUseTag`]) without dynamic enum overhead.
- **Syntax validation and normalization**:
  - Full ASCII canonical casing (lowercase language, Titlecase script, uppercase region, lowercase extensions).
  - Single-letter extensions sorted lexicographically by singleton (e.g. `t-` before `u-`).
  - RFC 5646 4ALPHA reserved grammar support (e.g., `root`, `abcd`).
  - Deep validation and dedicated views for Unicode locale extensions (`u`, RFC 6067) and transformed content extensions (`t`, RFC 6497).
- **Structural Matching & Fallback**:
  - Deterministic 6-level syntax-level match quality ([`MatchQuality`]): `Exact` > `CoreExact` > `LanguageScript` > `LanguageRegion` > `Language` > `NoMatch`.
  - Multi-request priority matching ([`match_languages`]).
  - Lazy, zero-allocation component fallback iterator ([`StructuralFallbackIter`]).
- **Optional `serde` & `alloc` Support**:
  - Optional `alloc` feature for unbounded heap storage ([`Grow`]).
  - Optional `serde` feature supporting zero-copy borrowed deserialization (`#[serde(borrow)]`).

## Changelog

See [English changelog](CHANGELOG.md) or [简体中文更新日志](CHANGELOG.zh-CN.md). Parsing checks supported syntax, not IANA registration or complete Unicode locale canonicalization.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
fy-i18n-lang-tag = "0.2"
```

To enable dynamic heap storage and `serde`:

```toml
[dependencies]
fy-i18n-lang-tag = { version = "0.2", features = ["alloc", "serde"] }
```

## Quick Start

### Normal Language Tags

```rust
use fy_i18n_lang_tag::NormalTag;

// Parses and normalizes casing and extension order
let tag: NormalTag = NormalTag::parse("ZH-hans-cn-u-ca-gregory-t-it")?;

assert_eq!(tag.to_string(), "zh-Hans-CN-t-it-u-ca-gregory");
assert_eq!(tag.language().as_str(), "zh");
assert_eq!(tag.script(), Some("Hans"));
assert_eq!(tag.region(), Some("CN"));

// Structured extension views
let u_ext = tag.u().unwrap();
assert_eq!(u_ext.keyword_type("ca"), Some("gregory"));

let t_ext = tag.t().unwrap();
assert_eq!(t_ext.source_language(), Some("it"));
# Ok::<(), fy_i18n_lang_tag::TagError>(())
```

### Zero-Copy Borrowed Views in `const` Contexts

```rust
use fy_i18n_lang_tag::NormalTagView;

const DEFAULT_LANG: NormalTagView<'static> = match NormalTagView::parse("en-US") {
    Ok(tag) => tag,
    Err(_) => unreachable!(),
};

assert_eq!(DEFAULT_LANG.language().as_str(), "en");
assert_eq!(DEFAULT_LANG.region(), Some("US"));
```

### Pure Private-Use Tags

```rust
use fy_i18n_lang_tag::PrivateUseTag;

let priv_tag: PrivateUseTag = PrivateUseTag::parse("x-custom-engine")?;
assert_eq!(priv_tag.as_str(), "custom-engine");
assert_eq!(priv_tag.to_string(), "x-custom-engine");
# Ok::<(), fy_i18n_lang_tag::private_use::TagError>(())
```

### Structural Language Matching

```rust
use fy_i18n_lang_tag::{NormalTag, match_languages, MatchQuality};

let requests: [NormalTag; 2] = [
    NormalTag::parse("zh-Hans")?, // Priority 1
    NormalTag::parse("en")?,      // Priority 2
];

let available: [NormalTag; 2] = [
    NormalTag::parse("en-US")?,
    NormalTag::parse("zh-Hans-CN")?,
];

let matched = match_languages(&requests, &available).unwrap();
assert_eq!(matched.request_index(), 0); // Matched zh-Hans
assert_eq!(matched.candidate().to_string(), "zh-Hans-CN");
assert_eq!(matched.quality(), MatchQuality::LanguageScript);
# Ok::<(), fy_i18n_lang_tag::TagError>(())
```

## Cargo Features

| Feature | Default | Description |
|---|---|---|
| `alloc` | No | Enables heap-allocated [`Grow`] storage backend for unbounded tags. |
| `serde` | No | Enables `Serialize` and `Deserialize` implementations for tags and views. |
| `unsafe` | No | Enables unchecked string conversions for maximum projection throughput. |

## MSRV

The Minimum Supported Rust Version (MSRV) is **Rust 1.86.0**.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
