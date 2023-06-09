use crate::dom::NodeRef;

use super::TreeHandle;

/// 6.5 Declarations
/// https://drafts.csswg.org/cssom/#css-declarations
pub struct CssDeclaration {
    pub property_name: String,
    pub value: Vec<String>,
    pub important: bool,
    pub case_sensitive: bool,
}

/// 6.6
/// https://drafts.csswg.org/cssom/#css-declaration-blocks
pub struct CssDeclarationBlock<'a> {
    computed: bool,
    declarations: Vec<CssDeclaration>,
    parent_css_rule: Option<TreeHandle>,
    owner_node: Option<NodeRef<'a>>,
    updating: bool,
}

/// 6.6.1
/// https://drafts.csswg.org/cssom/#the-cssstyledeclaration-interface
// FIXME: Implement
pub struct CssStyleDeclaration {}
