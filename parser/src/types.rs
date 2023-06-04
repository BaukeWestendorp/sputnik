use std::cell::Cell;

use crate::dom::node::Node;

pub type NodeLink<'a> = Cell<Option<NodeRef<'a>>>;
pub type NodeRef<'a> = &'a Node<'a>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertionMode {
    Initial,
    BeforeHtml,
    BeforeHead,
    InHead,
    InHeadNoscript,
    AfterHead,
    InBody,
    Text,
    InTable,
    InTableText,
    InCaption,
    InColumnGroup,
    InTableBody,
    InRow,
    InCell,
    InSelect,
    InSelectInTable,
    InTemplate,
    AfterBody,
    InFrameset,
    AfterFrameset,
    AfterAfterBody,
    AfterAfterFrameset,
}
