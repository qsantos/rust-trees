use std::cmp::Ordering;

type Anchor<K> = Option<Box<AvlNode<K>>>;

enum Balance {
    LeftLonger,
    RightLonger,
    Balanced,
}

struct AvlNode<K> {
    key: K,
    balance: Balance,
    left: Anchor<K>,
    right: Anchor<K>,
}

impl<K> AvlNode<K> {
    fn new(key: K) -> Self {
        AvlNode {
            key,
            balance: Balance::Balanced,
            left: None,
            right: None,
        }
    }
}

pub struct Avl<K> {
    root: Anchor<K>,
}

impl<K> Avl<K> {
    pub fn new() -> Self {
        Avl { root: None }
    }
}

impl<K> Default for Avl<K> {
    fn default() -> Self {
        Avl::new()
    }
}

impl<K: std::fmt::Display> Avl<K> {
    pub fn print(&self) {
        fn aux<K: std::fmt::Display>(anchor: &Anchor<K>, indent: usize) {
            let prefix = "    ".repeat(indent);
            match anchor {
                None => println!("{}-", prefix),
                Some(node) => {
                    println!("{}-{}", prefix, node.key);
                    aux(&node.left, indent + 1);
                    aux(&node.right, indent + 1);
                }
            }
        }
        aux(&self.root, 0);
    }
}

impl<K: Ord> Avl<K> {
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

    pub fn contains(&self, key: K) -> bool {
        fn aux<K: Ord>(anchor: &Anchor<K>, key: K) -> bool {
            match anchor {
                None => false,
                Some(node) => match key.cmp(&node.key) {
                    Ordering::Less => aux(&node.left, key),
                    Ordering::Greater => aux(&node.right, key),
                    Ordering::Equal => true,
                },
            }
        }
        aux(&self.root, key)
    }

    pub fn insert(&mut self, key: K) {
        fn aux<K: Ord>(anchor: &mut Anchor<K>, key: K) {
            match anchor {
                None => *anchor = Some(Box::new(AvlNode::new(key))),
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

    pub fn remove(&mut self, key: K) {
        fn leftmost<K>(mut node: &mut Box<AvlNode<K>>) -> Box<AvlNode<K>> {
            while node.left.as_ref().unwrap().left.is_some() {
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
                        (Some(left), Some(mut right)) => match right.left {
                            None => {
                                right.left = Some(left);
                                *anchor = Some(right);
                            }
                            Some(_) => {
                                let mut node = leftmost(&mut right);
                                node.left = Some(left);
                                node.right = Some(right);
                                *anchor = Some(node);
                            }
                        },
                    },
                },
            }
        }
        aux(&mut self.root, key);
        self.check();
    }
}

impl<K: Ord> FromIterator<K> for Avl<K> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = K>,
    {
        let mut avl = Avl::new();
        for x in iter {
            avl.insert(x);
        }
        avl
    }
}

// non-consuming iterator

enum ExplorationState {
    Unexplored,
    YieldedLeft,
}

pub struct IterRef<'a, K> {
    stack: Vec<(ExplorationState, &'a Anchor<K>)>,
}

impl<'a, K> Iterator for IterRef<'a, K> {
    type Item = &'a K;
    fn next(&mut self) -> Option<Self::Item> {
        let stack = &mut self.stack;
        if let Some((state, anchor)) = stack.pop() {
            match anchor {
                None => self.next(),
                Some(node) => match state {
                    ExplorationState::Unexplored => {
                        stack.push((ExplorationState::YieldedLeft, anchor));
                        stack.push((ExplorationState::Unexplored, &node.left));
                        self.next()
                    }
                    ExplorationState::YieldedLeft => {
                        stack.push((ExplorationState::Unexplored, &node.right));
                        Some(&node.key)
                    }
                },
            }
        } else {
            None
        }
    }
}

impl<'a, K> IntoIterator for &'a Avl<K> {
    type Item = &'a K;
    type IntoIter = IterRef<'a, K>;
    fn into_iter(self) -> Self::IntoIter {
        IterRef {
            stack: vec![(ExplorationState::Unexplored, &self.root)],
        }
    }
}

// consuming iterator

pub struct Iter<K> {
    stack: Vec<Anchor<K>>,
}

impl<K> Iterator for Iter<K> {
    type Item = K;
    fn next(&mut self) -> Option<Self::Item> {
        let stack = &mut self.stack;
        if let Some(anchor) = stack.pop() {
            match anchor {
                None => self.next(),
                Some(mut node) => {
                    if let Some(left) = node.left.take() {
                        stack.push(Some(node));
                        stack.push(Some(left));
                        return self.next();
                    }
                    stack.push(node.right.take());
                    Some(node.key)
                }
            }
        } else {
            None
        }
    }
}

impl<K> IntoIterator for Avl<K> {
    type IntoIter = Iter<K>;
    type Item = K;
    fn into_iter(self) -> Self::IntoIter {
        Iter {
            stack: vec![self.root],
        }
    }
}

impl<K> Avl<K> {
    pub fn iter(&self) -> IterRef<K> {
        self.into_iter()
    }
}

#[test]
fn test() {
    let mut t: Avl<i32> = [8, 4, 2, 1, 3, 6, 5, 7, 12, 10, 9, 11, 14, 13, 15]
        .iter()
        .copied()
        .collect();

    t.remove(8);

    let expected = vec![1, 2, 3, 4, 5, 6, 7, 9, 10, 11, 12, 13, 14, 15];

    let v: Vec<i32> = t.iter().copied().collect();
    assert_eq!(v, expected);

    let mut v = Vec::new();
    for &x in &t {
        v.push(x);
    }
    assert_eq!(v, expected);

    let mut v = Vec::new();
    for x in t {
        v.push(x);
    }
    assert_eq!(v, expected);
}
