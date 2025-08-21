#![cfg(test)]

use crate::feap::Feap;
use crate::feap::Heap;
use crate::feap::Item;
use crate::feap::NodePtr;

#[cfg(test)]
fn make_heap(start: i32, end: i32) -> Feap<i32> {
    (start..end).fold(Feap::new(), |mut feap, i| {
        feap.insert(NodePtr::new(i));
        feap
    })
}

#[test]
fn feap_empty() {
    let feap = make_heap(0, 0);
    assert!(feap.find_min().is_none());
    assert!(feap.is_empty());
}

#[test]
fn feap_find_min() {
    let mut feap = make_heap(1, 3);
    assert!(*feap.find_min().unwrap().key() == 1);

    feap.insert(NodePtr::new(0));
    assert!(*feap.find_min().unwrap().key() == 0);
    assert!(*feap.delete_min().unwrap().key() == 0);
    assert!(*feap.find_min().unwrap().key() == 1);

    feap.insert(NodePtr::new(0));
    assert!(*feap.find_min().unwrap().key() == 0);
}

#[test]
fn feap_clear() {
    let mut feap = make_heap(0, 10);
    assert!(feap.find_min().is_some());
    feap.clear();
    assert!(feap.find_min().is_none());
    assert!(feap.is_empty());
}

#[test]
fn feap_merge() {
    let feap1 = make_heap(0, 10);
    assert!(feap1.len() == 10);

    let feap2 = make_heap(10, 20);
    assert!(feap2.len() == 10);

    let feap = feap1.meld(feap2);
    assert!(feap.len() == 20);
    assert!(*feap.find_min().unwrap().key() == 0);
}

#[test]
fn feap_delete_min() {
    let mut feap = make_heap(0, 10);
    let mut len = feap.len();

    while len != 0 {
        let min1 = *feap.find_min().unwrap().key();
        let min2 = *feap.find_min().unwrap().key();
        assert!(min1 == min2);

        let min = *feap.delete_min().unwrap().key();
        assert_eq!(min, min1);
        assert_eq!(min, min2);
        len = feap.len();
    }
}
