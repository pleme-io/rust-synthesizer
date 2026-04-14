/// Every Rust language construct that rust-synthesizer can emit.
///
/// Nodes are pure data. `emit()` is deterministic: identical ASTs
/// produce byte-identical Rust source. 4-space indentation.

// ── AST Node ────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum RustNode {
    // ── Comments ────────────────────────────────────────────────────
    /// `// text`
    Comment(String),
    /// `/// text`
    DocComment(String),
    /// Empty line
    Blank,

    // ── Literals ────────────────────────────────────────────────────
    /// `"text"` (auto-escapes)
    Str(String),
    /// Integer literal
    Int(i64),
    /// `true` / `false`
    Bool(bool),
    /// Raw expression (escape hatch)
    Raw(String),

    // ── Identifiers ─────────────────────────────────────────────────
    /// Bare identifier: `String`, `self`, `usize`
    Ident(String),
    /// Qualified path: `crate::node::RustNode`
    Path(Vec<String>),

    // ── Declarations ────────────────────────────────────────────────
    /// `use crate::types::RustType;` or `pub use node::*;`
    Use {
        path: Vec<String>,
        alias: Option<String>,
        public: bool,
    },
    /// `pub mod node;`
    Mod {
        name: String,
        public: bool,
    },
    /// Struct definition
    Struct {
        name: String,
        public: bool,
        derives: Vec<String>,
        fields: Vec<StructField>,
    },
    /// Enum definition
    Enum {
        name: String,
        public: bool,
        derives: Vec<String>,
        variants: Vec<EnumVariant>,
    },
    /// `impl Target { ... }` or `impl Trait for Target { ... }`
    Impl {
        target: String,
        trait_name: Option<String>,
        body: Vec<RustNode>,
    },
    /// Function / method definition
    Fn {
        name: String,
        public: bool,
        must_use: bool,
        args: Vec<FnArg>,
        return_type: Option<String>,
        body: Vec<RustNode>,
    },

    // ── Expressions ─────────────────────────────────────────────────
    /// `StructName { field: value, ... }` — struct initialization
    StructInit {
        name: String,
        fields: Vec<(String, RustNode)>,
    },
    /// `|arg| body` — closure expression
    Closure {
        args: Vec<String>,
        body: Box<RustNode>,
    },
    /// `let name: Type = value;`
    Let {
        name: String,
        mutable: bool,
        type_ann: Option<String>,
        value: Box<RustNode>,
    },
    /// `match expr { arms }`
    Match {
        expr: Box<RustNode>,
        arms: Vec<MatchArm>,
    },
    /// `receiver.method(args)`
    MethodCall {
        receiver: Box<RustNode>,
        method: String,
        args: Vec<RustNode>,
    },
    /// `name(args)`
    FnCall {
        name: String,
        args: Vec<RustNode>,
    },
    /// `name!(args)` — format!, vec!, assert!, etc.
    MacroCall {
        name: String,
        args: Vec<RustNode>,
    },
    /// `{ statements }`
    Block(Vec<RustNode>),
    /// `return expr`
    Return(Box<RustNode>),

    // ── Control flow ────────────────────────────────────────────────
    /// `if cond { then } else { else }`
    If {
        cond: Box<RustNode>,
        then_body: Vec<RustNode>,
        else_body: Option<Vec<RustNode>>,
    },
    /// `for binding in iter { body }`
    For {
        binding: String,
        iter: Box<RustNode>,
        body: Vec<RustNode>,
    },
}

// ── Companion types ─────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct StructField {
    pub name: String,
    pub field_type: String,
    pub public: bool,
    pub doc: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EnumVariant {
    /// `VariantName,`
    Unit(String),
    /// `VariantName(T1, T2),`
    Tuple(String, Vec<String>),
    /// `VariantName { field: Type, ... },`
    Struct(String, Vec<StructField>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FnArg {
    pub name: String,
    pub arg_type: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: String,
    pub body: RustNode,
}

// ── Constructors ────────────────────────────────────────────────────

impl RustNode {
    #[must_use]
    pub fn str(s: &str) -> Self {
        Self::Str(s.to_string())
    }

    #[must_use]
    pub fn ident(s: &str) -> Self {
        Self::Ident(s.to_string())
    }

    #[must_use]
    pub fn path(segments: &[&str]) -> Self {
        Self::Path(segments.iter().map(|s| (*s).to_string()).collect())
    }

    #[must_use]
    pub fn raw(s: &str) -> Self {
        Self::Raw(s.to_string())
    }

    #[must_use]
    pub fn method_call(receiver: Self, method: &str, args: Vec<Self>) -> Self {
        Self::MethodCall {
            receiver: Box::new(receiver),
            method: method.to_string(),
            args,
        }
    }

    #[must_use]
    pub fn fn_call(name: &str, args: Vec<Self>) -> Self {
        Self::FnCall {
            name: name.to_string(),
            args,
        }
    }

    #[must_use]
    pub fn macro_call(name: &str, args: Vec<Self>) -> Self {
        Self::MacroCall {
            name: name.to_string(),
            args,
        }
    }
}

impl StructField {
    #[must_use]
    pub fn new(name: &str, field_type: &str) -> Self {
        Self {
            name: name.to_string(),
            field_type: field_type.to_string(),
            public: true,
            doc: None,
        }
    }

    #[must_use]
    pub fn private(name: &str, field_type: &str) -> Self {
        Self {
            name: name.to_string(),
            field_type: field_type.to_string(),
            public: false,
            doc: None,
        }
    }

    #[must_use]
    pub fn with_doc(mut self, doc: &str) -> Self {
        self.doc = Some(doc.to_string());
        self
    }
}

impl FnArg {
    #[must_use]
    pub fn new(name: &str, arg_type: &str) -> Self {
        Self {
            name: name.to_string(),
            arg_type: arg_type.to_string(),
        }
    }

    #[must_use]
    pub fn ref_self() -> Self {
        Self {
            name: "&self".to_string(),
            arg_type: String::new(),
        }
    }

    #[must_use]
    pub fn mut_self() -> Self {
        Self {
            name: "&mut self".to_string(),
            arg_type: String::new(),
        }
    }
}

impl MatchArm {
    #[must_use]
    pub fn new(pattern: &str, body: RustNode) -> Self {
        Self {
            pattern: pattern.to_string(),
            body,
        }
    }
}

impl EnumVariant {
    #[must_use]
    pub fn unit(name: &str) -> Self {
        Self::Unit(name.to_string())
    }

    #[must_use]
    pub fn tuple(name: &str, types: Vec<&str>) -> Self {
        Self::Tuple(name.to_string(), types.into_iter().map(|s| s.to_string()).collect())
    }

    #[must_use]
    pub fn with_fields(name: &str, fields: Vec<StructField>) -> Self {
        Self::Struct(name.to_string(), fields)
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Unit(n) | Self::Tuple(n, _) | Self::Struct(n, _) => n,
        }
    }
}

// ── Emit ────────────────────────────────────────────────────────────

impl RustNode {
    /// Emit this node as Rust source at the given indentation level.
    /// Each level is 4 spaces.
    #[must_use]
    pub fn emit(&self, indent: usize) -> String {
        let pad = "    ".repeat(indent);
        match self {
            // Comments
            Self::Comment(text) => format!("{pad}// {text}"),
            Self::DocComment(text) => format!("{pad}/// {text}"),
            Self::Blank => String::new(),

            // Literals
            Self::Str(s) => {
                let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
                format!("{pad}\"{escaped}\"")
            }
            Self::Int(n) => format!("{pad}{n}"),
            Self::Bool(b) => format!("{pad}{b}"),
            Self::Raw(s) => format!("{pad}{s}"),

            // Identifiers
            Self::Ident(name) => format!("{pad}{name}"),
            Self::Path(segments) => format!("{pad}{}", segments.join("::")),

            // Declarations
            Self::Use { path, alias, public } => {
                let vis = if *public { "pub " } else { "" };
                let p = path.join("::");
                match alias {
                    Some(a) => format!("{pad}{vis}use {p} as {a};"),
                    None => format!("{pad}{vis}use {p};"),
                }
            }
            Self::Mod { name, public } => {
                let vis = if *public { "pub " } else { "" };
                format!("{pad}{vis}mod {name};")
            }
            Self::Struct {
                name,
                public,
                derives,
                fields,
            } => {
                let mut out = String::new();
                if !derives.is_empty() {
                    out.push_str(&format!(
                        "{pad}#[derive({})]\n",
                        derives.join(", ")
                    ));
                }
                let vis = if *public { "pub " } else { "" };
                if fields.is_empty() {
                    out.push_str(&format!("{pad}{vis}struct {name};"));
                    return out;
                }
                out.push_str(&format!("{pad}{vis}struct {name} {{\n"));
                for f in fields {
                    if let Some(ref doc) = f.doc {
                        out.push_str(&format!("{pad}    /// {doc}\n"));
                    }
                    let fvis = if f.public { "pub " } else { "" };
                    out.push_str(&format!(
                        "{pad}    {fvis}{}: {},\n",
                        f.name, f.field_type
                    ));
                }
                out.push_str(&format!("{pad}}}"));
                out
            }
            Self::Enum {
                name,
                public,
                derives,
                variants,
            } => {
                let mut out = String::new();
                if !derives.is_empty() {
                    out.push_str(&format!(
                        "{pad}#[derive({})]\n",
                        derives.join(", ")
                    ));
                }
                let vis = if *public { "pub " } else { "" };
                out.push_str(&format!("{pad}{vis}enum {name} {{\n"));
                for v in variants {
                    match v {
                        EnumVariant::Unit(vname) => {
                            out.push_str(&format!("{pad}    {vname},\n"));
                        }
                        EnumVariant::Tuple(vname, types) => {
                            out.push_str(&format!(
                                "{pad}    {vname}({}),\n",
                                types.join(", ")
                            ));
                        }
                        EnumVariant::Struct(vname, fields) => {
                            out.push_str(&format!("{pad}    {vname} {{\n"));
                            for f in fields {
                                // Enum variant fields are always public — no `pub` keyword
                                out.push_str(&format!(
                                    "{pad}        {}: {},\n",
                                    f.name, f.field_type
                                ));
                            }
                            out.push_str(&format!("{pad}    }},\n"));
                        }
                    }
                }
                out.push_str(&format!("{pad}}}"));
                out
            }
            Self::Impl {
                target,
                trait_name,
                body,
            } => {
                let header = match trait_name {
                    Some(t) => format!("{pad}impl {t} for {target} {{"),
                    None => format!("{pad}impl {target} {{"),
                };
                let mut out = format!("{header}\n");
                for (i, node) in body.iter().enumerate() {
                    out.push_str(&node.emit(indent + 1));
                    out.push('\n');
                    if i < body.len() - 1 {
                        out.push('\n');
                    }
                }
                out.push_str(&format!("{pad}}}"));
                out
            }
            Self::Fn {
                name,
                public,
                must_use,
                args,
                return_type,
                body,
            } => {
                let mut out = String::new();
                if *must_use {
                    out.push_str(&format!("{pad}#[must_use]\n"));
                }
                let vis = if *public { "pub " } else { "" };
                let args_str = args
                    .iter()
                    .map(|a| {
                        if a.arg_type.is_empty() {
                            a.name.clone()
                        } else {
                            format!("{}: {}", a.name, a.arg_type)
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                let ret = match return_type {
                    Some(r) => format!(" -> {r}"),
                    None => String::new(),
                };
                out.push_str(&format!("{pad}{vis}fn {name}({args_str}){ret} {{\n"));
                for node in body {
                    out.push_str(&node.emit(indent + 1));
                    out.push('\n');
                }
                out.push_str(&format!("{pad}}}"));
                out
            }

            // Expressions
            Self::StructInit { name, fields } => {
                if fields.is_empty() {
                    format!("{pad}{name} {{ }}")
                } else {
                    let fields_str = fields
                        .iter()
                        .map(|(k, v)| {
                            let val = v.emit(0);
                            if val == *k {
                                k.clone() // shorthand: `name` instead of `name: name`
                            } else {
                                format!("{k}: {val}")
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("{pad}{name} {{ {fields_str} }}")
                }
            }
            Self::Closure { args, body } => {
                let args_str = args.join(", ");
                let b = body.emit(0);
                format!("{pad}|{args_str}| {b}")
            }
            Self::Let {
                name,
                mutable,
                type_ann,
                value,
            } => {
                let m = if *mutable { "mut " } else { "" };
                let t = match type_ann {
                    Some(ty) => format!(": {ty}"),
                    None => String::new(),
                };
                let v = value.emit(0);
                format!("{pad}let {m}{name}{t} = {v};")
            }
            Self::Match { expr, arms } => {
                let e = expr.emit(0);
                let mut out = format!("{pad}match {e} {{\n");
                for arm in arms {
                    let body = arm.body.emit(0);
                    if body.contains('\n') {
                        out.push_str(&format!("{pad}    {} => {{\n", arm.pattern));
                        for line in body.lines() {
                            out.push_str(&format!("{pad}        {}\n", line.trim()));
                        }
                        out.push_str(&format!("{pad}    }}\n"));
                    } else {
                        out.push_str(&format!("{pad}    {} => {},\n", arm.pattern, body.trim()));
                    }
                }
                out.push_str(&format!("{pad}}}"));
                out
            }
            Self::MethodCall {
                receiver,
                method,
                args,
            } => {
                let r = receiver.emit(0);
                let a = args
                    .iter()
                    .map(|arg| arg.emit(0))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{pad}{r}.{method}({a})")
            }
            Self::FnCall { name, args } => {
                let a = args
                    .iter()
                    .map(|arg| arg.emit(0))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{pad}{name}({a})")
            }
            Self::MacroCall { name, args } => {
                let a = args
                    .iter()
                    .map(|arg| arg.emit(0))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{pad}{name}!({a})")
            }
            Self::Block(stmts) => {
                let mut out = format!("{pad}{{\n");
                for s in stmts {
                    out.push_str(&s.emit(indent + 1));
                    out.push('\n');
                }
                out.push_str(&format!("{pad}}}"));
                out
            }
            Self::Return(expr) => {
                let e = expr.emit(0);
                format!("{pad}return {e};")
            }

            // Control flow
            Self::If {
                cond,
                then_body,
                else_body,
            } => {
                let c = cond.emit(0);
                let mut out = format!("{pad}if {c} {{\n");
                for s in then_body {
                    out.push_str(&s.emit(indent + 1));
                    out.push('\n');
                }
                match else_body {
                    Some(eb) => {
                        out.push_str(&format!("{pad}}} else {{\n"));
                        for s in eb {
                            out.push_str(&s.emit(indent + 1));
                            out.push('\n');
                        }
                        out.push_str(&format!("{pad}}}"));
                    }
                    None => {
                        out.push_str(&format!("{pad}}}"));
                    }
                }
                out
            }
            Self::For {
                binding,
                iter,
                body,
            } => {
                let it = iter.emit(0);
                let mut out = format!("{pad}for {binding} in {it} {{\n");
                for s in body {
                    out.push_str(&s.emit(indent + 1));
                    out.push('\n');
                }
                out.push_str(&format!("{pad}}}"));
                out
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comment_emits() {
        assert_eq!(RustNode::Comment("test".into()).emit(0), "// test");
    }

    #[test]
    fn doc_comment_emits() {
        assert_eq!(RustNode::DocComment("doc".into()).emit(0), "/// doc");
    }

    #[test]
    fn blank_emits_empty() {
        assert_eq!(RustNode::Blank.emit(0), "");
    }

    #[test]
    fn str_escapes() {
        assert_eq!(RustNode::str("say \"hi\"").emit(0), r#""say \"hi\"""#);
    }

    #[test]
    fn int_emits() {
        assert_eq!(RustNode::Int(42).emit(0), "42");
    }

    #[test]
    fn bool_emits() {
        assert_eq!(RustNode::Bool(true).emit(0), "true");
    }

    #[test]
    fn ident_emits() {
        assert_eq!(RustNode::ident("String").emit(0), "String");
    }

    #[test]
    fn path_emits() {
        assert_eq!(RustNode::path(&["crate", "node"]).emit(0), "crate::node");
    }

    #[test]
    fn use_emits() {
        let node = RustNode::Use {
            path: vec!["crate".into(), "types".into(), "RustType".into()],
            alias: None, public: false,
        };
        assert_eq!(node.emit(0), "use crate::types::RustType;");
    }

    #[test]
    fn mod_public() {
        let node = RustNode::Mod { name: "node".into(), public: true };
        assert_eq!(node.emit(0), "pub mod node;");
    }

    #[test]
    fn struct_empty() {
        let node = RustNode::Struct {
            name: "Empty".into(),
            public: true,
            derives: vec!["Debug".into()],
            fields: vec![],
        };
        let out = node.emit(0);
        assert!(out.contains("#[derive(Debug)]"));
        assert!(out.contains("pub struct Empty;"));
    }

    #[test]
    fn struct_with_fields() {
        let node = RustNode::Struct {
            name: "Point".into(),
            public: true,
            derives: vec!["Debug".into(), "Clone".into()],
            fields: vec![
                StructField::new("x", "f64"),
                StructField::new("y", "f64"),
            ],
        };
        let out = node.emit(0);
        assert!(out.contains("#[derive(Debug, Clone)]"));
        assert!(out.contains("pub struct Point {"));
        assert!(out.contains("pub x: f64,"));
        assert!(out.contains("pub y: f64,"));
    }

    #[test]
    fn enum_unit_variants() {
        let node = RustNode::Enum {
            name: "Color".into(),
            public: true,
            derives: vec!["Debug".into()],
            variants: vec![
                EnumVariant::unit("Red"),
                EnumVariant::unit("Green"),
                EnumVariant::unit("Blue"),
            ],
        };
        let out = node.emit(0);
        assert!(out.contains("pub enum Color {"));
        assert!(out.contains("Red,"));
        assert!(out.contains("Green,"));
        assert!(out.contains("Blue,"));
    }

    #[test]
    fn enum_mixed_variants() {
        let node = RustNode::Enum {
            name: "Node".into(),
            public: true,
            derives: vec![],
            variants: vec![
                EnumVariant::unit("Blank"),
                EnumVariant::tuple("Comment", vec!["String"]),
                EnumVariant::with_fields("Struct", vec![
                    StructField::new("name", "String"),
                    StructField::new("fields", "Vec<Field>"),
                ]),
            ],
        };
        let out = node.emit(0);
        assert!(out.contains("Blank,"));
        assert!(out.contains("Comment(String),"));
        assert!(out.contains("Struct {"));
        assert!(out.contains("name: String,"));
    }

    #[test]
    fn impl_block() {
        let node = RustNode::Impl {
            target: "Point".into(),
            trait_name: None,
            body: vec![RustNode::Fn {
                name: "new".into(),
                public: true,
                must_use: true,
                args: vec![FnArg::new("x", "f64"), FnArg::new("y", "f64")],
                return_type: Some("Self".into()),
                body: vec![RustNode::raw("Self { x, y }")],
            }],
        };
        let out = node.emit(0);
        assert!(out.contains("impl Point {"));
        assert!(out.contains("#[must_use]"));
        assert!(out.contains("pub fn new(x: f64, y: f64) -> Self {"));
    }

    #[test]
    fn impl_trait() {
        let node = RustNode::Impl {
            target: "Point".into(),
            trait_name: Some("Display".into()),
            body: vec![],
        };
        let out = node.emit(0);
        assert!(out.contains("impl Display for Point {"));
    }

    #[test]
    fn fn_with_self() {
        let node = RustNode::Fn {
            name: "emit".into(),
            public: true,
            must_use: true,
            args: vec![FnArg::ref_self(), FnArg::new("indent", "usize")],
            return_type: Some("String".into()),
            body: vec![RustNode::raw("String::new()")],
        };
        let out = node.emit(0);
        assert!(out.contains("pub fn emit(&self, indent: usize) -> String {"));
    }

    #[test]
    fn let_binding() {
        let node = RustNode::Let {
            name: "x".into(),
            mutable: false,
            type_ann: Some("i64".into()),
            value: Box::new(RustNode::Int(42)),
        };
        assert_eq!(node.emit(0), "let x: i64 = 42;");
    }

    #[test]
    fn let_mut() {
        let node = RustNode::Let {
            name: "buf".into(),
            mutable: true,
            type_ann: None,
            value: Box::new(RustNode::fn_call("String::new", vec![])),
        };
        assert_eq!(node.emit(0), "let mut buf = String::new();");
    }

    #[test]
    fn match_expr() {
        let node = RustNode::Match {
            expr: Box::new(RustNode::ident("x")),
            arms: vec![
                MatchArm::new("1", RustNode::str("one")),
                MatchArm::new("_", RustNode::str("other")),
            ],
        };
        let out = node.emit(0);
        assert!(out.contains("match x {"));
        assert!(out.contains("1 => \"one\","));
        assert!(out.contains("_ => \"other\","));
    }

    #[test]
    fn method_call_emits() {
        let node = RustNode::method_call(RustNode::ident("s"), "push_str", vec![RustNode::str("hello")]);
        assert_eq!(node.emit(0), "s.push_str(\"hello\")");
    }

    #[test]
    fn fn_call_emits() {
        let node = RustNode::fn_call("format", vec![RustNode::str("{}")]);
        assert_eq!(node.emit(0), "format(\"{}\")");
    }

    #[test]
    fn macro_call_emits() {
        let node = RustNode::macro_call("vec", vec![RustNode::Int(1), RustNode::Int(2)]);
        assert_eq!(node.emit(0), "vec!(1, 2)");
    }

    #[test]
    fn return_emits() {
        let node = RustNode::Return(Box::new(RustNode::ident("result")));
        assert_eq!(node.emit(0), "return result;");
    }

    #[test]
    fn if_then() {
        let node = RustNode::If {
            cond: Box::new(RustNode::ident("x")),
            then_body: vec![RustNode::raw("do_thing();")],
            else_body: None,
        };
        let out = node.emit(0);
        assert!(out.contains("if x {"));
        assert!(out.contains("do_thing();"));
    }

    #[test]
    fn if_then_else() {
        let node = RustNode::If {
            cond: Box::new(RustNode::Bool(true)),
            then_body: vec![RustNode::Int(1)],
            else_body: Some(vec![RustNode::Int(2)]),
        };
        let out = node.emit(0);
        assert!(out.contains("if true {"));
        assert!(out.contains("} else {"));
    }

    #[test]
    fn for_loop() {
        let node = RustNode::For {
            binding: "item".into(),
            iter: Box::new(RustNode::ident("items")),
            body: vec![RustNode::raw("process(item);")],
        };
        let out = node.emit(0);
        assert!(out.contains("for item in items {"));
    }

    #[test]
    fn indent_4_spaces() {
        let out = RustNode::Comment("indented".into()).emit(1);
        assert_eq!(out, "    // indented");
    }

    #[test]
    fn indent_8_spaces() {
        let out = RustNode::Comment("deep".into()).emit(2);
        assert_eq!(out, "        // deep");
    }

    #[test]
    fn block_emits() {
        let node = RustNode::Block(vec![RustNode::raw("x + 1")]);
        let out = node.emit(0);
        assert!(out.contains('{'));
        assert!(out.contains('}'));
    }
}
