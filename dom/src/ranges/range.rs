// FIXME: Remove when we actually use ranges
#![allow(dead_code)]

use crate::arena::NodeRef;
use crate::ranges::{AbstractRange, BoundaryPoint};

// SPECLINK: https://dom.spec.whatwg.org/#interface-range
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
struct Range<'a> {
    start: BoundaryPoint<'a>,
    end: BoundaryPoint<'a>,
}

impl<'a> AbstractRange<'a> for Range<'a> {
    fn start(&self) -> &BoundaryPoint<'a> {
        &self.start
    }

    fn end(&self) -> &BoundaryPoint<'a> {
        &self.end
    }
}

impl<'a> Range<'a> {
    // SPEC: FIXME{The new Range() constructor steps are to set this’s start and end to (current global object’s associated Document, 0).}
    pub fn new(document: NodeRef<'a>) -> Self {
        Self {
            start: BoundaryPoint::new(document, 0),
            end: BoundaryPoint::new(document, 0),
        }
    }

    pub fn root(&self) -> NodeRef<'a> {
        self.start().node.root()
    }
}
