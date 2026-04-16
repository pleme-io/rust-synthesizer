//! Integration tests proving `RustNode` conforms to `synthesizer_core` traits.
//!
//! Wave 2 of the compound-knowledge refactor. Every test calls one of
//! `synthesizer_core::node::laws::*` on a real `RustNode` value, compounding
//! proof surface: the same laws prove properties of every synthesizer that
//! conforms.

use rust_synthesizer::{EnumVariant, FnArg, MatchArm, RustNode, StructField};
use synthesizer_core::node::laws;
use synthesizer_core::{NoRawAttestation, SynthesizerNode};

// ─── Trait shape ────────────────────────────────────────────────────

#[test]
fn indent_unit_is_four_spaces() {
    assert_eq!(<RustNode as SynthesizerNode>::indent_unit(), "    ");
}

#[test]
fn variant_ids_distinct_across_disjoint_variants() {
    let samples: Vec<RustNode> = vec![
        RustNode::Comment("c".into()),
        RustNode::DocComment("d".into()),
        RustNode::Blank,
        RustNode::Str("s".into()),
        RustNode::Int(1),
        RustNode::Bool(true),
        RustNode::Ident("x".into()),
        RustNode::Path(vec!["crate".into(), "node".into()]),
        RustNode::Use {
            path: vec!["std".into(), "fmt".into()],
            alias: None,
            public: false,
        },
        RustNode::Mod {
            name: "node".into(),
            public: true,
        },
        RustNode::Block(vec![]),
        RustNode::Return(Box::new(RustNode::Ident("r".into()))),
        RustNode::Attr {
            path: "test".into(),
            args: None,
        },
        RustNode::InlineMod {
            name: "m".into(),
            public: false,
            body: vec![],
        },
        RustNode::MacroBlock {
            name: "proptest".into(),
            body: String::new(),
        },
    ];
    let before = samples.len();
    let mut ids: Vec<u8> = samples.iter().map(SynthesizerNode::variant_id).collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(
        ids.len(),
        before,
        "variant_id must be distinct for disjoint variants"
    );
}

// ─── SynthesizerNode laws ───────────────────────────────────────────

#[test]
fn law_determinism_holds_on_simple_nodes() {
    for n in [
        RustNode::Blank,
        RustNode::Comment("hello".into()),
        RustNode::DocComment("docs".into()),
        RustNode::Str("v".into()),
        RustNode::Int(42),
        RustNode::Bool(false),
        RustNode::Ident("foo".into()),
    ] {
        assert!(laws::is_deterministic(&n, 0));
        assert!(laws::is_deterministic(&n, 3));
    }
}

#[test]
fn law_determinism_holds_on_struct_node() {
    let n = RustNode::Struct {
        name: "Point".into(),
        public: true,
        derives: vec!["Debug".into(), "Clone".into()],
        fields: vec![
            StructField::new("x", "f64"),
            StructField::new("y", "f64"),
        ],
    };
    assert!(laws::is_deterministic(&n, 2));
}

#[test]
fn law_determinism_holds_on_enum_node() {
    let n = RustNode::Enum {
        name: "Shape".into(),
        public: true,
        derives: vec![],
        variants: vec![
            EnumVariant::unit("Circle"),
            EnumVariant::tuple("Square", vec!["f64"]),
        ],
    };
    assert!(laws::is_deterministic(&n, 1));
}

#[test]
fn law_determinism_holds_on_fn_node() {
    let n = RustNode::Fn {
        name: "add".into(),
        public: true,
        must_use: true,
        args: vec![FnArg::new("a", "i64"), FnArg::new("b", "i64")],
        return_type: Some("i64".into()),
        body: vec![RustNode::Return(Box::new(RustNode::Ident("a".into())))],
    };
    assert!(laws::is_deterministic(&n, 0));
    assert!(laws::is_deterministic(&n, 2));
}

#[test]
fn law_determinism_holds_on_match_node() {
    let n = RustNode::Match {
        expr: Box::new(RustNode::Ident("x".into())),
        arms: vec![
            MatchArm::new("1", RustNode::Str("one".into())),
            MatchArm::new("_", RustNode::Str("other".into())),
        ],
    };
    assert!(laws::is_deterministic(&n, 1));
}

#[test]
fn law_honors_indent_unit_on_comment() {
    assert!(laws::honors_indent_unit(
        &RustNode::Comment("hello".into()),
        0
    ));
    assert!(laws::honors_indent_unit(
        &RustNode::Comment("hello".into()),
        2
    ));
}

#[test]
fn law_honors_indent_unit_on_use() {
    let n = RustNode::Use {
        path: vec!["std".into(), "fmt".into()],
        alias: None,
        public: false,
    };
    assert!(laws::honors_indent_unit(&n, 0));
    assert!(laws::honors_indent_unit(&n, 3));
}

#[test]
fn law_indent_monotone_len_on_doc_comment() {
    assert!(laws::indent_monotone_len(
        &RustNode::DocComment("doc".into()),
        0
    ));
    assert!(laws::indent_monotone_len(
        &RustNode::DocComment("doc".into()),
        3
    ));
}

#[test]
fn law_indent_monotone_len_on_mod() {
    let n = RustNode::Mod {
        name: "node".into(),
        public: true,
    };
    assert!(laws::indent_monotone_len(&n, 0));
    assert!(laws::indent_monotone_len(&n, 5));
}

#[test]
fn law_variant_id_valid_on_all_sample_variants() {
    let samples = [
        RustNode::Blank,
        RustNode::Comment("x".into()),
        RustNode::Str("s".into()),
        RustNode::Int(0),
        RustNode::Bool(false),
        RustNode::Ident("i".into()),
        RustNode::Path(vec!["a".into(), "b".into()]),
        RustNode::Block(vec![]),
    ];
    for n in &samples {
        assert!(laws::variant_id_is_valid(n));
    }
}

// ─── NoRawAttestation ───────────────────────────────────────────────

#[test]
fn attestation_is_nonempty() {
    assert!(!<RustNode as NoRawAttestation>::attestation().is_empty());
}

#[test]
fn attestation_mentions_raw() {
    let s = <RustNode as NoRawAttestation>::attestation();
    assert!(
        s.to_lowercase().contains("raw"),
        "attestation must explain how no-raw is enforced — got: {s}"
    );
}

// ─── Delegation parity ──────────────────────────────────────────────

#[test]
fn trait_emit_matches_inherent_emit() {
    // The trait impl is a thin delegation. Prove parity on several
    // representative variants at multiple indent levels.
    let samples = [
        RustNode::Blank,
        RustNode::Comment("test".into()),
        RustNode::Str("hello".into()),
        RustNode::Int(123),
        RustNode::Ident("foo".into()),
        RustNode::Path(vec!["a".into(), "b".into()]),
    ];
    for n in &samples {
        for indent in [0usize, 1, 2, 5] {
            let inherent = RustNode::emit(n, indent);
            let via_trait = <RustNode as SynthesizerNode>::emit(n, indent);
            assert_eq!(
                inherent, via_trait,
                "trait emit must delegate to inherent for {n:?} at indent {indent}"
            );
        }
    }
}
