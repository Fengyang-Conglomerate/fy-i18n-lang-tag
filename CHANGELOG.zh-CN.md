# 更新日志

[English](CHANGELOG.md) | 简体中文

## 0.2.0 — 2026-10-16

本版本重写实现，相较 0.1.0 包含破坏性 API 变更。

### 新增

- 零分配借用视图 `NormalTagView`、`PrivateUseTagView`，支持 `const` 解析。
- 可配置的 `Fixed<N>` 内联存储；可选 `alloc` 特性提供 `Grow<N>` 堆存储。拥有型普通标签使用 15 字节核心存储，并预索引扩展边界以直接投影借用视图。
- 语言、文字、地区、Unicode `u`、转换内容 `t` 及其他扩展的结构化视图，包含扩展专属语法检查。
- 四字母主语言子标签，以及 `t` 扩展中的转换源语言语法。
- 六级 `MatchQuality`、单请求与多请求匹配、结构化回退迭代器。
- 可选 `unsafe` 快速路径；默认构建禁止不安全代码。可选 `serde` 支持借用反序列化。

### 变更及从 0.1 迁移

- 原 `Tag` 枚举改为独立的 `NormalTag<S>`、`PrivateUseTag<S>` 入口；原 `Subtag` 与分量构造 API 由新的视图和存储 API 替代。
- 拥有型解析返回对应模块的 `TagError`，区分语法错误和容量错误；借用型解析返回对应模块的 `ParseError`。
- 使用类型标注明确存储，例如 `let tag: NormalTag = NormalTag::parse("en-US")?;`。默认拥有型存储为 `Fixed<32>`；存储内容较大时选择更大容量或 `Grow`。容量不足会返回错误。
- `script()`、`region()` 返回可选字符串切片；强类型访问改用 `script_view()`、`region_view()`。
- 默认不再依赖 `alloc`；使用 `Grow` 时显式启用该特性。
- 最低 Rust 版本为 1.86.0（Rust 2024 edition），低于 0.1.0 声明的 1.98。

### 验证边界与发布检查

- 规范化包括 ASCII 大小写和扩展单字母排序，私用内容置尾。解析检查所支持的语法，不检查 IANA 注册有效性，不执行完整的 Unicode 语言环境规范化，也不推断缺失分量。
- 修正两份 README 示例，并纳入中英文可执行示例检查。
- CI 覆盖 MSRV、stable、两端特性组合、格式、Clippy、rustdoc、打包、覆盖率及 wasm32/thumbv7m 的 `no_std` 构建。标签推送不重复触发 main CI。

- 版本标签在同一提交的 main CI 通过后，自动生成双语 GitHub Release 说明。

[比较 0.1.0 与 0.2.0](https://github.com/Fengyang-Conglomerate/fy-i18n-lang-tag/compare/v0.1.0...v0.2.0)

## 0.1.0

首次公开发布：提供拥有型现代语言标签解析与规范化、强类型分量、涵盖普通与全私用标签的 `Tag` 枚举、语法诊断和可选 `serde`。支持 `no_std`，但必须依赖 `alloc`；未应用 `u`、`t` 扩展的专属语法。
