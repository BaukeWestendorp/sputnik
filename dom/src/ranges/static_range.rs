// FIXME: Remove when we actually use ranges
#![allow(dead_code)]

use crate::arena::Ref;
use crate::dom_exception::DomException;
use crate::node::Node;
use crate::ranges::{AbstractRange, BoundaryPoint, BoundaryPointPosition};

// SPECLINK: https://dom.spec.whatwg.org/#interface-staticrange
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct StaticRange<'arena> {
    start: BoundaryPoint<'arena>,
    end: BoundaryPoint<'arena>,
}

impl<'arena> AbstractRange<'arena> for StaticRange<'arena> {
    fn start(&self) -> &BoundaryPoint<'arena> {
        &self.start
    }

    fn end(&self) -> &BoundaryPoint<'arena> {
        &self.end
    }
}

impl<'arena> StaticRange<'arena> {
    pub fn new(
        start_container: Ref<'arena>,
        start_offset: usize,
        end_container: Ref<'arena>,
        end_offset: usize,
    ) -> Result<Self, DomException> {
        // SPEC: 1. If init["startContainer"] or init["endContainer"] is a DocumentType or Attr node,
        if start_container.is_doctype()
            || start_container.is_attr()
            || end_container.is_doctype()
            || end_container.is_attr()
        {
            // SPEC: then throw an "InvalidNodeTypeError" DOMException.
            return Err(DomException::InvalidNodeTypeError);
        }

        // SPEC: 2. Set this’s start to (init["startContainer"], init["startOffset"])
        //          and end to (init["endContainer"], init["endOffset"]).
        let range = StaticRange {
            start: BoundaryPoint::new(start_container, start_offset),
            end: BoundaryPoint::new(end_container, end_offset),
        };

        Ok(range)
    }

    pub fn is_valid(&self) -> bool {
        let position = self.start().position(self.end().clone());

        // SPEC: A StaticRange is valid if all of the following are true:
        // SPEC: * Its start and end are in the same node tree.
        Node::are_same(self.start().node.document(), self.end().node.document()) &&
            // SPEC: * Its start offset is between 0 and its start node’s length, inclusive.
            self.start().verify_correctness() &&
            // SPEC: * Its end offset is between 0 and its end node’s length, inclusive.
            self.end().verify_correctness() &&
            // SPEC: * Its start is before or equal to its end.
            (position == BoundaryPointPosition::Before || position == BoundaryPointPosition::Equal)
    }
}
