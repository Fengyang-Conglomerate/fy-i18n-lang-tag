# fy-i18n-lang-tag

[![crates.io](https://img.shields.io/crates/v/fy-i18n-lang-tag)](https://crates.io/crates/fy-i18n-lang-tag)
[![docs.rs](https://docs.rs/fy-i18n-lang-tag/badge.svg)](https://docs.rs/fy-i18n-lang-tag)
[![CI](https://github.com/Fengyang-Conglomerate/fy-i18n-lang-tag/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/Fengyang-Conglomerate/fy-i18n-lang-tag/actions/workflows/ci.yml)
[![Coverage](https://coveralls.io/repos/github/Fengyang-Conglomerate/fy-i18n-lang-tag/badge.svg?branch=main)](https://coveralls.io/github/Fengyang-Conglomerate/fy-i18n-lang-tag?branch=main)
[![Rust 1.98+](https://img.shields.io/badge/rust-1.98%2B-blue)](https://www.rust-lang.org/)
[![no_std + alloc](https://img.shields.io/badge/no__std-alloc-green)](#scope)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](#license)

English | [简体中文](README.zh-CN.md)

A lightweight parser and normalizer for modern language tags based on a subset
of [RFC 5646](https://www.rfc-editor.org/rfc/rfc5646). The crate supports `no_std`,
requires `alloc`, and has no dependencies unless the `serde` feature is enabled.

## Scope

- **Syntax only (well-formedness):** no IANA registry, preferred-value
  replacement, extlang collapsing, or grandfathered-tag lookup. Registry-based
  validity checking must be handled separately.
- The primary language subtag consists of 2-3 or 5-8 ASCII letters. Extlang
  subtags and reserved four-letter primary language subtags are rejected.
- Parsing normalizes casing: lowercase language, title-case script, uppercase
  region, and lowercase remaining subtags. Extensions are sorted by singleton.
  Original input casing is not preserved; variant order is preserved.
- Missing script and region subtags remain `None`. The parser never infers or
  inserts them.
- POSIX locale forms such as `en_US`, `en_US.UTF-8`, and `de_DE@euro` are rejected.

## Usage

Requires Rust 1.98 or newer.

```toml
[dependencies]
fy-i18n-lang-tag = "0.1"
```

Use `NormalTag` for tags with a primary language subtag:

```rust
use fy_i18n_lang_tag::NormalTag;

let tag = NormalTag::parse("ZH-hans-cn-u-ca-buddhist-x-note")?;

assert_eq!(tag.to_string(), "zh-Hans-CN-u-ca-buddhist-x-note");
assert_eq!(tag.language().as_str(), "zh");
assert_eq!(tag.script().unwrap().as_str(), "Hans");
assert_eq!(tag.region().unwrap().as_str(), "CN");
assert_eq!(tag.extensions()[0].singleton(), 'u');
assert_eq!(tag.private_use().len(), 1);
# Ok::<(), fy_i18n_lang_tag::ParseError>(())
```

Use `Tag` when private-use-only tags (`x-...`) are also needed:

```rust
use fy_i18n_lang_tag::Tag;

match Tag::parse("x-pig-latin")? {
    Tag::PrivateUse(tag) => assert_eq!(tag.to_string(), "x-pig-latin"),
    _ => unreachable!(),
}
# Ok::<(), fy_i18n_lang_tag::ParseError>(())
```

`ParseError` identifies the error category. Its `position` is a zero-based byte
offset into the input:

```rust
use fy_i18n_lang_tag::{NormalTag, ParseError};

assert_eq!(
    NormalTag::parse("zh_CN"),
    Err(ParseError::InvalidCharacter { position: 2 })
);
```

## Validation and Normalization

Successful parsing means that the input matches this crate's supported syntax.
It does not establish that subtags are registered or that their combination is
appropriate. Grandfathered tags that also fit the supported syntax, such as
`art-lojban`, are parsed literally without applying registry mappings.

An extension consists of a single ASCII letter or digit other than `x`
(case-insensitive), followed by one or more subtags of 2-8 ASCII letters or digits.
Extension-specific rules,
including Unicode `u` and `t` extension semantics and canonicalization, are not
applied. Normalization is limited to casing and extension order as described
above; it is not full registry-based or Unicode locale canonicalization.

The parser imposes no fixed total-length or subtag-count limit. Tags longer than
255 bytes are supported, subject to available memory.

`NormalTag::from_parts` constructs a tag from validated component types and
returns `BuildError` for duplicate variants or extension singletons.
`Extension::new` and `PrivateUseTag::new` also return `BuildError` for invalid
component combinations. Construction errors describe component values or sequence
indices; only string parsing returns `ParseError` with input byte offsets. Parsing,
component construction, and transformations preserve the same invariants, so
their output can be parsed again without changing the value.

## Diagnostics and Ordering

Lexical checks run before grammar checks. A left-to-right scan reports invalid
characters as encountered and empty or overlong subtags at a separator or the
end of input. The first detected lexical error wins. Grammar errors depend on
the parser entry point and available grammar slots. For example, a second
singleton starts another extension in a full tag, but is unexpected when parsing
a single `Extension`. `PrivateUseTag::parse` requires an `x` prefix and reports
`MissingPrivateUsePrefix` if it is absent after lexical validation.

Ordering is structural, not displayed-string ordering. `NormalTag` compares
language, script, region, variants, extensions, and private use in that order;
`None` precedes `Some`, and sequences compare lexicographically. Thus `en-US`
sorts before `en-Latn`. `Tag` places normal tags before private-use-only tags.

## Optional Serde Support

The `serde` feature enables string serialization and deserialization for all
core types: `Tag`, `NormalTag`, `PrivateUseTag`, `Extension`, and the five atomic
subtag types. Serialization uses the normalized `Display` representation;
deserialization validates and normalizes through `parse`.

```toml
[dependencies]
fy-i18n-lang-tag = { version = "0.1", features = ["serde"] }
```

## License

Licensed under either of the following, at your option:

- [Apache License, Version 2.0](LICENSE-APACHE)
- [MIT license](LICENSE-MIT)
