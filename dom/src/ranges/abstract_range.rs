// FIXME: Remove when we actually use ranges
#![allow(dead_code)]

use crate::arena::Ref;
use crate::node::Node;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum BoundaryPointPosition {
    Before,
    Equal,
    After,
}

// SPECLINK: https://dom.spec.whatwg.org/#boundary-points
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct BoundaryPoint<'a> {
    // SPEC: A boundary point is a tuple consisting of a node (a node)
    pub node: Ref<'a>,
    // SPEC: and an offset (a non-negative integer).
    pub offset: usize,
}

impl<'a> BoundaryPoint<'a> {
    pub fn new(node: Ref<'a>, offset: usize) -> Self {
        Self { node, offset }
    }

    pub fn verify_correctness(&self) -> bool {
        // SPEC: A correct boundary point’s offset will be between 0 and the boundary point’s node’s length, inclusive.
        let range = 0..=self.node.length();
        range.contains(&self.offset)
    }

    pub fn position(&self, relative_to: BoundaryPoint<'a>) -> BoundaryPointPosition {
        // SPEC: 1. Assert: nodeA and nodeB have the same root.
        assert!(Node::have_same_root(self.node, relative_to.node));

        // SPEC: 2. If nodeA is nodeB,
        if Node::are_same(self.node, relative_to.node) {
            // SPEC: then return equal if offsetA is offsetB,
            if self.offset == relative_to.offset {
                return BoundaryPointPosition::Equal;
            }
            // SPEC: before if offsetA is less than offsetB,
            if self.offset < relative_to.offset {
                return BoundaryPointPosition::Before;
            }
            // SPEC: and after if offsetA is greater than offsetB.
            if self.offset > relative_to.offset {
                return BoundaryPointPosition::After;
            }
        }
        // SPEC: 3. If nodeA is following nodeB,
        if self.node.is_following(relative_to.node) {
            return match relative_to.position(self.clone()) {
                // SPEC: then if the position of (nodeB, offsetB) relative to (nodeA, offsetA) is before, return after,
                BoundaryPointPosition::Before => BoundaryPointPosition::After,
                BoundaryPointPosition::Equal => panic!("A and B can't be the same"),
                // SPEC: and if it is after, return before.
                BoundaryPointPosition::After => BoundaryPointPosition::Before,
            };
        }

        // SPEC: 4. If nodeA is an ancestor of nodeB:
        if self.node.is_ancestor_of(relative_to.node) {
            // SPEC: 4.1. Let child be nodeB.
            let mut child = relative_to.node;
            // SPEC: 4.2. While child is not a child of nodeA,
            while child.is_child_of(self.node) {
                // SPEC: set child to its parent.
                child = child.parent().unwrap();
                // SPEC: 4.3. If child’s index is less than offsetA, then return after.
                if child.index() < self.offset {
                    return BoundaryPointPosition::After;
                }
            }
        }

        // SPEC: 5. Return before.
        BoundaryPointPosition::Before
    }
}

pub trait AbstractRange<'a> {
    fn start(&self) -> &BoundaryPoint<'a>;

    fn end(&self) -> &BoundaryPoint<'a>;

    fn collapsed(&self) -> bool {
        Node::are_same(self.start().node, self.end().node)
            && self.start().offset == self.end().offset
    }
}
