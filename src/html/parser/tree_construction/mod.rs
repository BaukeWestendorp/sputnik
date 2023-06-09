use super::Parser;

mod creating_inserting;
pub(crate) mod insertion_mode;
pub(crate) mod list_of_active_formatting_elements;
pub(crate) mod stack_of_open_elements;

impl<'a> Parser<'a> {
    // https://html.spec.whatwg.org/multipage/parsing.html#generate-implied-end-tags
    pub(crate) fn generate_implied_end_tags_except_for(&'a self, except_for: Option<&str>) {
        // while the current node is a dd element, a dt element, an li element, an optgroup element, an option element, a p element, an rb element, an rp element, an rt element, or an rtc element, the UA must pop the current node off the stack of open elements.
        loop {
            let node = self.current_node();

            if let Some(except_for) = except_for {
                if node.is_element_with_tag(except_for) {
                    return;
                }
            }

            if !node.is_element_with_one_of_tags(&[
                "dd", "dt", "li", "optgroup", "option", "p", "rb", "rp", "rt", "rtc",
            ]) {
                return;
            }

            self.open_elements.pop();
        }
    }
}
