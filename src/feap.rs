use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    ptr::NonNull,
};

/// A pointer to a node
type NodePtr<K, V> = NonNull<Node<K, V>>;

/// A simple fibonacci heap implementing a priority queue
#[derive(Debug)]
pub struct Feap<K: PartialOrd + Eq + Hash, V> {
    /// A pointer to the root of the heap
    root: Option<NodePtr<K, V>>,

    /// The number of nodes in the heap
    len: usize,

    /// A map from keys to nodes with that key
    nodes: HashMap<K, HashSet<NodePtr<K, V>>>,
}
impl<K: PartialOrd + Eq + Hash + Copy, V> Feap<K, V> {
    /// Create an empty heap
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear the heap, deallocating all the previous nodes
    pub fn clear(&mut self) {
        std::mem::swap(self, &mut Self::new());
    }

    /// Check if the heap is empty
    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    /// Return the number of nodes in the heap
    pub fn len(&self) -> usize {
        self.len
    }

    /// Return a reference to the minimum element, if it exists
    pub fn get_min(&self) -> Option<(&K, &V)> {
        self.root
            .map(|root| unsafe { (&root.as_ref().key, &root.as_ref().val) })
    }

    /// Return an entry to a the minimum element, if it exists
    pub fn min(&mut self) -> Option<Entry<K, V>> {
        self.root.map(|root| Entry {
            node: root,
            feap: NonNull::new(self as *mut _).unwrap(),
        })
    }

    /// Return an iterator over the entries with the given key
    pub fn entries<'a>(&'a mut self, key: &K) -> impl Iterator<Item = Entry<K, V>> + 'a {
        let feap = NonNull::new(self as *mut _).unwrap();
        self.nodes
            .get(key)
            .map(|nodes| nodes.iter().map(move |&node| Entry { node, feap }))
            .into_iter()
            .flatten()
    }

    /// Merge two heaps into a new one
    pub fn merge(mut self, mut other: Self) -> Self {
        match (self.root, other.root) {
            (None, _) => other,
            (_, None) => self,
            (Some(this_root), Some(other_root)) => {
                for (k, v) in std::mem::take(&mut other.nodes) {
                    self.nodes.entry(k).or_default().extend(v);
                }
                Self {
                    root: Some(naive_link(this_root, other_root)),
                    len: self.len + other.len,
                    nodes: std::mem::take(&mut self.nodes),
                }
            }
        }
    }

    /// Insert a new element in the heap
    pub fn insert(&mut self, (key, val): (K, V)) {
        let n = NonNull::from(Box::leak(Box::new(Node::new(key, val))));
        self.root = self.root.map_or(Some(n), |root| Some(naive_link(root, n)));
        self.nodes.entry(key).or_default().insert(n);
        self.len += 1;
    }

    /// Delete and return the minimum element, if it exists
    pub fn delete_min(&mut self) -> Option<(K, V)> {
        unsafe {
            if let Some(root) = self.root {
                self.root = None;

                if root.as_ref().first_child.is_some() {
                    let mut rank_to_node = HashMap::new();

                    // fair link the children
                    for mut node in root.as_ref().children() {
                        while let Some(other) = rank_to_node.remove(&node.as_ref().rank) {
                            node = fair_link(node, other);
                        }
                        rank_to_node.insert(node.as_ref().rank, node);
                    }

                    // naive link what's left
                    self.root = rank_to_node.into_values().reduce(naive_link);
                }

                let min = Box::from_raw(root.as_ptr());
                self.nodes.get_mut(&min.key).unwrap().remove(&root);
                self.len -= 1;
                return Some((min.key, min.val));
            }
            None
        }
    }
}
impl<K: PartialOrd + Eq + Hash, V> Default for Feap<K, V> {
    fn default() -> Self {
        Self {
            root: Default::default(),
            len: Default::default(),
            nodes: Default::default(),
        }
    }
}
impl<K: PartialOrd + Eq + Hash, V> Drop for Feap<K, V> {
    fn drop(&mut self) {
        for node in self.nodes.values().flatten() {
            unsafe { drop(Box::from_raw(node.as_ptr())) };
        }
    }
}

/// An exclusive entry to a node in the heap for specific node operations
#[derive(Debug)]
pub struct Entry<K: PartialOrd + Eq + Hash + Copy, V> {
    /// A pointer to the heap
    feap: NonNull<Feap<K, V>>,

    /// A pointer to the node
    node: NodePtr<K, V>,
}
impl<K: PartialOrd + Eq + Hash + Copy, V> Entry<K, V> {
    /// The key of the entry
    pub fn key(&self) -> &K {
        unsafe { &self.node.as_ref().key }
    }

    /// The value of the entry
    pub fn value(&self) -> &V {
        unsafe { &self.node.as_ref().val }
    }

    /// Remove this entry from the heap
    pub fn delete(mut self) {
        let feap = unsafe { self.feap.as_mut() };
        if let Some(root) = feap.root {
            let min_key = unsafe { root.as_ref().key };
            self.decrease_key(min_key);
            let _ = feap.delete_min();
        }
    }

    /// Update the key of the entry
    pub fn decrease_key(&mut self, new_key: K) {
        let feap = unsafe { self.feap.as_mut() };

        if let Some(mut root) = feap.root {
            unsafe {
                let key = self.node.as_ref().key;
                assert!(new_key <= key);
                feap.nodes.entry(key).or_default().remove(&self.node);
                feap.nodes.entry(new_key).or_default().insert(self.node);
                self.node.as_mut().key = new_key;

                if root != self.node {
                    root.as_mut().is_marked = true;
                    self.node.as_mut().cascade_decrease_rank();
                    feap.root = Some(naive_link(root, unlink(self.node)));
                }
            }
        }
    }
}

// A node in the heap
#[derive(Debug)]
struct Node<K: PartialOrd, V> {
    key: K,
    val: V,
    rank: u32,
    is_marked: bool,
    parent: Option<NonNull<Self>>,
    first_child: Option<NonNull<Self>>,
    prev: Option<NonNull<Self>>,
    next: Option<NonNull<Self>>,
}
impl<K: PartialOrd, V> Node<K, V> {
    fn new(key: K, val: V) -> Self {
        Node {
            key,
            val,
            rank: 0,
            is_marked: false,
            parent: None,
            first_child: None,
            prev: None,
            next: None,
        }
    }
    fn cascade_decrease_rank(&mut self) {
        if !self.is_marked {
            self.is_marked = !self.is_marked;
            self.rank = self.rank.saturating_sub(1);
            if let Some(mut parent) = self.parent {
                unsafe { parent.as_mut().cascade_decrease_rank() };
            }
        }
    }
    fn children(&self) -> NodeChildrenIterator<K, V> {
        NodeChildrenIterator {
            curr: self.first_child,
        }
    }
}

// An iterator over the children of a node
struct NodeChildrenIterator<K: PartialOrd, V> {
    curr: Option<NodePtr<K, V>>,
}
impl<K: PartialOrd, V> Iterator for NodeChildrenIterator<K, V> {
    type Item = NodePtr<K, V>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(curr) = self.curr {
            self.curr = unsafe { curr.as_ref().next };
            return Some(curr);
        }
        None
    }
}

/// Detaches a node from the heap
fn unlink<K: PartialOrd, V>(this: NodePtr<K, V>) -> NodePtr<K, V> {
    unsafe {
        if let Some(mut parent) = this.as_ref().parent {
            if parent.as_ref().first_child == Some(this) {
                parent.as_mut().first_child = this.as_ref().next;
            }
            if let Some(mut prev) = this.as_ref().prev {
                prev.as_mut().next = this.as_ref().next;
            }
            if let Some(mut next) = this.as_ref().next {
                next.as_mut().prev = this.as_ref().prev;
            }
        }
        this
    }
}

// Links two nodes, while mantaining the heap invariant
fn naive_link<K: PartialOrd, V>(this: NodePtr<K, V>, other: NodePtr<K, V>) -> NodePtr<K, V> {
    let this_node = unsafe { this.as_ref() };
    let other_node = unsafe { other.as_ref() };

    if this_node.key < other_node.key {
        add_child(this, other)
    } else {
        add_child(other, this)
    }
}

// Naive links two nodes and updates the rank of the parent
fn fair_link<K: PartialOrd, V>(this: NodePtr<K, V>, other: NodePtr<K, V>) -> NodePtr<K, V> {
    unsafe {
        assert_eq!(this.as_ref().rank, other.as_ref().rank);
        let mut node = naive_link(this, other);
        node.as_mut().rank += 1;
        node
    }
}

// Inserts a node as the first child of another node, updating the pointers accordingly
fn add_child<K: PartialOrd, V>(mut this: NodePtr<K, V>, mut other: NodePtr<K, V>) -> NodePtr<K, V> {
    unsafe {
        other.as_mut().parent = Some(this);
        other.as_mut().prev = None;
        other.as_mut().next = None;
        if let Some(mut first_child) = this.as_ref().first_child {
            other.as_mut().next = Some(first_child);
            first_child.as_mut().prev = Some(other);
        }
        this.as_mut().first_child = Some(other);
        this
    }
}
