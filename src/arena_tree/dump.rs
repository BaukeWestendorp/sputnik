use crate::ArenaTree;

use super::node::TreeNode;

impl<T> ArenaTree<T>
where
    T: PartialEq,
{
    pub fn dump<O, C>(&self, indent: &str, opening_tag: O, closing_tag: C)
    where
        O: Fn(&TreeNode<T>) -> Option<String>,
        C: Fn(&TreeNode<T>) -> Option<String>,
    {
        self.dump_nodes(
            indent,
            &opening_tag,
            &closing_tag,
            "",
            &self.root_children(),
        );
    }

    fn dump_nodes<O, C>(
        &self,
        indent: &str,
        opening_tag: &O,
        closing_tag: &C,
        current_indentation: &str,
        nodes: &Vec<usize>,
    ) where
        O: Fn(&TreeNode<T>) -> Option<String>,
        C: Fn(&TreeNode<T>) -> Option<String>,
    {
        for child in nodes {
            let child_node = self.get(*child);

            if let Some(opening_tag) = opening_tag(child_node) {
                eprintln!("{current_indentation}{}", opening_tag);
            }

            let mut current_indentation = current_indentation.to_string();
            current_indentation.push_str(indent);

            self.dump_nodes(
                indent,
                opening_tag,
                closing_tag,
                current_indentation.as_str(),
                &child_node.children,
            );

            if let Some(closing_tag) = closing_tag(child_node) {
                eprintln!("{current_indentation}{}", closing_tag);
            }
        }
    }
}
