# rust-synthesizer

Typed AST for structurally correct Rust source code generation. **Self-generating**: proves it can describe and regenerate its own enum definition (fixed point).

## Tests: 74 | Status: Proven

## Core API

| Type | Purpose |
|------|---------|
| `RustNode` | 24-variant enum: Comment, DocComment, Blank, Str, Int, Bool, Raw, Ident, Path, Use, Mod, Struct, Enum, Impl, Fn, Let, Match, MethodCall, FnCall, MacroCall, Block, Return, If, For |
| `StructField` | Struct field with name, type, visibility, doc |
| `EnumVariant` | Unit / Tuple / Struct shapes |
| `FnArg` | Function argument (supports &self, &mut self) |
| `MatchArm` | Pattern + body |
| `emit_file(&[RustNode])` | Emit nodes as complete Rust source file |

## Builders (fluent, #[must_use])

- `StructBuilder` — `.public().derive("Debug").field("x", "f64").build()`
- `EnumBuilder` — `.public().derive("Debug").unit("Blank").tuple("Comment", vec!["String"]).build()`
- `FnBuilder` — `.public().must_use().arg_ref_self().arg("indent", "usize").returns("String").build()`
- `ImplBuilder` — `.new("Target").for_trait("Display").method(fn_node).build()`

## Self-Generation (self_gen.rs)

`generate_self_enum() -> RustNode` constructs the `RustNode` enum definition as a RustNode AST. When emitted, it is structurally identical to the hand-written enum in `node.rs`.

## syn-Based Proofs (syn_gen.rs)

Uses syn/quote/prettyplease for proper AST-level fixed-point proof:
- `generate_self_syn_enum() -> syn::ItemEnum` via `quote!{}`
- `syn_fixed_point_structural` — parse canonical with syn, strip docs, PartialEq
- `syn_fixed_point_formatted` — prettyplease format both, byte-compare
- Auto-verification: variant count, coverage, companion types, all source files parse

## Validators (validators.rs)

Multi-language validation via syn (Rust semantic) and tree-sitter (JSON/YAML/Rust syntax):
- `RustValidator` — full semantic parse via `syn::parse_file()`
- `TreeSitterValidator` — generic for any tree-sitter grammar
- `validator_for(lang)` — factory function

## Dependencies

- syn 2 (full, extra-traits), quote, proc-macro2, prettyplease
- tree-sitter + tree-sitter-rust/json/yaml
- proptest (dev)

## Proven Properties

- Deterministic emission (proptest + exhaustive)
- Self-generating fixed point (3 independent proofs)
- Multi-language validation (syn + tree-sitter)
- Every RustNode variant tested
- All source files parse via syn
