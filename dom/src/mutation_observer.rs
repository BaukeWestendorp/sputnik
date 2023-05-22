// SPECLINK: https://dom.spec.whatwg.org/#registered-observer
// SPEC: A registered observer consists of an observer (a MutationObserver object) and options (a MutationObserverInit dictionary).
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub struct RegisteredObserver {
    pub observer: MutationObserver,
    pub options: MutationObserverInit,
}

// SPECLINK: https://dom.spec.whatwg.org/#interface-mutationobserver
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub struct MutationObserver {}

// SPECLINK: https://dom.spec.whatwg.org/#interface-mutationobserver
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub struct MutationObserverInit {}
