//! Conformance to [`synthesizer_core`] traits.
//!
//! Wave 2 of the compound-knowledge refactor: purely additive. No behavior
//! change to rust-synthesizer's existing APIs — this module only adds trait
//! impls that downstream generic code can consume.

use crate::node::RustNode;
use synthesizer_core::{NoRawAttestation, SynthesizerNode};

impl SynthesizerNode for RustNode {
    fn emit(&self, indent: usize) -> String {
        // Delegate to the inherent `RustNode::emit` — inherent takes
        // priority over trait methods in UFCS path lookup.
        RustNode::emit(self, indent)
    }

    fn indent_unit() -> &'static str {
        // rustfmt convention is 4 spaces, and `src/node.rs` uses
        // `"    ".repeat(indent)` as the pad. Honor that here so
        // `honors_indent_unit` law holds.
        "    "
    }

    fn variant_id(&self) -> u8 {
        match self {
            // Comments
            Self::Comment(_) => 0,
            Self::DocComment(_) => 1,
            Self::Blank => 2,

            // Literals
            Self::Str(_) => 3,
            Self::Int(_) => 4,
            Self::Bool(_) => 5,
            #[allow(deprecated)]
            Self::Raw(_) => 6,

            // Identifiers
            Self::Ident(_) => 7,
            Self::Path(_) => 8,

            // Declarations
            Self::Use { .. } => 9,
            Self::Mod { .. } => 10,
            Self::Struct { .. } => 11,
            Self::Enum { .. } => 12,
            Self::Impl { .. } => 13,
            Self::Fn { .. } => 14,

            // Expressions
            Self::StructInit { .. } => 15,
            Self::Closure { .. } => 16,
            Self::Let { .. } => 17,
            Self::Match { .. } => 18,
            Self::MethodCall { .. } => 19,
            Self::FnCall { .. } => 20,
            Self::MacroCall { .. } => 21,
            Self::MacroBlock { .. } => 22,
            Self::Block(_) => 23,
            Self::Return(_) => 24,

            // Control flow
            Self::If { .. } => 25,
            Self::For { .. } => 26,

            // Attributes & inline modules
            Self::Attr { .. } => 27,
            Self::InlineMod { .. } => 28,
        }
    }
}

impl NoRawAttestation for RustNode {
    fn attestation() -> &'static str {
        "RustNode::Raw carries #[deprecated] in src/node.rs and is scheduled \
         for removal in Wave 3 of the compound-knowledge refactor. \
         tests/no_raw_invariant.rs::no_raw_in_production_code scans \
         src/{node,emitter,builders,syn_gen}.rs for Raw constructions; any \
         accidental reintroduction fails CI. The #[allow(deprecated)] pin \
         in synthesizer_core_impl.rs is the one intentional reference — a \
         match arm pattern, not a construction."
    }
}
