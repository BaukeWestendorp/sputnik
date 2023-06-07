// Shamelessly inspired by this great article about arena trees:
// https://dev.to/deciduously/no-more-tears-no-more-knots-arena-allocated-trees-in-rust-44k6

use node::TreeNode;

mod dump;
mod node;

#[derive(Debug, Default)]
pub struct ArenaTree<T>
where
    T: PartialEq,
{
    arena: Vec<TreeNode<T>>,
}

impl<T> ArenaTree<T>
where
    T: PartialEq,
{
    pub fn new() -> Self {
        Self { arena: vec![] }
    }

    pub fn size(&self) -> usize {
        self.arena.len()
    }

    pub fn node(&mut self, value: T) -> usize {
        for node in &self.arena {
            if node.value == value {
                return node.index;
            }
        }

        let index = self.arena.len();
        self.arena.push(TreeNode::new(index, value));
        index
    }

    pub fn insert(&mut self, parent: usize, child: usize) {
        self.arena[child].parent = Some(parent);
        self.arena[parent].children.push(child);
    }

    pub fn set(&mut self, target: usize, value: T) {
        self.arena[target].value = value
    }

    pub fn get(&self, index: usize) -> &TreeNode<T> {
        &self.arena[index]
    }

    pub fn root_children(&self) -> Vec<usize> {
        self.arena
            .iter()
            .filter(|node| node.parent.is_none())
            .map(|node| node.index)
            .collect()
    }

    pub fn edges(&self) -> usize {
        self.arena
            .iter()
            .fold(0, |acc, node| acc + node.children.len())
    }

    pub fn depth(&self, index: usize) -> usize {
        match self.arena[index].parent {
            Some(id) => 1 + self.depth(id),
            None => 0,
        }
    }

    pub fn depth_to_target(&self, index: usize, target: &T) -> Option<usize> {
        if target == &self.arena[index].value {
            return Some(0);
        }

        for child in &self.arena[index].children {
            if let Some(depth) = self.depth_to_target(*child, target) {
                return Some(depth + 1);
            }
        }

        None
    }

    pub fn distance_between(&mut self, from: T, target: T) -> usize {
        let start_node = self.node(from);
        let mut distance = 0;
        let mut traversal = &self.arena[start_node];

        while let Some(inner) = traversal.parent {
            if let Some(depth) = self.depth_to_target(inner, &target) {
                distance += depth;
                break;
            }
            traversal = &self.arena[inner];
            distance += 1;
        }

        distance - 1
    }
}
