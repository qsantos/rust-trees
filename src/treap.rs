use std::cmp::Ordering;

type Anchor<K> = Option<Box<TreapNode<K>>>;

struct TreapNode<K> {
    key: K,
    priority: u64,
    children: [Anchor<K>; 2],
}

impl<K> TreapNode<K> {
    fn new(key: K) -> Self {
        TreapNode {
            key,
            priority: rand::random(),
            children: [None, None],
        }
    }
}

pub struct Treap<K> {
    root: Anchor<K>,
}

impl<K> Treap<K> {
    pub fn new() -> Self {
        Treap { root: None }
    }
}

impl<K: std::fmt::Display> Treap<K> {
    pub fn print(&self) {
        fn aux<K: std::fmt::Display>(anchor: &Anchor<K>, depth: usize) {
            let prefix = "    ".repeat(depth);
            match anchor {
                None => println!("{}-", prefix),
                Some(node) => {
                    println!("{}- {}", prefix, node.key);
                    aux(&node.children[0], depth + 1);
                    aux(&node.children[1], depth + 1);
                }
            }
        }
        aux(&self.root, 0)
    }
}

impl<K: Ord> Treap<K> {
    fn check(&self) {
        fn aux<K: Ord>(
            anchor: &Anchor<K>,
            min_key: Option<&K>,
            max_key: Option<&K>,
            parent_priority: Option<u64>,
        ) {
            match anchor {
                None => (),
                Some(node) => {
                    // check this is a binary search tree
                    if let Some(min_key) = min_key {
                        assert!(node.key > *min_key);
                    }
                    if let Some(max_key) = max_key {
                        assert!(node.key < *max_key);
                    }
                    // check this is a heap
                    /* TODO
                    if let Some(parent_priority) = parent_priority {
                        assert!(node.priority < parent_priority);
                    }
                    // check this is a complete tree
                    if node.children[0].is_none() {
                        assert!(node.children[1].is_none());
                    }
                    */
                    // recurse
                    let prio = Some(node.priority);
                    aux(&node.children[0], min_key, Some(&node.key), prio);
                    aux(&node.children[1], Some(&node.key), max_key, prio);
                }
            }
        }
        aux(&self.root, None, None, None);
    }

    pub fn insert(&mut self, key: K) {
        fn aux<K: Ord>(anchor: &mut Anchor<K>, key: K) {
            match anchor {
                None => *anchor = Some(Box::new(TreapNode::new(key))),
                Some(node) => match key.cmp(&node.key) {
                    Ordering::Less => aux(&mut node.children[0], key),
                    Ordering::Greater => aux(&mut node.children[1], key),
                    Ordering::Equal => (),
                },
            }
        }
        aux(&mut self.root, key);
        self.check();
    }

    pub fn contains(&self, key: K) -> bool {
        fn aux<K: Ord>(anchor: &Anchor<K>, key: K) -> bool {
            match anchor {
                None => false,
                Some(node) => match key.cmp(&node.key) {
                    Ordering::Less => aux(&node.children[0], key),
                    Ordering::Greater => aux(&node.children[1], key),
                    Ordering::Equal => true,
                },
            }
        }
        aux(&self.root, key)
    }

    pub fn remove(&mut self, key: K) {
        fn leftmost<K: Ord>(mut node: &mut TreapNode<K>) -> Box<TreapNode<K>> {
            while node.children[0].as_ref().unwrap().children[0].is_some() {
                node = node.children[0].as_mut().unwrap();
            }
            let mut ret = node.children[0].take().unwrap();
            node.children[0] = ret.children[1].take();
            assert!(ret.children[0].is_none());
            assert!(ret.children[1].is_none());
            ret
        }
        fn aux<K: Ord>(anchor: &mut Anchor<K>, key: K) {
            match anchor {
                None => (),
                Some(node) => match key.cmp(&node.key) {
                    Ordering::Less => aux(&mut node.children[0], key),
                    Ordering::Greater => aux(&mut node.children[1], key),
                    Ordering::Equal => match (node.children[0].take(), node.children[1].take()) {
                        (None, None) => *anchor = None,
                        (Some(left), None) => *anchor = Some(left),
                        (None, Some(right)) => *anchor = Some(right),
                        (Some(left), Some(mut right)) => {
                            if right.children[0].is_none() {
                                right.children[0] = Some(left);
                                *anchor = Some(right);
                            } else {
                                let mut new_node = leftmost(&mut right);
                                new_node.children[0] = Some(left);
                                new_node.children[1] = Some(right);
                                *anchor = Some(new_node);
                            }
                        }
                    },
                },
            }
        }
        aux(&mut self.root, key);
    }
}

#[cfg(test)]
mod tests {
    use rand::seq::IteratorRandom;
    use rand::{Rng, SeedableRng};
    use std::collections::HashSet;

    #[test]
    fn test() {
        let mut treap = super::Treap::new();
        treap.print();
        for x in [5, 4, 2, 3, 9, 6, 8] {
            println!("Inserting {x}");
            treap.insert(x);
            treap.print();
        }
        for x in [5, 4, 2, 3, 9, 6, 8] {
            println!("Removing {x}");
            treap.remove(x);
            treap.print();
        }
    }
}
