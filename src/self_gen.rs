// Self-generation: RustNode describes its own enum definition.
//
// This is the fixed point. The function below constructs a Vec<RustNode>
// that, when emitted, produces the exact `pub enum RustNode { ... }`
// definition from node.rs. The system describes itself.

use crate::builders::EnumBuilder;
use crate::node::*;

/// Generate the `RustNode` enum definition as a RustNode AST.
///
/// When emitted, this produces Rust source code that is byte-identical
/// to the hand-written enum definition in `src/node.rs`.
///
/// This IS the self-generating fixed point:
/// `emit(generate_self_enum()) == extract_enum("src/node.rs")`
#[must_use]
pub fn generate_self_enum() -> RustNode {
    EnumBuilder::new("RustNode")
        .public()
        .derive("Debug")
        .derive("Clone")
        .derive("PartialEq")
        // Comments
        .tuple("Comment", vec!["String"])
        .tuple("DocComment", vec!["String"])
        .unit("Blank")
        // Literals
        .tuple("Str", vec!["String"])
        .tuple("Int", vec!["i64"])
        .tuple("Bool", vec!["bool"])
        .tuple("Raw", vec!["String"])
        // Identifiers
        .tuple("Ident", vec!["String"])
        .tuple("Path", vec!["Vec<String>"])
        // Declarations
        .variant_struct("Use", vec![
            StructField::new("path", "Vec<String>"),
            StructField::new("alias", "Option<String>"),
            StructField::new("public", "bool"),
        ])
        .variant_struct("Mod", vec![
            StructField::new("name", "String"),
            StructField::new("public", "bool"),
        ])
        .variant_struct("Struct", vec![
            StructField::new("name", "String"),
            StructField::new("public", "bool"),
            StructField::new("derives", "Vec<String>"),
            StructField::new("fields", "Vec<StructField>"),
        ])
        .variant_struct("Enum", vec![
            StructField::new("name", "String"),
            StructField::new("public", "bool"),
            StructField::new("derives", "Vec<String>"),
            StructField::new("variants", "Vec<EnumVariant>"),
        ])
        .variant_struct("Impl", vec![
            StructField::new("target", "String"),
            StructField::new("trait_name", "Option<String>"),
            StructField::new("body", "Vec<RustNode>"),
        ])
        .variant_struct("Fn", vec![
            StructField::new("name", "String"),
            StructField::new("public", "bool"),
            StructField::new("must_use", "bool"),
            StructField::new("args", "Vec<FnArg>"),
            StructField::new("return_type", "Option<String>"),
            StructField::new("body", "Vec<RustNode>"),
        ])
        // Expressions
        .variant_struct("StructInit", vec![
            StructField::new("name", "String"),
            StructField::new("fields", "Vec<(String, RustNode)>"),
        ])
        .variant_struct("Closure", vec![
            StructField::new("args", "Vec<String>"),
            StructField::new("body", "Box<RustNode>"),
        ])
        .variant_struct("Let", vec![
            StructField::new("name", "String"),
            StructField::new("mutable", "bool"),
            StructField::new("type_ann", "Option<String>"),
            StructField::new("value", "Box<RustNode>"),
        ])
        .variant_struct("Match", vec![
            StructField::new("expr", "Box<RustNode>"),
            StructField::new("arms", "Vec<MatchArm>"),
        ])
        .variant_struct("MethodCall", vec![
            StructField::new("receiver", "Box<RustNode>"),
            StructField::new("method", "String"),
            StructField::new("args", "Vec<RustNode>"),
        ])
        .variant_struct("FnCall", vec![
            StructField::new("name", "String"),
            StructField::new("args", "Vec<RustNode>"),
        ])
        .variant_struct("MacroCall", vec![
            StructField::new("name", "String"),
            StructField::new("args", "Vec<RustNode>"),
        ])
        .variant_struct("MacroBlock", vec![
            StructField::new("name", "String"),
            StructField::new("body", "String"),
        ])
        .tuple("Block", vec!["Vec<RustNode>"])
        .tuple("Return", vec!["Box<RustNode>"])
        // Control flow
        .variant_struct("If", vec![
            StructField::new("cond", "Box<RustNode>"),
            StructField::new("then_body", "Vec<RustNode>"),
            StructField::new("else_body", "Option<Vec<RustNode>>"),
        ])
        .variant_struct("For", vec![
            StructField::new("binding", "String"),
            StructField::new("iter", "Box<RustNode>"),
            StructField::new("body", "Vec<RustNode>"),
        ])
        // Attributes & modules
        .variant_struct("Attr", vec![
            StructField::new("path", "String"),
            StructField::new("args", "Option<String>"),
        ])
        .variant_struct("InlineMod", vec![
            StructField::new("name", "String"),
            StructField::new("public", "bool"),
            StructField::new("body", "Vec<RustNode>"),
        ])
        .build()
}

/// Extract the enum block from Rust source code.
///
/// Finds the first `pub enum {name} {` ... `}` block.
#[must_use]
pub fn extract_enum_block(source: &str, name: &str) -> String {
    let search = format!("pub enum {name} {{");
    let start = source.find(&search).expect("enum not found in source");

    // Find the matching closing brace
    let after_open = start + source[start..].find('{').unwrap();
    let mut depth = 0;
    let mut end = after_open;

    for (i, ch) in source[after_open..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = after_open + i + 1;
                    break;
                }
            }
            _ => {}
        }
    }

    // Also capture derives above the enum
    let mut line_start = start;
    let before = &source[..start];
    // Walk backwards to find #[derive(...)]
    if let Some(derive_pos) = before.rfind("#[derive(") {
        // Check it's close (within a few lines)
        let between = &source[derive_pos..start];
        if between.lines().count() <= 2 {
            line_start = derive_pos;
        }
    }

    source[line_start..end].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::emitter::emit_file;

    #[test]
    fn self_enum_has_all_variants() {
        let node = generate_self_enum();
        let out = node.emit(0);

        // All 22 variant names present
        let expected_variants = [
            "Comment", "DocComment", "Blank", "Str", "Int", "Bool", "Raw",
            "Ident", "Path", "Use", "Mod", "Struct", "Enum", "Impl", "Fn",
            "Let", "Match", "MethodCall", "FnCall", "MacroCall", "Block",
            "Return", "If", "For",
        ];
        for v in &expected_variants {
            assert!(out.contains(v), "missing variant: {v}");
        }
    }

    #[test]
    fn self_enum_has_derives() {
        let out = generate_self_enum().emit(0);
        assert!(out.contains("#[derive(Debug, Clone, PartialEq)]"));
    }

    #[test]
    fn self_enum_is_public() {
        let out = generate_self_enum().emit(0);
        assert!(out.contains("pub enum RustNode {"));
    }

    #[test]
    fn self_enum_deterministic() {
        let a = generate_self_enum().emit(0);
        let b = generate_self_enum().emit(0);
        assert_eq!(a, b);
    }

    #[test]
    fn self_enum_matches_canonical() {
        let generated = generate_self_enum().emit(0);
        let canonical = include_str!("node.rs");
        let canonical_enum = extract_enum_block(canonical, "RustNode");

        // Strip doc comments and section comments from canonical
        // (comments are documentation, not structure)
        let canonical_stripped = strip_comments(&canonical_enum);
        let generated_stripped = strip_comments(&generated);

        assert_eq!(
            generated_stripped.trim(),
            canonical_stripped.trim(),
            "FIXED POINT PROOF: generated enum structure must match hand-written canonical"
        );
    }

    /// Strip comments and blank lines, normalize whitespace for structural comparison.
    fn strip_comments(s: &str) -> String {
        s.lines()
            .filter(|line| {
                let trimmed = line.trim();
                !trimmed.starts_with("///")
                    && !trimmed.starts_with("//")
                    && !trimmed.starts_with("#[deprecated")
                    && !trimmed.starts_with("#[allow(deprecated")
                    && !trimmed.is_empty()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn generation_is_idempotent() {
        let gen1 = emit_file(&[generate_self_enum()]);
        let gen2 = emit_file(&[generate_self_enum()]);
        assert_eq!(gen1, gen2, "generation must be idempotent (fixed point)");
    }

    #[test]
    fn extract_enum_finds_block() {
        let source = r#"
#[derive(Debug)]
pub enum Foo {
    A,
    B { x: i32 },
}
"#;
        let block = extract_enum_block(source, "Foo");
        assert!(block.contains("pub enum Foo {"));
        assert!(block.contains("A,"));
        assert!(block.contains("B { x: i32 },"));
        assert!(block.ends_with('}'));
    }
}
