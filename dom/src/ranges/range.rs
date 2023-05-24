use crate::arena::Ref;
use crate::ranges::{AbstractRange, BoundaryPoint};

// SPECLINK: https://dom.spec.whatwg.org/#interface-range
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
struct LiveRange<'arena> {
    start: BoundaryPoint<'arena>,
    end: BoundaryPoint<'arena>,
}

impl<'arena> AbstractRange<'arena> for LiveRange<'arena> {
    fn start(&self) -> &BoundaryPoint<'arena> {
        &self.start
    }

    fn end(&self) -> &BoundaryPoint<'arena> {
        &self.end
    }
}

impl<'arena> LiveRange<'arena> {
    // SPEC: FIXME{The new Range() constructor steps are to set this’s start and end to (current global object’s associated Document, 0).}
    pub fn new(document: Ref<'arena>) -> Self {
        Self {
            start: BoundaryPoint::new(document, 0),
            end: BoundaryPoint::new(document, 0),
        }
    }

    pub fn root(&self) -> Ref<'arena> {
        self.start().node.root()
    }
}
