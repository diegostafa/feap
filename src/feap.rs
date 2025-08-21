use std::{collections::HashMap, ptr::NonNull};

pub trait Item {
    type K: Ord;

    fn new(key: Self::K) -> Self;
    fn key(&self) -> &Self::K;
}
pub trait Heap {
    type Item: Item;

    fn new() -> Self;
    fn find_min(&self) -> Option<&Self::Item>;
    fn insert(&mut self, node: Self::Item);
    fn delete_min(&mut self) -> Option<Self::Item>;
    fn meld(self, other: Self) -> Self;
    fn decrease_key(&mut self, node: Self::Item, new_key: <Self::Item as Item>::K);
    fn delete(&mut self, node: Self::Item);
}

#[derive(Debug, PartialEq, Eq)]
pub struct NodePtr<K: Ord>(NonNull<Node<K>>);
impl<K: Ord> NodePtr<K> {
    pub fn inner_ref(&self) -> &Node<K> {
        unsafe { self.0.as_ref() }
    }
    pub fn inner_mut(&mut self) -> &mut Node<K> {
        unsafe { self.0.as_mut() }
    }
    pub fn inner_ptr(&self) -> *mut Node<K> {
        self.0.as_ptr()
    }
}
impl<K: Ord> Copy for NodePtr<K> {}
impl<K: Ord> Clone for NodePtr<K> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<K: Ord> Item for NodePtr<K> {
    type K = K;

    fn new(key: Self::K) -> Self {
        let node = Node {
            key,
            rank: 0,
            is_marked: false,
            parent: None,
            first_child: None,
            prev: None,
            next: None,
        };
        unsafe { Self(NonNull::new_unchecked(Box::into_raw(Box::new(node)))) }
    }

    fn key(&self) -> &Self::K {
        &self.inner_ref().key
    }
}

#[derive(Debug, Default)]
pub struct Feap<K: Ord> {
    root: Option<NodePtr<K>>,
    len: usize,
}
impl<K: Ord> Feap<K> {
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    pub fn clear(&mut self) {
        *self = Self::new();
    }
}
impl<K: Ord> Heap for Feap<K> {
    type Item = NodePtr<K>;

    fn new() -> Self {
        Self { root: None, len: 0 }
    }

    fn find_min(&self) -> Option<&Self::Item> {
        self.root.as_ref()
    }

    fn insert(&mut self, node: Self::Item) {
        let new_root = self.root.map_or(node, |root| naive_link(root, node));
        self.root = Some(new_root);
        self.len += 1;
    }

    fn delete_min(&mut self) -> Option<Self::Item> {
        if let Some(root) = self.root {
            self.root = None;
            if root.inner_ref().first_child.is_some() {
                let mut rank_to_node = HashMap::new();
                for mut node in root.inner_ref().children() {
                    while let Some(other) = rank_to_node.remove(&node.inner_ref().rank) {
                        node = fair_link(node, other);
                    }
                    rank_to_node.insert(node.inner_ref().rank, node);
                }

                self.root = rank_to_node.into_values().reduce(naive_link);
            }
            self.len -= 1;
            return Some(root);
        }
        None
    }

    fn meld(mut self, mut other: Self) -> Self {
        match (self.root, other.root) {
            (None, _) => other,
            (_, None) => self,
            (Some(this_root), Some(other_root)) => {
                self.root = Some(naive_link(this_root, other_root));
                self.len += other.len;
                other.root = None;
                other.len = 0;
                self
            }
        }
    }

    fn decrease_key(&mut self, mut node: Self::Item, new_key: <Self::Item as Item>::K) {
        assert!(&new_key <= node.key());
        node.inner_mut().key = new_key;
        if let Some(mut root) = self.root
            && root != node
        {
            root.inner_mut().is_marked = false;
            decrease_ranks(node);
            let node = unlink(node);
            self.root = Some(naive_link(root, node));
        }
    }

    fn delete(&mut self, node: Self::Item) {
        todo!()
    }
}
impl<K: Ord> Drop for Feap<K> {
    fn drop(&mut self) {
        fn rec_drop<K: Ord>(node: NodePtr<K>) {
            unsafe {
                let children = node.inner_ref().children().collect::<Vec<_>>();
                for c in children {
                    rec_drop(c);
                }
                drop(Box::from_raw(node.inner_ptr()));
            }
        }
        if let Some(root) = self.root {
            rec_drop(root);
        }
    }
}

#[derive(Debug, Default)]
pub struct Node<K: Ord> {
    key: K,
    rank: u32,
    is_marked: bool,
    parent: Option<NodePtr<K>>,
    first_child: Option<NodePtr<K>>,
    prev: Option<NodePtr<K>>,
    next: Option<NodePtr<K>>,
}
impl<K: Ord> Node<K> {
    pub fn children(&self) -> NodeChildrenIterator<K> {
        NodeChildrenIterator {
            curr: self.first_child,
        }
    }
}

#[derive(Debug)]
pub struct NodeChildrenIterator<K: Ord> {
    curr: Option<NodePtr<K>>,
}
impl<K: Ord> Iterator for NodeChildrenIterator<K> {
    type Item = NodePtr<K>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(curr) = self.curr {
            self.curr = curr.inner_ref().next;
            return Some(curr);
        }
        None
    }
}

fn unlink<K: Ord>(this: NodePtr<K>) -> NodePtr<K> {
    if let Some(mut parent) = this.inner_ref().parent {
        if parent.inner_ref().first_child == Some(this) {
            parent.inner_mut().first_child = this.inner_ref().next;
        }
        if let Some(mut prev) = this.inner_ref().prev {
            prev.inner_mut().next = this.inner_ref().next;
        }
        if let Some(mut next) = this.inner_ref().next {
            next.inner_mut().prev = this.inner_ref().prev;
        }
    }
    this
}
fn naive_link<K: Ord>(this: NodePtr<K>, other: NodePtr<K>) -> NodePtr<K> {
    if this.key() < other.key() {
        add_child(this, other)
    } else {
        add_child(other, this)
    }
}
fn fair_link<K: Ord>(this: NodePtr<K>, other: NodePtr<K>) -> NodePtr<K> {
    assert_eq!(this.inner_ref().rank, other.inner_ref().rank);
    let mut node = naive_link(this, other);
    node.inner_mut().rank += 1;
    node
}
fn add_child<K: Ord>(mut this: NodePtr<K>, mut other: NodePtr<K>) -> NodePtr<K> {
    other.inner_mut().parent = Some(this);
    other.inner_mut().prev = None;
    other.inner_mut().next = None;
    if let Some(mut first_child) = this.inner_ref().first_child {
        other.inner_mut().next = Some(first_child);
        first_child.inner_mut().prev = Some(other);
    }
    this.inner_mut().first_child = Some(other);
    this
}
fn decrease_ranks<K: Ord>(mut node: NodePtr<K>) {
    loop {
        let Some(parent) = node.inner_ref().parent else {
            break;
        };
        node = parent;
        let n = node.inner_mut();
        n.rank = n.rank.saturating_sub(1);
        n.is_marked = !n.is_marked;
        if n.is_marked {
            break;
        }
    }
}
