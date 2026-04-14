# rust-synthesizer

Typed AST for structurally correct Rust source code generation. **Self-generating**: proves it can describe and regenerate its own enum definition (fixed point, 3 independent proofs).

## Tests: 75 | Status: Proven, Self-Generating, Zero Raw in Production

## Core API

`RustNode` — 26 variants:

| Category | Variants |
|----------|----------|
| Comments | Comment, DocComment, Blank |
| Literals | Str, Int, Bool, ~~Raw~~ (deprecated) |
| Identifiers | Ident, Path |
| Declarations | Use (with public), Mod, Struct, Enum, Impl, Fn |
| Expressions | StructInit, Closure, Let, Match, MethodCall, FnCall, MacroCall, Block, Return |
| Control | If, For |

`Raw` is **deprecated** — compiler warning on any usage. No-raw invariant test enforced.

## Builders (fluent, #[must_use])

- `StructBuilder` — `.public().derive("Debug").field("x", "f64").build()`
- `EnumBuilder` — `.public().unit("Blank").tuple("Comment", vec!["String"]).build()`
- `FnBuilder` — `.public().must_use().arg_ref_self().returns("String").build()`
- `ImplBuilder` — `.new("Target").for_trait("Display").method(fn).build()`

## Self-Generation (self_gen.rs)

`generate_self_enum()` → RustNode AST that emits the canonical `pub enum RustNode { ... }`. **Fixed point proven.**

## syn-Based Proofs (syn_gen.rs)

3 independent fixed-point proofs:
1. **String** — emit + strip comments → compare
2. **syn AST** — parse canonical → PartialEq with generated (structural)
3. **prettyplease** — format both → byte-compare

Auto-verification: variant count (26), coverage, companion types, all source parses via syn.

## Validators (validators.rs)

| Parser | Language | Strength |
|--------|----------|----------|
| syn | Rust | Full semantic |
| tree-sitter-rust | Rust | Syntax |
| tree-sitter-json | JSON | Syntax |
| tree-sitter-yaml | YAML | Syntax |

## No-Raw Invariant

`#[deprecated]` on `Raw(String)` + test scanning production source → assert zero constructors.

## Dependencies

syn 2 (full, extra-traits), quote, proc-macro2, prettyplease, tree-sitter + grammars. proptest (dev).
