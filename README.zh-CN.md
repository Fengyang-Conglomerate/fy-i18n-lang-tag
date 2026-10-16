# fy-i18n-lang-tag

[![crates.io](https://img.shields.io/crates/v/fy-i18n-lang-tag)](https://crates.io/crates/fy-i18n-lang-tag)
[![docs.rs](https://docs.rs/fy-i18n-lang-tag/badge.svg)](https://docs.rs/fy-i18n-lang-tag)
[![CI](https://github.com/Fengyang-Conglomerate/fy-i18n-lang-tag/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/Fengyang-Conglomerate/fy-i18n-lang-tag/actions/workflows/ci.yml)
[![Coverage](https://coveralls.io/repos/github/Fengyang-Conglomerate/fy-i18n-lang-tag/badge.svg?branch=main)](https://coveralls.io/github/Fengyang-Conglomerate/fy-i18n-lang-tag?branch=main)
[![Rust 1.86+](https://img.shields.io/badge/rust-1.86%2B-blue)](https://www.rust-lang.org/)
[![no_std](https://img.shields.io/badge/no__std-compatible-green)](#特性标志)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](#许可)

[English](README.md) | 简体中文

基于 [RFC 5646](https://www.rfc-editor.org/rfc/rfc5646) 标准的轻量级、高性能、`no_std` 优先的 BCP 47 语言标签解析、规范化与匹配库。

## 特性

- **默认零堆分配**：全面支持裸机 `no_std` 环境。核心三元组紧凑存储于 15 字节栈缓冲区 `[u8; 15]`，动态扩展字段默认采用可配置定长栈存储（[`Fixed<32>`]）。
- **零成本借用视图**：提供借用视图（[`NormalTagView`] 与 [`PrivateUseTagView`]），支持在 `const` 编译期常量上下文中完成解析。
- **预索引即时投影**：拥有型结构内置边界索引，借出借用视图无需重新进行字符串语法解析。
- **强类型二元体系**：清晰分离普通语言标签（[`NormalTag`]）与全私用标签（[`PrivateUseTag`]），无额外枚举判别开销。
- **语法验证与规范化**：
  - 全 ASCII 规范大小写（主语言全小写、文字首字母大写、地区全大写、扩展全小写）。
  - 单字母扩展严格按 singleton 字典序升序排列（例如 `t-` 位于 `u-` 之前）。
  - 完整支持 RFC 5646 4ALPHA 保留语言产生式（例如 `root`、`abcd`）。
  - 深入支持 Unicode 语言环境扩展（`u`，RFC 6067）与转换内容扩展（`t`，RFC 6497）的语法验证与结构化视图。
- **语法级匹配与回退**：
  - 确定性六级匹配质量（[`MatchQuality`]）：`Exact` > `CoreExact` > `LanguageScript` > `LanguageRegion` > `Language` > `NoMatch`。
  - 多请求优先级匹配（[`match_languages`])。
  - 惰性、零分配的组件级回退迭代器（[`StructuralFallbackIter`]）。
- **可选 `serde` 与 `alloc` 支持**：
  - 可选 `alloc` 特性支持无界动态堆存储（[`Grow`]）。
  - 可选 `serde` 特性支持零拷贝借用反序列化（`#[serde(borrow)]`）。

## 更新日志

参见[中文更新日志](CHANGELOG.zh-CN.md)或 [English changelog](CHANGELOG.md)。解析检查所支持的语法，不检查 IANA 注册有效性，也不执行完整的 Unicode 语言环境规范化。

## 安装

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
fy-i18n-lang-tag = "0.2"
```

若需启用动态堆存储与 `serde` 支持：

```toml
[dependencies]
fy-i18n-lang-tag = { version = "0.2", features = ["alloc", "serde"] }
```

## 快速上手

### 普通语言标签

```rust
use fy_i18n_lang_tag::NormalTag;

// 解析并规范化大小写与扩展顺序
let tag: NormalTag = NormalTag::parse("ZH-hans-cn-u-ca-gregory-t-it")?;

assert_eq!(tag.to_string(), "zh-Hans-CN-t-it-u-ca-gregory");
assert_eq!(tag.language().as_str(), "zh");
assert_eq!(tag.script(), Some("Hans"));
assert_eq!(tag.region(), Some("CN"));

// 结构化扩展视图
let u_ext = tag.u().unwrap();
assert_eq!(u_ext.keyword_type("ca"), Some("gregory"));

let t_ext = tag.t().unwrap();
assert_eq!(t_ext.source_language(), Some("it"));
# Ok::<(), fy_i18n_lang_tag::TagError>(())
```

### 编译期 `const` 借用视图

```rust
use fy_i18n_lang_tag::NormalTagView;

const DEFAULT_LANG: NormalTagView<'static> = match NormalTagView::parse("en-US") {
    Ok(tag) => tag,
    Err(_) => unreachable!(),
};

assert_eq!(DEFAULT_LANG.language().as_str(), "en");
assert_eq!(DEFAULT_LANG.region(), Some("US"));
```

### 全私用语言标签

```rust
use fy_i18n_lang_tag::PrivateUseTag;

let priv_tag: PrivateUseTag = PrivateUseTag::parse("x-custom-engine")?;
assert_eq!(priv_tag.as_str(), "custom-engine");
assert_eq!(priv_tag.to_string(), "x-custom-engine");
# Ok::<(), fy_i18n_lang_tag::private_use::TagError>(())
```

### 结构化语言匹配

```rust
use fy_i18n_lang_tag::{NormalTag, match_languages, MatchQuality};

let requests: [NormalTag; 2] = [
    NormalTag::parse("zh-Hans")?, // 优先 1：简体中文
    NormalTag::parse("en")?,      // 优先 2：英文
];

let available: [NormalTag; 2] = [
    NormalTag::parse("en-US")?,
    NormalTag::parse("zh-Hans-CN")?,
];

let matched = match_languages(&requests, &available).unwrap();
assert_eq!(matched.request_index(), 0); // 命中 zh-Hans
assert_eq!(matched.candidate().to_string(), "zh-Hans-CN");
assert_eq!(matched.quality(), MatchQuality::LanguageScript);
# Ok::<(), fy_i18n_lang_tag::TagError>(())
```

## 特性标志

| 特性 | 默认开启 | 说明 |
|---|---|---|
| `alloc` | 否 | 启用基于堆分配的 [`Grow`] 存储后端，支持无上限扩展。 |
| `serde` | 否 | 为各标签与视图提供 `Serialize` 与 `Deserialize` 支持。 |
| `unsafe` | 否 | 启用不检查的底层字符串转换，提供极致投影吞吐。 |

## MSRV

最低支持的 Rust 版本（MSRV）为 **Rust 1.86.0**。

## 许可

可根据您的意愿选择以下任一许可证授权：

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))
