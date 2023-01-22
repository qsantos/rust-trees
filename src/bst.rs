use std::cmp::Ordering;

type Anchor<K> = Option<Box<BstNode<K>>>;

struct BstNode<K> {
    key: K,
    children: [Anchor<K>; 2],
}

impl<K> BstNode<K> {
    fn new(key: K) -> Self {
        BstNode {
            key,
            children: [None, None],
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
                    aux(&node.children[0], indent + 1);
                    aux(&node.children[1], indent + 1);
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
                    aux(&node.children[0], min, Some(&node.key));
                    aux(&node.children[1], Some(&node.key), max);
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
        fn aux<K: Ord>(anchor: &Anchor<K>, key: &K) -> bool {
            match anchor {
                None => false,
                Some(node) => match key.cmp(&node.key) {
                    Ordering::Less => aux(&node.children[0], key),
                    Ordering::Greater => aux(&node.children[1], key),
                    Ordering::Equal => true,
                },
            }
        }
        aux(&self.root, &key)
    }

    pub fn remove(&mut self, key: K) {
        fn leftmost<K>(mut node: &mut Box<BstNode<K>>) -> Box<BstNode<K>> {
            while node.children[0].as_ref().unwrap().children[0].is_some() {
                node = node.children[0].as_mut().unwrap();
            }
            let mut ret = node.children[0].take().unwrap();
            node.children[0] = ret.children[1].take();
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
                                let mut node = leftmost(&mut right);
                                node.children[0] = Some(left);
                                node.children[1] = Some(right);
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

impl<K: Ord> FromIterator<K> for Bst<K> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = K>,
    {
        let mut bst = Bst::new();
        for k in iter {
            bst.insert(k);
        }
        bst
    }
}

/*
fn iter(anchor: &Anchor<K>) -> Iterator<&K> {
    match anchor {
        None => (),
        Some(node) => {
            yield from iter(&node.children[0]);
            yield &node.key;
            yield from iter(&node.children[1]);
        },
    }
}
*/

enum ExplorationState {
    Unexplored,
    YieldedLeft,
}

// non-consuming iterator
pub struct IterRef<'a, K> {
    stack: Vec<(ExplorationState, &'a Anchor<K>)>,
}

impl<'a, K> IterRef<'a, K> {
    fn new(anchor: &'a Anchor<K>) -> Self {
        IterRef {
            stack: vec![(ExplorationState::Unexplored, anchor)],
        }
    }
}

impl<'a, K> Iterator for IterRef<'a, K> {
    type Item = &'a K;
    fn next(&mut self) -> Option<&'a K> {
        let stack = &mut self.stack;
        if let Some((state, anchor)) = stack.pop() {
            match anchor {
                None => self.next(),
                Some(node) => {
                    match state {
                        ExplorationState::Unexplored => {
                            // yield from iter(&node.children[0]);
                            stack.push((ExplorationState::YieldedLeft, anchor));
                            stack.push((ExplorationState::Unexplored, &node.children[0]));
                            self.next()
                        }
                        ExplorationState::YieldedLeft => {
                            // yield &node.key;
                            // yield from iter(&node.children[1]);
                            stack.push((ExplorationState::Unexplored, &node.children[1]));
                            Some(&node.key)
                        }
                    }
                }
            }
        } else {
            None
        }
    }
}

impl<'a, K> IntoIterator for &'a Bst<K> {
    type Item = &'a K;
    type IntoIter = IterRef<'a, K>;
    fn into_iter(self) -> Self::IntoIter {
        IterRef::new(&self.root)
    }
}

impl<K> Bst<K> {
    pub fn iter(&self) -> IterRef<K> {
        self.into_iter()
    }
}

// consuming iterator
pub struct Iter<K> {
    stack: Vec<Anchor<K>>,
}

impl<K> Iter<K> {
    fn new(tree: Bst<K>) -> Self {
        Iter {
            stack: vec![tree.root],
        }
    }
}

impl<K> Iterator for Iter<K> {
    type Item = K;
    fn next(&mut self) -> Option<Self::Item> {
        let stack = &mut self.stack;
        if let Some(anchor) = stack.pop() {
            match anchor {
                None => self.next(),
                Some(mut node) => {
                    if let Some(left) = node.children[0].take() {
                        stack.push(Some(node));
                        stack.push(Some(left));
                        return self.next();
                    }
                    if let Some(right) = node.children[1].take() {
                        stack.push(Some(right));
                    }
                    Some(node.key)
                }
            }
        } else {
            None
        }
    }
}

impl<K> IntoIterator for Bst<K> {
    type Item = K;
    type IntoIter = Iter<K>;
    fn into_iter(self) -> Self::IntoIter {
        Iter::new(self)
    }
}

#[test]
fn test() {
    let mut t: Bst<i32> = [8, 4, 2, 1, 3, 6, 5, 7, 12, 10, 9, 11, 14, 13, 15]
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
