#![cfg(test)]

use crate::feap::Feap;

#[cfg(test)]
fn make_heap(a: i32, b: i32) -> Feap<i32, ()> {
    (a..b).fold(Feap::new(), |mut feap, i| {
        feap.insert((i, ()));
        feap
    })
}

#[test]
fn feap_empty() {
    let feap = make_heap(0, 0);
    assert!(feap.get_min().is_none());
    assert!(feap.is_empty());
}

#[test]
fn feap_get_min() {
    let mut feap = make_heap(1, 3);
    assert!(*feap.get_min().unwrap().0 == 1);

    feap.insert((0, ()));
    assert!(*feap.get_min().unwrap().0 == 0);

    assert!(feap.delete_min().unwrap().0 == 0);
    assert!(*feap.get_min().unwrap().0 == 1);

    feap.insert((0, ()));
    assert!(*feap.get_min().unwrap().0 == 0);

    feap.entries(&0).for_each(|e| e.delete());
    assert!(*feap.get_min().unwrap().0 == 1);
}

#[test]
fn feap_clear() {
    let mut feap = make_heap(0, 10);
    assert!(feap.get_min().is_some());
    feap.clear();
    assert!(feap.get_min().is_none());
    assert!(feap.is_empty());
}

#[test]
fn feap_merge() {
    let feap1 = make_heap(0, 10);
    assert!(feap1.len() == 10);

    let feap2 = make_heap(10, 20);
    assert!(feap2.len() == 10);

    let feap = feap1.merge(feap2);
    assert!(feap.len() == 20);
    assert!(*feap.get_min().unwrap().0 == 0);
}

#[test]
fn feap_delete_min() {
    let mut feap = make_heap(0, 10);
    let mut len = feap.len();

    while len != 0 {
        let min1 = *feap.get_min().unwrap().0;
        let min2 = *feap.get_min().unwrap().0;
        assert!(min1 == min2);

        let min = feap.delete_min().unwrap().0;
        assert_eq!(min, min1);
        assert_eq!(min, min2);
        len = feap.len();
    }
}
