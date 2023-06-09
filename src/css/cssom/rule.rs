use super::{CssStyleDeclaration, TreeHandle};

// 6.4. CSS Rule
// https://drafts.csswg.org/cssom/#css-rules
#[derive(Debug, Clone, PartialEq)]
pub struct CssRule {
    // FIXME: Implement historical attributes and constants.
    type_: usize,
    text: String,
    parent_css_rule: Option<TreeHandle>,
    parent_css_stylesheet: Option<TreeHandle>,
    child_css_rules: Option<Vec<TreeHandle>>,
}

// FIXME: SPECLINK
pub struct CssStyleRule {
    rule: CssRule,
    selector_text: String,
    style: CssStyleDeclaration,
}
