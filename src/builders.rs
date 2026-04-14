use crate::node::*;

// ── StructBuilder ───────────────────────────────────────────────────

pub struct StructBuilder {
    name: String,
    public: bool,
    derives: Vec<String>,
    fields: Vec<StructField>,
}

impl StructBuilder {
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            public: false,
            derives: Vec::new(),
            fields: Vec::new(),
        }
    }

    #[must_use]
    pub fn public(mut self) -> Self {
        self.public = true;
        self
    }

    #[must_use]
    pub fn derive(mut self, d: &str) -> Self {
        self.derives.push(d.to_string());
        self
    }

    #[must_use]
    pub fn field(mut self, name: &str, ty: &str) -> Self {
        self.fields.push(StructField::new(name, ty));
        self
    }

    #[must_use]
    pub fn private_field(mut self, name: &str, ty: &str) -> Self {
        self.fields.push(StructField::private(name, ty));
        self
    }

    #[must_use]
    pub fn field_with_doc(mut self, name: &str, ty: &str, doc: &str) -> Self {
        self.fields.push(StructField::new(name, ty).with_doc(doc));
        self
    }

    #[must_use]
    pub fn build(self) -> RustNode {
        RustNode::Struct {
            name: self.name,
            public: self.public,
            derives: self.derives,
            fields: self.fields,
        }
    }
}

// ── EnumBuilder ─────────────────────────────────────────────────────

pub struct EnumBuilder {
    name: String,
    public: bool,
    derives: Vec<String>,
    variants: Vec<EnumVariant>,
}

impl EnumBuilder {
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            public: false,
            derives: Vec::new(),
            variants: Vec::new(),
        }
    }

    #[must_use]
    pub fn public(mut self) -> Self {
        self.public = true;
        self
    }

    #[must_use]
    pub fn derive(mut self, d: &str) -> Self {
        self.derives.push(d.to_string());
        self
    }

    #[must_use]
    pub fn unit(mut self, name: &str) -> Self {
        self.variants.push(EnumVariant::unit(name));
        self
    }

    #[must_use]
    pub fn tuple(mut self, name: &str, types: Vec<&str>) -> Self {
        self.variants.push(EnumVariant::tuple(name, types));
        self
    }

    #[must_use]
    pub fn variant_struct(mut self, name: &str, fields: Vec<StructField>) -> Self {
        self.variants.push(EnumVariant::with_fields(name, fields));
        self
    }

    #[must_use]
    pub fn build(self) -> RustNode {
        RustNode::Enum {
            name: self.name,
            public: self.public,
            derives: self.derives,
            variants: self.variants,
        }
    }
}

// ── FnBuilder ───────────────────────────────────────────────────────

pub struct FnBuilder {
    name: String,
    public: bool,
    must_use: bool,
    args: Vec<FnArg>,
    return_type: Option<String>,
    body: Vec<RustNode>,
}

impl FnBuilder {
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            public: false,
            must_use: false,
            args: Vec::new(),
            return_type: None,
            body: Vec::new(),
        }
    }

    #[must_use]
    pub fn public(mut self) -> Self {
        self.public = true;
        self
    }

    #[must_use]
    pub fn must_use(mut self) -> Self {
        self.must_use = true;
        self
    }

    #[must_use]
    pub fn arg_ref_self(mut self) -> Self {
        self.args.push(FnArg::ref_self());
        self
    }

    #[must_use]
    pub fn arg(mut self, name: &str, ty: &str) -> Self {
        self.args.push(FnArg::new(name, ty));
        self
    }

    #[must_use]
    pub fn returns(mut self, ty: &str) -> Self {
        self.return_type = Some(ty.to_string());
        self
    }

    #[must_use]
    pub fn body(mut self, nodes: Vec<RustNode>) -> Self {
        self.body = nodes;
        self
    }

    #[must_use]
    pub fn stmt(mut self, node: RustNode) -> Self {
        self.body.push(node);
        self
    }

    #[must_use]
    pub fn build(self) -> RustNode {
        RustNode::Fn {
            name: self.name,
            public: self.public,
            must_use: self.must_use,
            args: self.args,
            return_type: self.return_type,
            body: self.body,
        }
    }
}

// ── ImplBuilder ─────────────────────────────────────────────────────

pub struct ImplBuilder {
    target: String,
    trait_name: Option<String>,
    body: Vec<RustNode>,
}

impl ImplBuilder {
    #[must_use]
    pub fn new(target: &str) -> Self {
        Self {
            target: target.to_string(),
            trait_name: None,
            body: Vec::new(),
        }
    }

    #[must_use]
    pub fn for_trait(mut self, trait_name: &str) -> Self {
        self.trait_name = Some(trait_name.to_string());
        self
    }

    #[must_use]
    pub fn method(mut self, m: RustNode) -> Self {
        self.body.push(m);
        self
    }

    #[must_use]
    pub fn build(self) -> RustNode {
        RustNode::Impl {
            target: self.target,
            trait_name: self.trait_name,
            body: self.body,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn struct_builder() {
        let node = StructBuilder::new("Point")
            .public()
            .derive("Debug")
            .derive("Clone")
            .field("x", "f64")
            .field("y", "f64")
            .build();
        let out = node.emit(0);
        assert!(out.contains("pub struct Point"));
        assert!(out.contains("pub x: f64"));
    }

    #[test]
    fn enum_builder() {
        let node = EnumBuilder::new("Color")
            .public()
            .derive("Debug")
            .unit("Red")
            .unit("Green")
            .tuple("Custom", vec!["u8", "u8", "u8"])
            .build();
        let out = node.emit(0);
        assert!(out.contains("pub enum Color"));
        assert!(out.contains("Red,"));
        assert!(out.contains("Custom(u8, u8, u8),"));
    }

    #[test]
    fn fn_builder() {
        let node = FnBuilder::new("emit")
            .public()
            .must_use()
            .arg_ref_self()
            .arg("indent", "usize")
            .returns("String")
            .stmt(RustNode::raw("String::new()"))
            .build();
        let out = node.emit(0);
        assert!(out.contains("#[must_use]"));
        assert!(out.contains("pub fn emit(&self, indent: usize) -> String"));
    }

    #[test]
    fn impl_builder() {
        let node = ImplBuilder::new("Point")
            .method(
                FnBuilder::new("origin")
                    .public()
                    .must_use()
                    .returns("Self")
                    .stmt(RustNode::raw("Self { x: 0.0, y: 0.0 }"))
                    .build(),
            )
            .build();
        let out = node.emit(0);
        assert!(out.contains("impl Point {"));
        assert!(out.contains("pub fn origin()"));
    }

    #[test]
    fn impl_trait_builder() {
        let node = ImplBuilder::new("Point")
            .for_trait("Display")
            .build();
        let out = node.emit(0);
        assert!(out.contains("impl Display for Point {"));
    }
}
