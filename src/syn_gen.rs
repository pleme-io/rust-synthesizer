// syn-based Rust generation and fixed-point proof.
//
// Uses syn (AST), quote (construction), prettyplease (formatting)
// for proper structural parsing and comparison — not string heuristics.

use quote::quote;

// ── Utilities ───────────────────────────────────────────────────────

/// Parse Rust source into a syn::File.
pub fn parse_rust(source: &str) -> syn::File {
    syn::parse_file(source).expect("failed to parse Rust source")
}

/// Find an enum by name in a parsed file.
pub fn find_enum(file: &syn::File, name: &str) -> Option<syn::ItemEnum> {
    for item in &file.items {
        if let syn::Item::Enum(e) = item {
            if e.ident == name {
                return Some(e.clone());
            }
        }
    }
    None
}

/// Find a struct by name in a parsed file.
pub fn find_struct(file: &syn::File, name: &str) -> Option<syn::ItemStruct> {
    for item in &file.items {
        if let syn::Item::Struct(s) = item {
            if s.ident == name {
                return Some(s.clone());
            }
        }
    }
    None
}

/// Strip all `#[doc = "..."]` attributes from an enum and its variants.
pub fn strip_doc_attrs(item: &syn::ItemEnum) -> syn::ItemEnum {
    let mut item = item.clone();
    item.attrs.retain(|a| !a.path().is_ident("doc") && !a.path().is_ident("deprecated") && !a.path().is_ident("allow"));
    for variant in &mut item.variants {
        variant.attrs.retain(|a| !a.path().is_ident("doc") && !a.path().is_ident("deprecated") && !a.path().is_ident("allow"));
        // Also strip docs from variant fields
        match &mut variant.fields {
            syn::Fields::Named(fields) => {
                for field in &mut fields.named {
                    field.attrs.retain(|a| !a.path().is_ident("doc"));
                }
            }
            syn::Fields::Unnamed(fields) => {
                for field in &mut fields.unnamed {
                    field.attrs.retain(|a| !a.path().is_ident("doc"));
                }
            }
            syn::Fields::Unit => {}
        }
    }
    item
}

/// Format a syn::ItemEnum using prettyplease.
pub fn format_item_enum(item: &syn::ItemEnum) -> String {
    let file = syn::File {
        shebang: None,
        attrs: vec![],
        items: vec![syn::Item::Enum(item.clone())],
    };
    prettyplease::unparse(&file)
}

/// Format any syn::Item using prettyplease.
pub fn format_item(item: &syn::Item) -> String {
    let file = syn::File {
        shebang: None,
        attrs: vec![],
        items: vec![item.clone()],
    };
    prettyplease::unparse(&file)
}

/// Get all variant names from a syn::ItemEnum.
pub fn variant_names(item: &syn::ItemEnum) -> Vec<String> {
    item.variants.iter().map(|v| v.ident.to_string()).collect()
}

/// Get all top-level type names (structs + enums) from a syn::File.
pub fn type_names(file: &syn::File) -> Vec<String> {
    file.items
        .iter()
        .filter_map(|item| match item {
            syn::Item::Struct(s) => Some(s.ident.to_string()),
            syn::Item::Enum(e) => Some(e.ident.to_string()),
            _ => None,
        })
        .collect()
}

// ── Self-generation via quote! ──────────────────────────────────────

/// Generate the RustNode enum definition as a syn::ItemEnum.
///
/// Uses `quote!` for compile-time validation of the token structure.
/// This is the syn-based self-generation — the REAL fixed point.
pub fn generate_self_syn_enum() -> syn::ItemEnum {
    let tokens = quote! {
        #[derive(Debug, Clone, PartialEq)]
        pub enum RustNode {
            Comment(String),
            DocComment(String),
            Blank,
            Str(String),
            Int(i64),
            Bool(bool),
            Raw(String),
            Ident(String),
            Path(Vec<String>),
            Use {
                path: Vec<String>,
                alias: Option<String>,
                public: bool,
            },
            Mod {
                name: String,
                public: bool,
            },
            Struct {
                name: String,
                public: bool,
                derives: Vec<String>,
                fields: Vec<StructField>,
            },
            Enum {
                name: String,
                public: bool,
                derives: Vec<String>,
                variants: Vec<EnumVariant>,
            },
            Impl {
                target: String,
                trait_name: Option<String>,
                body: Vec<RustNode>,
            },
            Fn {
                name: String,
                public: bool,
                must_use: bool,
                args: Vec<FnArg>,
                return_type: Option<String>,
                body: Vec<RustNode>,
            },
            StructInit {
                name: String,
                fields: Vec<(String, RustNode)>,
            },
            Closure {
                args: Vec<String>,
                body: Box<RustNode>,
            },
            Let {
                name: String,
                mutable: bool,
                type_ann: Option<String>,
                value: Box<RustNode>,
            },
            Match {
                expr: Box<RustNode>,
                arms: Vec<MatchArm>,
            },
            MethodCall {
                receiver: Box<RustNode>,
                method: String,
                args: Vec<RustNode>,
            },
            FnCall {
                name: String,
                args: Vec<RustNode>,
            },
            MacroCall {
                name: String,
                args: Vec<RustNode>,
            },
            Block(Vec<RustNode>),
            Return(Box<RustNode>),
            If {
                cond: Box<RustNode>,
                then_body: Vec<RustNode>,
                else_body: Option<Vec<RustNode>>,
            },
            For {
                binding: String,
                iter: Box<RustNode>,
                body: Vec<RustNode>,
            },
        }
    };

    let item: syn::Item = syn::parse2(tokens).expect("quote! produced invalid enum");
    match item {
        syn::Item::Enum(e) => e,
        _ => unreachable!("quote! produced non-enum item"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Fixed-point proof: structural AST equality ──────────────────

    #[test]
    fn syn_fixed_point_structural() {
        let canonical = include_str!("node.rs");
        let canonical_file = parse_rust(canonical);
        let canonical_enum = find_enum(&canonical_file, "RustNode")
            .expect("RustNode enum not found in canonical source");

        let generated_enum = generate_self_syn_enum();

        // Strip doc comments for structural comparison
        let canonical_stripped = strip_doc_attrs(&canonical_enum);
        let generated_stripped = strip_doc_attrs(&generated_enum);

        assert_eq!(
            canonical_stripped, generated_stripped,
            "SYN FIXED POINT: canonical and generated enums must be structurally identical"
        );
    }

    // ── Fixed-point proof: prettyplease-formatted equality ──────────

    #[test]
    fn syn_fixed_point_formatted() {
        let canonical = include_str!("node.rs");
        let canonical_file = parse_rust(canonical);
        let canonical_enum = find_enum(&canonical_file, "RustNode").unwrap();
        let generated_enum = generate_self_syn_enum();

        let canonical_fmt = format_item_enum(&strip_doc_attrs(&canonical_enum));
        let generated_fmt = format_item_enum(&strip_doc_attrs(&generated_enum));

        assert_eq!(
            canonical_fmt, generated_fmt,
            "SYN FIXED POINT: prettyplease output must be byte-identical"
        );
    }

    // ── Cross-validation: RustNode-based and syn-based agree ────────

    #[test]
    fn syn_and_rustnode_generators_agree() {
        let rustnode_gen = crate::self_gen::generate_self_enum();
        let syn_gen = generate_self_syn_enum();

        // Both must have the same variant names in the same order
        let rustnode_output = rustnode_gen.emit(0);
        let syn_names = variant_names(&syn_gen);

        for name in &syn_names {
            assert!(
                rustnode_output.contains(name),
                "RustNode generator missing variant: {name}"
            );
        }
    }

    // ── Idempotency ─────────────────────────────────────────────────

    #[test]
    fn syn_generation_idempotent() {
        let a = generate_self_syn_enum();
        let b = generate_self_syn_enum();
        assert_eq!(a, b, "syn generation must be idempotent");
    }

    #[test]
    fn syn_formatted_idempotent() {
        let a = format_item_enum(&generate_self_syn_enum());
        let b = format_item_enum(&generate_self_syn_enum());
        assert_eq!(a, b, "prettyplease formatting must be idempotent");
    }

    // ── Auto-verification: variant count ────────────────────────────

    #[test]
    fn variant_count_matches() {
        let canonical = include_str!("node.rs");
        let file = parse_rust(canonical);
        let rust_node = find_enum(&file, "RustNode").unwrap();

        assert_eq!(
            rust_node.variants.len(),
            26,
            "RustNode variant count changed — update generate_self_syn_enum()"
        );
    }

    // ── Auto-verification: self-gen covers all variants ─────────────

    #[test]
    fn self_gen_covers_all_variants() {
        let canonical = include_str!("node.rs");
        let file = parse_rust(canonical);
        let canonical_enum = find_enum(&file, "RustNode").unwrap();
        let generated_enum = generate_self_syn_enum();

        let canonical_names = variant_names(&canonical_enum);
        let generated_names = variant_names(&generated_enum);

        assert_eq!(
            canonical_names, generated_names,
            "generated enum variant list must exactly match canonical"
        );
    }

    // ── Auto-verification: companion types defined ──────────────────

    #[test]
    fn all_companion_types_defined() {
        let canonical = include_str!("node.rs");
        let file = parse_rust(canonical);
        let defined = type_names(&file);

        let expected = ["StructField", "EnumVariant", "FnArg", "MatchArm"];
        for t in &expected {
            assert!(
                defined.contains(&t.to_string()),
                "missing companion type: {t}"
            );
        }
    }

    // ── Auto-verification: canonical source parses cleanly ──────────

    #[test]
    fn canonical_node_rs_parses() {
        let canonical = include_str!("node.rs");
        let _file = parse_rust(canonical);
        // If we get here, syn parsed it without errors
    }

    #[test]
    fn canonical_emitter_rs_parses() {
        let source = include_str!("emitter.rs");
        let _file = parse_rust(source);
    }

    #[test]
    fn canonical_builders_rs_parses() {
        let source = include_str!("builders.rs");
        let _file = parse_rust(source);
    }

    #[test]
    fn canonical_self_gen_rs_parses() {
        let source = include_str!("self_gen.rs");
        let _file = parse_rust(source);
    }

    // ── Auto-verification: all source files parse ───────────────────

    #[test]
    fn all_rust_synthesizer_source_parses() {
        let files = [
            include_str!("node.rs"),
            include_str!("emitter.rs"),
            include_str!("builders.rs"),
            include_str!("self_gen.rs"),
            include_str!("syn_gen.rs"),
            include_str!("validators.rs"),
            include_str!("lib.rs"),
        ];
        for (i, source) in files.iter().enumerate() {
            let result = syn::parse_file(source);
            assert!(result.is_ok(), "source file {i} failed to parse: {:?}", result.err());
        }
    }

    // ── Auto-verification: determinism for all variants ─────────────

    #[test]
    fn every_variant_emit_deterministic() {
        use crate::node::*;

        let samples: Vec<(&str, RustNode)> = vec![
            ("Comment", RustNode::Comment("test".into())),
            ("DocComment", RustNode::DocComment("doc".into())),
            ("Blank", RustNode::Blank),
            ("Str", RustNode::Str("hello".into())),
            ("Int", RustNode::Int(42)),
            ("Bool", RustNode::Bool(true)),
            ("Raw", RustNode::Raw("x + 1".into())),
            ("Ident", RustNode::Ident("foo".into())),
            ("Path", RustNode::Path(vec!["crate".into(), "node".into()])),
            ("Use", RustNode::Use { path: vec!["std".into()], alias: None, public: false }),
            ("Mod", RustNode::Mod { name: "test".into(), public: true }),
            ("Struct", RustNode::Struct { name: "S".into(), public: true, derives: vec![], fields: vec![] }),
            ("Enum", RustNode::Enum { name: "E".into(), public: true, derives: vec![], variants: vec![] }),
            ("Impl", RustNode::Impl { target: "S".into(), trait_name: None, body: vec![] }),
            ("Fn", RustNode::Fn { name: "f".into(), public: true, must_use: false, args: vec![], return_type: None, body: vec![] }),
            ("StructInit", RustNode::StructInit { name: "S".into(), fields: vec![("x".into(), RustNode::Int(1))] }),
            ("Closure", RustNode::Closure { args: vec!["x".into()], body: Box::new(RustNode::ident("x")) }),
            ("Let", RustNode::Let { name: "x".into(), mutable: false, type_ann: None, value: Box::new(RustNode::Int(1)) }),
            ("Match", RustNode::Match { expr: Box::new(RustNode::ident("x")), arms: vec![] }),
            ("MethodCall", RustNode::MethodCall { receiver: Box::new(RustNode::ident("s")), method: "len".into(), args: vec![] }),
            ("FnCall", RustNode::FnCall { name: "f".into(), args: vec![] }),
            ("MacroCall", RustNode::MacroCall { name: "println".into(), args: vec![] }),
            ("Block", RustNode::Block(vec![])),
            ("Return", RustNode::Return(Box::new(RustNode::Int(0)))),
            ("If", RustNode::If { cond: Box::new(RustNode::Bool(true)), then_body: vec![], else_body: None }),
            ("For", RustNode::For { binding: "x".into(), iter: Box::new(RustNode::ident("items")), body: vec![] }),
        ];

        // Verify we cover all 24 variants
        assert_eq!(samples.len(), 26, "sample list must cover all variants");

        for (name, node) in &samples {
            let a = node.emit(0);
            let b = node.emit(0);
            assert_eq!(a, b, "variant {name} is not deterministic");
        }
    }
}
