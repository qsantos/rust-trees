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

// non-consuming iterator
enum ExplorationState {
    Unexplored,
    LeftYielded,
}

pub struct IterRef<'a, K> {
    stack: Vec<(ExplorationState, &'a TreapNode<K>)>,
}

impl<'a, K> IterRef<'a, K> {
    fn new(treap: &'a Treap<K>) -> Self {
        match &treap.root {
            None => IterRef { stack: vec![] },
            Some(node) => IterRef {
                stack: vec![(ExplorationState::Unexplored, node)],
            },
        }
    }
}

impl<'a, K> Iterator for IterRef<'a, K> {
    type Item = &'a K;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some((state, node)) = self.stack.pop() {
            match state {
                ExplorationState::Unexplored => {
                    if let Some(child) = &node.children[0] {
                        self.stack.push((ExplorationState::LeftYielded, node));
                        self.stack.push((ExplorationState::Unexplored, child));
                        self.next()
                    } else {
                        if let Some(child) = &node.children[1] {
                            self.stack.push((ExplorationState::Unexplored, child));
                            Some(&node.key)
                        } else {
                            Some(&node.key)
                        }
                    }
                }
                ExplorationState::LeftYielded => {
                    if let Some(child) = &node.children[1] {
                        self.stack.push((ExplorationState::Unexplored, child));
                        Some(&node.key)
                    } else {
                        Some(&node.key)
                    }
                }
            }
        } else {
            None
        }
    }
}

impl<'a, K> IntoIterator for &'a Treap<K> {
    type IntoIter = IterRef<'a, K>;
    type Item = &'a K;
    fn into_iter(self) -> Self::IntoIter {
        IterRef::new(self)
    }
}

// consuming iterator
pub struct Iter<K> {
    stack: Vec<Box<TreapNode<K>>>,
}

impl<K> Iter<K> {
    fn new(treap: Treap<K>) -> Self {
        match treap.root {
            None => Iter { stack: vec![] },
            Some(node) => Iter { stack: vec![node] },
        }
    }
}

impl<K> Iterator for Iter<K> {
    type Item = K;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(mut node) = self.stack.pop() {
            if let Some(child) = node.children[0].take() {
                self.stack.push(node);
                self.stack.push(child);
                self.next()
            } else {
                let k = node.key;
                if let Some(child) = node.children[1].take() {
                    self.stack.push(child);
                }
                Some(k)
            }
        } else {
            None
        }
    }
}

impl<K> IntoIterator for Treap<K> {
    type IntoIter = Iter<K>;
    type Item = K;
    fn into_iter(self) -> Self::IntoIter {
        Iter::new(self)
    }
}

impl<K> Treap<K> {
    pub fn iter(&self) -> IterRef<K> {
        self.into_iter()
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

    #[test]
    fn big_test() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let mut treap = super::Treap::new();
        let mut expected = HashSet::new();

        // try to unbalance the tree
        for x in 0..10000 {
            treap.insert(x);
            expected.insert(x);
        }

        // add some more
        for _ in 0..10000 {
            let x: u64 = rng.gen();
            treap.insert(x);
            expected.insert(x);
        }
        let actual: HashSet<_> = treap.iter().copied().collect();
        assert_eq!(actual, expected);

        // remove some
        for _ in 0..1000 {
            let x: u64 = *expected.iter().choose(&mut rng).unwrap();
            treap.remove(x);
            expected.remove(&x);
        }
        let actual: HashSet<_> = treap.iter().copied().collect();
        assert_eq!(actual, expected);
    }
}
