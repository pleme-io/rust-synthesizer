// Multi-language code validation using syn (Rust) and tree-sitter (everything else).
//
// Every synthesizer's output can be validated by a real parser for that language.
// If it parses, it's structurally valid. If it doesn't, we caught a generation bug.

use std::fmt;

// ── Error type ──────────────────────────────────────────────────────

#[derive(Debug)]
pub struct ValidationError {
    pub language: String,
    pub message: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.line, self.column) {
            (Some(l), Some(c)) => write!(f, "[{}] {}:{}: {}", self.language, l, c, self.message),
            _ => write!(f, "[{}] {}", self.language, self.message),
        }
    }
}

// ── Validator trait ─────────────────────────────────────────────────

/// Validates that generated source code is syntactically correct.
pub trait CodeValidator {
    /// Validate the source. Returns Ok(()) if the code parses without errors.
    fn validate(&self, source: &str) -> Result<(), ValidationError>;

    /// The language this validator handles.
    fn language_name(&self) -> &str;
}

// ── Rust validator (syn — full semantic parse) ──────────────────────

pub struct RustValidator;

impl CodeValidator for RustValidator {
    fn validate(&self, source: &str) -> Result<(), ValidationError> {
        syn::parse_file(source).map_err(|e| ValidationError {
            language: "Rust".into(),
            message: e.to_string(),
            line: None,
            column: None,
        })?;
        Ok(())
    }

    fn language_name(&self) -> &str {
        "Rust"
    }
}

// ── Tree-sitter validator (any grammar) ─────────────────────────────

pub struct TreeSitterValidator {
    language: tree_sitter::Language,
    name: String,
}

impl TreeSitterValidator {
    #[must_use]
    pub fn new(language: tree_sitter::Language, name: &str) -> Self {
        Self {
            language,
            name: name.to_string(),
        }
    }

    /// Create a Rust tree-sitter validator.
    #[must_use]
    pub fn rust() -> Self {
        Self::new(tree_sitter_rust::LANGUAGE.into(), "Rust-TS")
    }

    /// Create a JSON tree-sitter validator.
    #[must_use]
    pub fn json() -> Self {
        Self::new(tree_sitter_json::LANGUAGE.into(), "JSON")
    }

    /// Create a YAML tree-sitter validator.
    #[must_use]
    pub fn yaml() -> Self {
        Self::new(tree_sitter_yaml::language().into(), "YAML")
    }
}

impl CodeValidator for TreeSitterValidator {
    fn validate(&self, source: &str) -> Result<(), ValidationError> {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&self.language).map_err(|e| ValidationError {
            language: self.name.clone(),
            message: format!("failed to set language: {e}"),
            line: None,
            column: None,
        })?;

        let tree = parser.parse(source, None).ok_or_else(|| ValidationError {
            language: self.name.clone(),
            message: "parser returned no tree".into(),
            line: None,
            column: None,
        })?;

        let root = tree.root_node();

        // Check for ERROR nodes in the parse tree
        if root.has_error() {
            // Find the first error node for location info
            let mut cursor = root.walk();
            let error_node = find_error_node(&mut cursor);

            return Err(ValidationError {
                language: self.name.clone(),
                message: format!(
                    "parse error: {} node at position",
                    error_node
                        .map(|n| n.kind().to_string())
                        .unwrap_or_else(|| "unknown".into())
                ),
                line: error_node.map(|n| n.start_position().row + 1),
                column: error_node.map(|n| n.start_position().column),
            });
        }

        Ok(())
    }

    fn language_name(&self) -> &str {
        &self.name
    }
}

/// Walk a tree-sitter cursor to find the first ERROR node.
fn find_error_node<'a>(cursor: &mut tree_sitter::TreeCursor<'a>) -> Option<tree_sitter::Node<'a>> {
    loop {
        let node = cursor.node();
        if node.is_error() || node.is_missing() {
            return Some(node);
        }
        if cursor.goto_first_child() {
            if let Some(n) = find_error_node(cursor) {
                return Some(n);
            }
            cursor.goto_parent();
        }
        if !cursor.goto_next_sibling() {
            return None;
        }
    }
}

// ── Factory ─────────────────────────────────────────────────────────

/// Get a validator for the given language name.
#[must_use]
pub fn validator_for(language: &str) -> Box<dyn CodeValidator> {
    match language.to_lowercase().as_str() {
        "rust" | "rs" => Box::new(RustValidator),
        "json" => Box::new(TreeSitterValidator::json()),
        "yaml" | "yml" => Box::new(TreeSitterValidator::yaml()),
        "rust-ts" => Box::new(TreeSitterValidator::rust()),
        _ => panic!("no validator for language: {language}"),
    }
}

/// All available validator language names.
pub fn available_validators() -> Vec<&'static str> {
    vec!["rust", "json", "yaml", "rust-ts"]
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Rust validation (syn) ───────────────────────────────────────

    #[test]
    fn rust_validator_accepts_valid() {
        let v = RustValidator;
        assert!(v.validate("fn main() {}").is_ok());
    }

    #[test]
    fn rust_validator_rejects_invalid() {
        let v = RustValidator;
        assert!(v.validate("fn main( {}").is_err());
    }

    #[test]
    fn rust_validator_accepts_struct() {
        let v = RustValidator;
        assert!(v.validate("#[derive(Debug)]\npub struct Foo { pub x: i32 }").is_ok());
    }

    #[test]
    fn rust_validator_accepts_enum() {
        let v = RustValidator;
        assert!(v.validate("pub enum Color { Red, Green, Blue }").is_ok());
    }

    #[test]
    fn rust_validator_accepts_impl() {
        let v = RustValidator;
        assert!(v.validate("impl Foo { fn new() -> Self { Self } }").is_ok());
    }

    // ── tree-sitter Rust validation ─────────────────────────────────

    #[test]
    fn ts_rust_accepts_valid() {
        let v = TreeSitterValidator::rust();
        assert!(v.validate("fn main() {}").is_ok());
    }

    #[test]
    fn ts_rust_rejects_invalid() {
        let v = TreeSitterValidator::rust();
        assert!(v.validate("fn main( {}").is_err());
    }

    // ── JSON validation ─────────────────────────────────────────────

    #[test]
    fn json_accepts_valid() {
        let v = TreeSitterValidator::json();
        assert!(v.validate(r#"{"key": "value", "num": 42}"#).is_ok());
    }

    #[test]
    fn json_accepts_array() {
        let v = TreeSitterValidator::json();
        assert!(v.validate("[1, 2, 3]").is_ok());
    }

    #[test]
    fn json_rejects_invalid() {
        let v = TreeSitterValidator::json();
        assert!(v.validate("{key: value}").is_err());
    }

    // ── YAML validation ─────────────────────────────────────────────

    #[test]
    fn yaml_accepts_valid() {
        let v = TreeSitterValidator::yaml();
        assert!(v.validate("key: value\nlist:\n  - item1\n  - item2\n").is_ok());
    }

    #[test]
    fn yaml_accepts_simple() {
        let v = TreeSitterValidator::yaml();
        assert!(v.validate("name: test\nversion: 1.0\n").is_ok());
    }

    // ── Factory ─────────────────────────────────────────────────────

    #[test]
    fn factory_returns_validators() {
        for lang in available_validators() {
            let v = validator_for(lang);
            assert!(!v.language_name().is_empty());
        }
    }

    // ── Validate rust-synthesizer's own output ──────────────────────

    #[test]
    #[allow(deprecated)]
    fn rust_synthesizer_emit_produces_valid_rust() {
        use crate::node::*;
        use crate::emitter::emit_file;
        use crate::builders::*;

        let v = RustValidator;

        // Generate a complete Rust file with every node type
        let nodes = vec![
            RustNode::Comment("Auto-generated".into()),
            RustNode::Blank,
            RustNode::Use { path: vec!["std".into(), "fmt".into()], alias: None, public: false },
            RustNode::Blank,
            StructBuilder::new("Point")
                .public()
                .derive("Debug")
                .derive("Clone")
                .field("x", "f64")
                .field("y", "f64")
                .build(),
            RustNode::Blank,
            EnumBuilder::new("Shape")
                .public()
                .derive("Debug")
                .unit("Circle")
                .tuple("Rect", vec!["f64", "f64"])
                .variant_struct("Polygon", vec![
                    StructField::new("sides", "Vec<f64>"),
                ])
                .build(),
            RustNode::Blank,
            ImplBuilder::new("Point")
                .method(
                    FnBuilder::new("origin")
                        .public()
                        .must_use()
                        .returns("Self")
                        .stmt(RustNode::raw("Self { x: 0.0, y: 0.0 }"))
                        .build(),
                )
                .method(
                    FnBuilder::new("distance")
                        .public()
                        .must_use()
                        .arg_ref_self()
                        .arg("other", "&Point")
                        .returns("f64")
                        .stmt(RustNode::Let {
                            name: "dx".into(),
                            mutable: false,
                            type_ann: None,
                            value: Box::new(RustNode::raw("self.x - other.x")),
                        })
                        .stmt(RustNode::Let {
                            name: "dy".into(),
                            mutable: false,
                            type_ann: None,
                            value: Box::new(RustNode::raw("self.y - other.y")),
                        })
                        .stmt(RustNode::raw("(dx * dx + dy * dy).sqrt()"))
                        .build(),
                )
                .build(),
        ];

        let output = emit_file(&nodes);
        let result = v.validate(&output);
        assert!(
            result.is_ok(),
            "rust-synthesizer output must be valid Rust: {:?}\n---\n{}",
            result.err(),
            output
        );
    }

    #[test]
    fn self_generated_enum_is_valid_rust() {
        let v = RustValidator;
        let generated = crate::self_gen::generate_self_enum();
        let output = generated.emit(0);
        // Wrap in a file context for syn to parse
        let full = format!(
            "struct StructField {{ name: String, field_type: String, public: bool, doc: Option<String> }}\n\
             enum EnumVariant {{ Unit(String), Tuple(String, Vec<String>), Struct(String, Vec<StructField>) }}\n\
             struct FnArg {{ name: String, arg_type: String }}\n\
             struct MatchArm {{ pattern: String, body: RustNode }}\n\
             {output}"
        );
        assert!(v.validate(&full).is_ok(), "self-generated enum must be valid Rust");
    }
}
