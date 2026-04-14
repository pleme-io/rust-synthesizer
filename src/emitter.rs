use crate::node::RustNode;

/// Emit a sequence of top-level Rust nodes as a complete source file.
///
/// Deterministic: identical ASTs produce byte-identical output.
/// Always ends with exactly one trailing newline.
#[must_use]
pub fn emit_file(nodes: &[RustNode]) -> String {
    let mut lines: Vec<String> = Vec::with_capacity(nodes.len());
    for node in nodes {
        lines.push(node.emit(0));
    }
    let mut out = lines.join("\n");
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_file() {
        assert_eq!(emit_file(&[]), "\n");
    }

    #[test]
    fn trailing_newline() {
        let out = emit_file(&[RustNode::Comment("test".into())]);
        assert!(out.ends_with('\n'));
    }

    #[test]
    fn deterministic() {
        let nodes = vec![
            RustNode::Comment("test".into()),
            RustNode::Blank,
            RustNode::Int(42),
        ];
        let a = emit_file(&nodes);
        let b = emit_file(&nodes);
        assert_eq!(a, b);
    }
}
