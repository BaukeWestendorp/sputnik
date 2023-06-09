use std::cell::RefCell;

use crate::html::Namespace;

use super::Attr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Element {
    // FIXME: This does not really follow the spec.
    pub tag_name: String,
    pub namespace: Option<Namespace>,
    pub attributes: RefCell<Vec<Attr>>,
}
