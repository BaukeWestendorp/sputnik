pub mod arena;
pub mod dom_exception;
pub mod element;
pub mod node;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone)]
pub struct QualifiedName {
    pub prefix: Option<String>,
    pub namespace: Option<Namespace>,
    pub local: String,
}

impl QualifiedName {
    // FIXME: `ns` and `prefix` should be passed as parameters.
    pub fn new(local: String) -> Self {
        QualifiedName {
            prefix: None,
            namespace: Some(Namespace::Html),
            local,
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone)]
pub struct Attribute {
    pub name: QualifiedName,
    pub value: String,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
pub enum Namespace {
    Html,
    MathMl,
    Svg,
    XLink,
    Xml,
    XmlNs,
}

impl ToString for Namespace {
    fn to_string(&self) -> String {
        match self {
            Namespace::Html => "http://www.w3.org/1999/xhtml".to_string(),
            Namespace::MathMl => "http://www.w3.org/1998/Math/MathML".to_string(),
            Namespace::Svg => "http://www.w3.org/2000/svg".to_string(),
            Namespace::XLink => "http://www.w3.org/1999/xlink".to_string(),
            Namespace::Xml => "http://www.w3.org/XML/1998/namespace".to_string(),
            Namespace::XmlNs => "http://www.w3.org/2000/xmlns/".to_string(),
        }
    }
}
