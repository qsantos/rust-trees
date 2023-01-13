use std::cmp::Ordering;

type Anchor<K> = Option<Box<BstNode<K>>>;

struct BstNode<K> {
    key: K,
    left: Anchor<K>,
    right: Anchor<K>,
}

impl<K> BstNode<K> {
    fn new(key: K) -> Self {
        BstNode {
            key,
            left: None,
            right: None,
        }
    }
}

pub struct Bst<K> {
    root: Anchor<K>,
}

impl<K> Bst<K> {
    pub fn new() -> Self {
        Bst { root: None }
    }
}

impl<K> Default for Bst<K> {
    fn default() -> Self {
        Bst::new()
    }
}

impl<K: std::fmt::Display> Bst<K> {
    pub fn print(&self) {
        fn aux<K: std::fmt::Display>(anchor: &Anchor<K>, indent: usize) {
            let prefix = "    ".repeat(indent);
            match anchor {
                None => println!("{}- ", prefix),
                Some(node) => {
                    println!("{}- {}", prefix, node.key);
                    aux(&node.left, indent + 1);
                    aux(&node.right, indent + 1);
                }
            }
        }
        aux(&self.root, 0);
    }
}

impl<K: Ord> Bst<K> {
    fn check(&self) {
        fn aux<K: Ord>(anchor: &Anchor<K>, min: Option<&K>, max: Option<&K>) {
            match anchor {
                None => (),
                Some(node) => {
                    if let Some(min) = min {
                        assert!(node.key > *min);
                    }
                    if let Some(max) = max {
                        assert!(node.key < *max);
                    }
                    aux(&node.left, min, Some(&node.key));
                    aux(&node.right, Some(&node.key), max);
                }
            }
        }
        aux(&self.root, None, None);
    }

    pub fn insert(&mut self, key: K) {
        fn aux<K: Ord>(anchor: &mut Anchor<K>, key: K) {
            match anchor {
                None => *anchor = Some(Box::new(BstNode::new(key))),
                Some(node) => match key.cmp(&node.key) {
                    Ordering::Less => aux(&mut node.left, key),
                    Ordering::Greater => aux(&mut node.right, key),
                    Ordering::Equal => (),
                },
            }
        }
        aux(&mut self.root, key);
        self.check();
    }

    pub fn contains(&self, key: K) -> bool {
        fn aux<K: Ord>(anchor: &Anchor<K>, key: &K) -> bool {
            match anchor {
                None => false,
                Some(node) => match key.cmp(&node.key) {
                    Ordering::Less => aux(&node.left, key),
                    Ordering::Greater => aux(&node.right, key),
                    Ordering::Equal => true,
                },
            }
        }
        aux(&self.root, &key)
    }

    pub fn remove(&mut self, key: K) {
        fn leftmost<K>(mut node: &mut Box<BstNode<K>>) -> Box<BstNode<K>> {
            while node.left.is_some() && node.left.as_ref().unwrap().left.is_some() {
                node = node.left.as_mut().unwrap();
            }
            let mut ret = node.left.take().unwrap();
            node.left = ret.right.take();
            ret
        }

        fn aux<K: Ord>(anchor: &mut Anchor<K>, key: K) {
            match anchor {
                None => (),
                Some(node) => match key.cmp(&node.key) {
                    Ordering::Less => aux(&mut node.left, key),
                    Ordering::Greater => aux(&mut node.right, key),
                    Ordering::Equal => match (node.left.take(), node.right.take()) {
                        (None, None) => *anchor = None,
                        (Some(left), None) => *anchor = Some(left),
                        (None, Some(right)) => *anchor = Some(right),
                        (Some(left), Some(mut right)) => {
                            if right.left.is_none() {
                                right.left = Some(left);
                                *anchor = Some(right);
                            } else {
                                let mut node = leftmost(&mut right);
                                node.left = Some(left);
                                node.right = Some(right);
                                *anchor = Some(node);
                            }
                        }
                    },
                },
            }
        }
        aux(&mut self.root, key);
        self.check();
    }
}

#[test]
fn test() {
    let mut t: Bst<i32> = Bst::new();

    t.insert(8);
    t.insert(4);
    t.insert(2);
    t.insert(1);
    t.insert(3);
    t.insert(6);
    t.insert(5);
    t.insert(7);

    t.insert(12);
    t.insert(10);
    t.insert(9);
    t.insert(11);
    t.insert(14);
    t.insert(13);
    t.insert(15);

    t.print();
    t.remove(8);
    t.print();

    for v in 1..15 {
        if v != 8 {
            assert!(t.contains(v), "should contain {}", v);
        }
    }
    assert!(!t.contains(8));
}
