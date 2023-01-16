use std::cmp::Ordering;

type Anchor<K> = Option<Box<AvlNode<K>>>;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum NodeDirection {
    Left = 0,
    Right = 1,
    None,
}

struct AvlNode<K> {
    key: K,
    longer_side: NodeDirection,
    children: [Anchor<K>; 2],
}

impl std::ops::Not for NodeDirection {
    type Output = NodeDirection;
    fn not(self) -> Self::Output {
        match self {
            NodeDirection::Left => NodeDirection::Right,
            NodeDirection::Right => NodeDirection::Left,
            NodeDirection::None => NodeDirection::None,
        }
    }
}

impl<K> AvlNode<K> {
    fn new(key: K) -> Self {
        AvlNode {
            key,
            longer_side: NodeDirection::None,
            children: [None, None],
        }
    }
}

fn rotate<K>(anchor: &mut Anchor<K>, dir: NodeDirection) {
    let mut node = anchor.take().unwrap();
    let mut new_root = node.children[!dir as usize].take().unwrap();
    node.children[!dir as usize] = new_root.children[dir as usize].take();
    new_root.children[dir as usize] = Some(node);
    *anchor = Some(new_root);
}

impl<K: Ord> AvlNode<K> {
    fn dir(&self, key: &K) -> NodeDirection {
        match key.cmp(&self.key) {
            Ordering::Less => NodeDirection::Left,
            Ordering::Greater => NodeDirection::Right,
            Ordering::Equal => NodeDirection::None,
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
        fn aux<K: Ord>(anchor: &Anchor<K>, min: Option<&K>, max: Option<&K>) -> i32 {
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
                    match lr - lh {
                        1 => assert_eq!(node.longer_side, NodeDirection::Right),
                        -1 => assert_eq!(node.longer_side, NodeDirection::Left),
                        0 => assert_eq!(node.longer_side, NodeDirection::None),
                        _ => unreachable!(),
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
                Some(node) => match node.dir(&key) {
                    NodeDirection::None => true,
                    dir => aux(&node.children[dir as usize], key),
                },
            }
        }
        aux(&self.root, key)
    }

    pub fn insert(&mut self, key: K) {
        // returns whether the height has increased
        fn aux<K: Ord>(anchor: &mut Anchor<K>, key: K) -> bool {
            match anchor {
                None => {
                    *anchor = Some(Box::new(AvlNode::new(key)));
                    true
                }
                Some(node) => {
                    match node.dir(&key) {
                        NodeDirection::None => false,
                        dir => {
                            if aux(&mut node.children[dir as usize], key) {
                                // the height has increased
                                match node.longer_side {
                                    // the node was balanced
                                    NodeDirection::None => {
                                        // it becomes unbalanced
                                        node.longer_side = dir;
                                        // and the height is still increased
                                        true
                                    }
                                    // the node was already unbalanced, in the opposite direction
                                    longer if longer != dir => {
                                        // the node has been rebalanced
                                        node.longer_side = NodeDirection::None;
                                        // the height has not changed
                                        false
                                    }
                                    // the node was already unbalanced, in the same direction
                                    _ => {
                                        // the node has become even more unbalanced, a rebalancing is needed
                                        let child = node.children[dir as usize].as_mut().unwrap();
                                        match child.longer_side {
                                            // the child is balanced
                                            NodeDirection::None => {
                                                // this cannot happen during insertion
                                                unreachable!();
                                            }
                                            // the child is unbalanced in the same direction
                                            cdir if cdir == dir => {
                                                // this will rebalance both nodes fully
                                                node.longer_side = NodeDirection::None;
                                                child.longer_side = NodeDirection::None;
                                                // need a single rotation
                                                rotate(anchor, !dir);
                                                // the height change is absorbed
                                                false
                                            }
                                            // the child is unbalanced in the opposite direction
                                            _ => {
                                                let grandchild =
                                                    child.children[!dir as usize].as_mut().unwrap();
                                                // the node and child (new sides) might still be unbalanced
                                                match grandchild.longer_side {
                                                    NodeDirection::None => {
                                                        node.longer_side = NodeDirection::None;
                                                        child.longer_side = NodeDirection::None;
                                                    }
                                                    gdir if gdir == dir => {
                                                        node.longer_side = !dir;
                                                        child.longer_side = NodeDirection::None;
                                                    }
                                                    _ => {
                                                        node.longer_side = NodeDirection::None;
                                                        child.longer_side = dir;
                                                    }
                                                }
                                                // this will always rebalance the grandchild (the new root)
                                                grandchild.longer_side = NodeDirection::None;
                                                // need two rotations
                                                rotate(&mut node.children[dir as usize], dir);
                                                rotate(anchor, !dir);
                                                // the height change is absorbed
                                                false
                                            }
                                        }
                                    }
                                }
                            } else {
                                false
                            }
                        }
                    }
                }
            }
        }
        aux(&mut self.root, key);
        self.check();
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
                Some(node) => match node.dir(&key) {
                    NodeDirection::None => {
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
                    dir => aux(&mut node.children[dir as usize], key),
                },
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
    let mut t: Avl<i32> = [1, 2, 3, 4, 5, 6, 7, 8, 15, 14, 13, 12, 11, 10, 9]
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
