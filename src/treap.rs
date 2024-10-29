use std::cmp::Ordering;

type Anchor<K> = Option<Box<Node<K>>>;

struct Node<K> {
    key: K,
    priority: u64,
    children: [Anchor<K>; 2],
}

impl<K> Node<K> {
    fn new(key: K) -> Self {
        Node {
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

impl<K> Default for Treap<K> {
    fn default() -> Self {
        Treap::new()
    }
}

impl<K: std::fmt::Display> Treap<K> {
    pub fn print(&self) {
        fn aux<K: std::fmt::Display>(anchor: &Anchor<K>, depth: usize) {
            let prefix = "    ".repeat(depth);
            if let Some(node) = anchor {
                println!("{}- {}", prefix, node.key);
                aux(&node.children[0], depth + 1);
                aux(&node.children[1], depth + 1);
            } else {
                println!("{}-", prefix);
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
            let Some(node) = anchor else {
                return;
            };
            // check this is a binary search tree
            if let Some(min_key) = min_key {
                assert!(node.key > *min_key);
            }
            if let Some(max_key) = max_key {
                assert!(node.key < *max_key);
            }
            // check this is a heap
            if let Some(parent_priority) = parent_priority {
                assert!(node.priority <= parent_priority);
            }
            // recurse
            let prio = Some(node.priority);
            aux(&node.children[0], min_key, Some(&node.key), prio);
            aux(&node.children[1], Some(&node.key), max_key, prio);
        }
        aux(&self.root, None, None, None);
    }

    fn rotate(anchor: &mut Anchor<K>, dir: usize) {
        let mut parent = anchor.take().unwrap();
        let mut new_parent = parent.children[dir].take().unwrap();
        assert!(new_parent.priority > parent.priority);
        parent.children[dir] = new_parent.children[1 - dir].take();
        new_parent.children[1 - dir] = Some(parent);
        *anchor = Some(new_parent);
    }

    pub fn insert(&mut self, key: K) {
        // returns true when we should check the heap invariant
        fn aux<K: Ord>(anchor: &mut Anchor<K>, key: K) -> bool {
            let Some(node) = anchor else {
                *anchor = Some(Box::new(Node::new(key)));
                return true;
            };
            let dir = match key.cmp(&node.key) {
                Ordering::Less => 0,
                Ordering::Greater => 1,
                Ordering::Equal => return false,
            };
            if !aux(&mut node.children[dir], key) {
                return false;
            }
            if node.children[dir].as_ref().unwrap().priority > node.priority {
                // bubble up
                Treap::rotate(anchor, dir);
                true
            } else {
                false
            }
        }
        aux(&mut self.root, key);
        self.check();
    }

    pub fn contains(&self, key: K) -> bool {
        fn aux<K: Ord>(anchor: &Anchor<K>, key: K) -> bool {
            if let Some(node) = anchor {
                match key.cmp(&node.key) {
                    Ordering::Less => aux(&node.children[0], key),
                    Ordering::Greater => aux(&node.children[1], key),
                    Ordering::Equal => true,
                }
            } else {
                false
            }
        }
        aux(&self.root, key)
    }

    pub fn remove(&mut self, key: K) {
        fn leftmost<K>(mut node: &mut Node<K>) -> Box<Node<K>> {
            while node.children[0].as_ref().unwrap().children[0].is_some() {
                node = node.children[0].as_mut().unwrap();
            }
            let mut ret = node.children[0].take().unwrap();
            node.children[0] = ret.children[1].take();
            assert!(ret.children[0].is_none());
            assert!(ret.children[1].is_none());
            ret
        }
        fn bubble_down<K: Ord>(mut anchor: &mut Anchor<K>) {
            loop {
                let node = anchor.as_mut().unwrap();
                let mut max_priority = node.priority;
                let mut max_priority_dir = 2;
                if let Some(child) = &node.children[0] {
                    if child.priority > max_priority {
                        max_priority = child.priority;
                        max_priority_dir = 0;
                    }
                }
                if let Some(child) = &node.children[1] {
                    if child.priority > max_priority {
                        // max_priority = child.priority;
                        max_priority_dir = 1;
                    }
                }
                if max_priority_dir == 2 {
                    break;
                }
                Treap::rotate(anchor, max_priority_dir);
                anchor = &mut anchor.as_mut().unwrap().children[1 - max_priority_dir];
            }
        }
        fn aux<K: Ord>(anchor: &mut Anchor<K>, key: K) {
            let Some(node) = anchor else {
                return;
            };
            match key.cmp(&node.key) {
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
                            bubble_down(anchor);
                        } else {
                            let mut new_node = leftmost(&mut right);
                            new_node.children[0] = Some(left);
                            new_node.children[1] = Some(right);
                            *anchor = Some(new_node);
                            bubble_down(anchor);
                        }
                    }
                },
            }
        }
        aux(&mut self.root, key);
        self.check();
    }
}

// non-consuming iterator
pub struct IterRef<'a, K> {
    stack: Vec<(bool, &'a Node<K>)>,
}

impl<'a, K> IterRef<'a, K> {
    fn new(treap: &'a Treap<K>) -> Self {
        if let Some(node) = &treap.root {
            IterRef {
                stack: vec![(false, node)],
            }
        } else {
            IterRef { stack: vec![] }
        }
    }
}

impl<'a, K> Iterator for IterRef<'a, K> {
    type Item = &'a K;
    fn next(&mut self) -> Option<Self::Item> {
        let (explored, node) = self.stack.pop()?;
        #[allow(clippy::collapsible_else_if)] // clearer to see the two cases this way
        if explored {
            if let Some(child) = &node.children[1] {
                self.stack.push((false, child));
                Some(&node.key)
            } else {
                Some(&node.key)
            }
        } else {
            if let Some(child) = &node.children[0] {
                self.stack.push((true, node));
                self.stack.push((false, child));
                self.next()
            } else if let Some(child) = &node.children[1] {
                self.stack.push((false, child));
                Some(&node.key)
            } else {
                Some(&node.key)
            }
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
    stack: Vec<Box<Node<K>>>,
}

impl<K> Iter<K> {
    fn new(treap: Treap<K>) -> Self {
        if let Some(node) = treap.root {
            Iter { stack: vec![node] }
        } else {
            Iter { stack: vec![] }
        }
    }
}

impl<K> Iterator for Iter<K> {
    type Item = K;

    fn next(&mut self) -> Option<Self::Item> {
        let mut node = self.stack.pop()?;
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
        let mut tree = super::Treap::new();
        tree.print();

        // add some
        for x in [5, 4, 2, 3, 9, 6, 8] {
            println!("Inserting {x}");
            tree.insert(x);
            tree.print();
        }

        // remove some
        for x in [5, 4, 2, 3, 9, 6, 8] {
            println!("Removing {x}");
            tree.remove(x);
            tree.print();
            tree.check();
        }
    }

    #[test]
    fn big_test() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let mut tree = super::Treap::new();
        let mut expected = HashSet::new();

        // try to unbalance the tree
        for x in 0..10000 {
            tree.insert(x);
            expected.insert(x);
        }

        // add some more
        for _ in 0..10000 {
            let x: u64 = rng.gen();
            tree.insert(x);
            expected.insert(x);
        }
        let actual: HashSet<_> = tree.iter().copied().collect();
        assert_eq!(actual, expected);

        // remove some
        for _ in 0..1000 {
            let x: u64 = *expected.iter().choose(&mut rng).unwrap();
            tree.remove(x);
            expected.remove(&x);
        }
        let actual: HashSet<_> = tree.iter().copied().collect();
        assert_eq!(actual, expected);
    }
}
