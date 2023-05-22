pub mod arena;
pub mod element;
pub mod mutation_observer;
pub mod mutation_record;
pub mod node;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone)]
pub struct QualifiedName {
    pub prefix: Option<String>,
    pub ns: Option<String>,
    pub local: String,
}

impl QualifiedName {
    // FIXME: `ns` should be passed as a parameter.
    pub fn new(prefix: Option<String>, local: String) -> Self {
        QualifiedName {
            prefix,
            ns: None,
            local,
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone)]
pub struct Attribute {
    pub name: QualifiedName,
    pub value: String,
}

impl From<tokenizer::Attribute> for Attribute {
    fn from(value: tokenizer::Attribute) -> Self {
        Attribute {
            name: QualifiedName::new(None, value.name),
            value: value.value,
        }
    }
}
