# Changelog

English | [简体中文](CHANGELOG.zh-CN.md)

## 0.2.0 — 2026-10-16

This release rewrites the implementation and introduces breaking API changes from 0.1.0.

### Added

- Zero-allocation borrowed `NormalTagView` and `PrivateUseTagView`, with `const` parsing.
- Configurable `Fixed<N>` inline storage; optional `alloc` enables `Grow<N>` heap storage. Owned normal tags store their core in 15 bytes and index extension boundaries for borrowed projection.
- Structured language, script, region, Unicode `u`, transformed `t`, and other-extension views, including extension-specific syntax checks.
- Four-letter primary language subtags and transformed-language syntax in `t` extensions.
- Six-level `MatchQuality`, single- and multiple-request matching, and structural fallback iteration.
- Optional `unsafe` fast paths; default builds forbid unsafe code. Optional `serde` supports borrowed deserialization.

### Changed / migration from 0.1

- Replace the old `Tag` enum with separate `NormalTag<S>` and `PrivateUseTag<S>` entry points. The old `Subtag` and component construction APIs have been replaced by the new view/storage APIs.
- Owned parsing now returns the corresponding module's `TagError`, separating syntax and capacity errors. Borrowed parsing returns that module's `ParseError`.
- Specify storage through a type annotation, for example `let tag: NormalTag = NormalTag::parse("en-US")?;`. Default owned storage is `Fixed<32>`; choose a larger capacity or `Grow` for larger stored content. Capacity exhaustion is an error.
- `script()` and `region()` return optional string slices; use `script_view()` and `region_view()` for typed views.
- `alloc` is no longer required by default. Enable it explicitly when using `Grow`.
- Minimum supported Rust version is 1.86.0 (Rust 2024 edition), down from the 0.1.0 declaration of 1.98.

### Validation and release checks

- Normalization covers ASCII casing and extension-singleton ordering, with private use last. Parsing validates supported syntax, not IANA registration or complete Unicode locale canonicalization; it does not infer missing language components.
- Corrected both README examples and added executable checks for both languages.
- CI checks MSRV and stable, both feature extremes, formatting, Clippy, rustdoc, packaging, coverage, and wasm32/thumbv7m `no_std` builds. Tag pushes do not duplicate main CI.

- Version tags automatically publish bilingual GitHub Release notes after main CI for the same commit succeeds.

[Compare 0.1.0…0.2.0](https://github.com/Fengyang-Conglomerate/fy-i18n-lang-tag/compare/v0.1.0...v0.2.0)

## 0.1.0

Initial public release: owned modern language-tag parsing and normalization, typed components, a `Tag` enum covering normal and pure private-use tags, syntax diagnostics, and optional `serde`. Supported `no_std` with required `alloc`; did not apply `u`/`t` extension-specific grammar.
