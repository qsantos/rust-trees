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
    children: [Anchor<K>; 2],
}

enum NodeDirection {
    Left = 0,
    Right = 1,
}

impl std::ops::Not for NodeDirection {
    type Output = NodeDirection;
    fn not(self) -> Self::Output {
        match self {
            NodeDirection::Left => NodeDirection::Right,
            NodeDirection::Right => NodeDirection::Left,
        }
    }
}

impl<K> AvlNode<K> {
    fn new(key: K) -> Self {
        AvlNode {
            key,
            balance: Balance::Balanced,
            children: [None, None],
        }
    }
}

impl<K: Ord> AvlNode<K> {
    fn dir(&self, key: &K) -> Option<NodeDirection> {
        match key.cmp(&self.key) {
            Ordering::Less => Some(NodeDirection::Left),
            Ordering::Greater => Some(NodeDirection::Right),
            Ordering::Equal => None,
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
                    println!("{}- {}", prefix, node.key);
                    aux(&node.children[0], indent + 1);
                    aux(&node.children[1], indent + 1);
                }
            }
        }
        aux(&self.root, 0);
    }
}

impl<K: Ord> Avl<K> {
    fn check(&self) {
        // returns the height
        fn aux<K: Ord>(anchor: &Anchor<K>, min: Option<&K>, max: Option<&K>) -> usize {
            match anchor {
                None => 1,
                Some(node) => {
                    if let Some(min) = min {
                        assert!(node.key > *min);
                    }
                    if let Some(max) = max {
                        assert!(node.key < *max);
                    }
                    let lh = aux(&node.children[0], min, Some(&node.key));
                    let lr = aux(&node.children[1], Some(&node.key), max);
                    match lh.cmp(&lr) {
                        Ordering::Less => {
                            assert_eq!(node.longer_side, Some(NodeDirection::Right));
                        }
                        Ordering::Greater => {
                            assert_eq!(node.longer_side, Some(NodeDirection::Left));
                        }
                        Ordering::Equal => assert!(node.longer_side.is_none()),
                    }
                    lh.max(lr) + 1
                }
            }
        }
        aux(&self.root, None, None);
    }

    pub fn contains(&self, key: K) -> bool {
        fn aux<K: Ord>(anchor: &Anchor<K>, key: K) -> bool {
            match anchor {
                None => false,
                Some(node) => {
                    let dir = node.dir(&key);
                    if let Some(dir) = dir {
                        aux(&node.children[dir as usize], key)
                    } else {
                        true
                    }
                }
            }
        }
        aux(&self.root, key)
    }

    pub fn insert(&mut self, key: K) {
        fn aux<K: Ord>(anchor: &mut Anchor<K>, key: K) {
            match anchor {
                None => *anchor = Some(Box::new(AvlNode::new(key))),
                Some(node) => {
                    let dir = node.dir(&key);
                    if let Some(dir) = dir {
                        aux(&mut node.children[dir as usize], key);
                    }
                }
            }
        }
        aux(&mut self.root, key);
        // self.check();
    }

    pub fn remove(&mut self, key: K) {
        fn leftmost<K>(mut node: &mut Box<AvlNode<K>>) -> Box<AvlNode<K>> {
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
                Some(node) => {
                    let dir = node.dir(&key);
                    if let Some(dir) = dir {
                        aux(&mut node.children[dir as usize], key);
                    } else {
                        match (node.children[0].take(), node.children[1].take()) {
                            (None, None) => *anchor = None,
                            (Some(left), None) => *anchor = Some(left),
                            (None, Some(right)) => *anchor = Some(right),
                            (Some(left), Some(mut right)) => match right.children[0] {
                                None => {
                                    right.children[0] = Some(left);
                                    *anchor = Some(right);
                                }
                                Some(_) => {
                                    let mut node = leftmost(&mut right);
                                    node.children[0] = Some(left);
                                    node.children[1] = Some(right);
                                    *anchor = Some(node);
                                }
                            },
                        }
                    }
                }
            }
        }
        aux(&mut self.root, key);
        // self.check();
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
                        stack.push((ExplorationState::Unexplored, &node.children[0]));
                        self.next()
                    }
                    ExplorationState::YieldedLeft => {
                        stack.push((ExplorationState::Unexplored, &node.children[1]));
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
                    if let Some(left) = node.children[0].take() {
                        stack.push(Some(node));
                        stack.push(Some(left));
                        return self.next();
                    }
                    stack.push(node.children[1].take());
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
