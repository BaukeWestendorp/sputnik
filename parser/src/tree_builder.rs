use dom::{Attribute, QualifiedName};

pub trait TreeBuilder {
    type Handle: Clone;

    fn document(&mut self) -> Self::Handle;

    fn create_element(&mut self, name: QualifiedName, attributes: Vec<Attribute>) -> Self::Handle;

    fn insert_comment(&mut self, text: &str) -> Self::Handle;

    fn insert_character(&mut self, character: char);

    fn append(&mut self, parent: &Self::Handle, child: Self::Handle);

    fn append_doctype_to_document(&mut self, name: &str, public_id: &str, system_id: &str);

    fn is_same_as(&self, a: &Self::Handle, b: &Self::Handle) -> bool;

    fn parser_error(&mut self, code: Option<&str>);
}
