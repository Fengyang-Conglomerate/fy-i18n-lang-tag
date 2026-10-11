# fy-i18n-lang-tag

[![crates.io](https://img.shields.io/crates/v/fy-i18n-lang-tag)](https://crates.io/crates/fy-i18n-lang-tag)
[![docs.rs](https://docs.rs/fy-i18n-lang-tag/badge.svg)](https://docs.rs/fy-i18n-lang-tag)
[![CI](https://github.com/Fengyang-Conglomerate/fy-i18n-lang-tag/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/Fengyang-Conglomerate/fy-i18n-lang-tag/actions/workflows/ci.yml)
[![Coverage](https://coveralls.io/repos/github/Fengyang-Conglomerate/fy-i18n-lang-tag/badge.svg?branch=main)](https://coveralls.io/github/Fengyang-Conglomerate/fy-i18n-lang-tag?branch=main)
[![Rust 1.98+](https://img.shields.io/badge/rust-1.98%2B-blue)](https://www.rust-lang.org/)
[![no_std + alloc](https://img.shields.io/badge/no__std-alloc-green)](#范围)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](#许可)

[English](README.md) | 简体中文

基于 [RFC 5646](https://www.rfc-editor.org/rfc/rfc5646) 语法子集的轻量现代语言标签
解析与规范化库。支持 `no_std`，需要 `alloc`；未启用 `serde` 特性时无依赖。

## 范围

- 仅做**语法校验**（well-formedness）：不携带 IANA 注册表、不做
  Preferred-Value 替换、不折叠 extlang、不查询历史保留标签（grandfathered）清单。
  注册表级校验（validity）需要由调用方另行处理。
- 主语言子标签接受 2–3 或 5–8 个 ASCII 字母，拒绝 extlang 子标签与
  4 字母的保留主语言形式。
- 解析即规范化：语言全小写、文字首字母大写、地区全大写、其余子标签全小写，
  扩展序列按 singleton 字典序排列，变体保留输入顺序。不保留输入的原始大小写。
- 输入未包含的文字或地区在解析结果中为 `None`，不推断或补全。
- 拒绝 `en_US`、`en_US.UTF-8`、`de_DE@euro` 等 POSIX locale 形式。

## 使用

需要 Rust 1.98 或更新版本。

```toml
[dependencies]
fy-i18n-lang-tag = "0.1"
```

包含主语言子标签时使用 `NormalTag`：

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

还需要支持纯私用标签（`x-...`）时使用 `Tag`：

```rust
use fy_i18n_lang_tag::Tag;

match Tag::parse("x-pig-latin")? {
    Tag::PrivateUse(tag) => assert_eq!(tag.to_string(), "x-pig-latin"),
    _ => unreachable!(),
}
# Ok::<(), fy_i18n_lang_tag::ParseError>(())
```

`ParseError` 区分错误类别，其 `position` 是输入中从 0 开始的字节偏移：

```rust
use fy_i18n_lang_tag::{NormalTag, ParseError};

assert_eq!(
    NormalTag::parse("zh_CN"),
    Err(ParseError::InvalidCharacter { position: 2 })
);
```

## 校验与规范化

解析成功表示输入符合本库支持的语法，不代表子标签已经注册，也不代表其组合适用于
实际语言场景。符合所支持语法的历史保留标签（如 `art-lojban`）按字面解析，
不应用注册表中的映射。

扩展由一个非 `x` 的 ASCII 字母或数字（singleton，不区分大小写）引导，后接一个或多个
由 2–8 个 ASCII 字母或数字组成的子标签。本库不应用各扩展的专有规则，
包括 Unicode `u`、`t` 扩展的语义校验和规范化。规范化仅包含上述大小写处理
与扩展排序，不等同于完整的注册表级或 Unicode locale 规范化。

解析器不对总长度或子标签数量设置固定上限，在可用内存允许的情况下支持超过
255 字节的标签。

`NormalTag::from_parts` 从已校验的组件类型构造标签，遇到重复变体或重复扩展
singleton 时返回 `BuildError`。`Extension::new` 和 `PrivateUseTag::new` 也使用
`BuildError` 报告非法组件组合。构造错误描述组件值或序列索引；只有字符串解析
使用带输入字节偏移的 `ParseError`。解析、组件构造和派生变换保持相同的不变量，
其输出再次解析后仍得到相同的值。

## 诊断与排序

词法校验先于语法校验。扫描从左向右进行，遇到非法字符即报告，在分隔符或输入
结尾检查空子标签和过长子标签；首先检测到的词法错误优先。语法错误取决于解析
入口和仍可使用的语法位置。例如，完整标签中的第二个 singleton 会开始新的扩展，
但单个 `Extension` 的解析入口会将其视为多余子标签。`PrivateUseTag::parse` 在
词法校验后要求 `x` 前缀，缺失时报告 `MissingPrivateUsePrefix`。

排序采用结构顺序，而非输出字符串顺序。`NormalTag` 依次比较语言、文字、地区、
变体、扩展和私用序列；`None` 排在 `Some` 前，序列按元素字典序比较。因此
`en-US` 排在 `en-Latn` 前。`Tag` 将普通标签排在纯私用标签前。

## 可选 Serde 支持

启用 `serde` 特性后，全部核心类型（`Tag`、`NormalTag`、`PrivateUseTag`、`Extension`
及五种原子子标签类型）均支持字符串序列化与反序列化。序列化使用 `Display`
输出的规范化形式，反序列化通过 `parse` 校验并规范化：

```toml
[dependencies]
fy-i18n-lang-tag = { version = "0.1", features = ["serde"] }
```

## 许可

可选择以下任一许可证使用：

- [Apache License, Version 2.0](LICENSE-APACHE)
- [MIT license](LICENSE-MIT)
