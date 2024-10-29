use std::cmp::Ordering;

type Anchor<K> = Option<Box<Node<K>>>;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum NodeDirection {
    Left = 0,
    Right = 1,
    None,
}

struct Node<K> {
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

impl<K> Node<K> {
    fn new(key: K) -> Self {
        Node {
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

impl<K: Ord> Node<K> {
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
            if let Some(node) = anchor {
                println!("{}- {} ({:?})", prefix, node.key, node.longer_side);
                aux(&node.children[0], indent + 1);
                aux(&node.children[1], indent + 1);
            } else {
                println!("{}-", prefix);
            }
        }
        aux(&self.root, 0);
    }
}

impl<K: Ord> Avl<K> {
    fn check(&self) {
        // returns the height
        fn aux<K: Ord>(anchor: &Anchor<K>, min: Option<&K>, max: Option<&K>) -> i32 {
            let Some(node) = anchor else {
                return 1;
            };
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
        aux(&self.root, None, None);
    }

    pub fn contains(&self, key: K) -> bool {
        fn aux<K: Ord>(anchor: &Anchor<K>, key: K) -> bool {
            let Some(node) = anchor else {
                return false;
            };
            match node.dir(&key) {
                NodeDirection::None => true,
                dir => aux(&node.children[dir as usize], key),
            }
        }
        aux(&self.root, key)
    }

    // For an insertion:
    //     - dir is the insertion direction
    //     - the return value is true when the height is still increased after rebalancing
    // For a deletion:
    //      - dir is the opposite of the insertion direction
    //      - the return value is true when the height is still decreased after rebalancing
    fn rebalance(anchor: &mut Anchor<K>, dir: NodeDirection) -> bool {
        let node = anchor.as_mut().unwrap();
        match node.longer_side {
            // the node was balanced
            NodeDirection::None => {
                // it becomes unbalanced
                node.longer_side = dir;
                // and the height is still increased (this is the only case)
                return true;
            }
            // the node was already unbalanced, in the opposite direction
            longer if longer != dir => {
                // the node has been rebalanced
                node.longer_side = NodeDirection::None;
                // the height has not changed
            }
            // the node was already unbalanced, in the same direction
            _ => {
                // the node has become even more unbalanced, a rebalancing is needed
                let child = node.children[dir as usize].as_mut().unwrap();
                match child.longer_side {
                    // the child is balanced
                    NodeDirection::None => {
                        // this will not fully rebalance the nodes
                        node.longer_side = dir;
                        child.longer_side = !dir;
                        // need a single rotation
                        rotate(anchor, !dir);
                        // this cannot happen during insertion
                        return true;
                    }
                    // the child is unbalanced in the same direction
                    cdir if cdir == dir => {
                        // this will rebalance both nodes fully
                        node.longer_side = NodeDirection::None;
                        child.longer_side = NodeDirection::None;
                        // need a single rotation
                        rotate(anchor, !dir);
                        // the height change is absorbed
                    }
                    // the child is unbalanced in the opposite direction
                    _ => {
                        let grandchild = child.children[!dir as usize].as_mut().unwrap();
                        // the node and child (new sides) still be unbalanced
                        match grandchild.longer_side {
                            NodeDirection::None => {
                                node.longer_side = NodeDirection::None;
                                child.longer_side = NodeDirection::None;
                                // this cannot happen during insertion
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
                    }
                }
            }
        }
        false
    }

    pub fn insert(&mut self, key: K) {
        // returns whether the height has increased
        fn aux<K: Ord>(anchor: &mut Anchor<K>, key: K) -> bool {
            let Some(node) = anchor else {
                *anchor = Some(Box::new(Node::new(key)));
                return true;
            };
            match node.dir(&key) {
                NodeDirection::None => false,
                dir => {
                    if !aux(&mut node.children[dir as usize], key) {
                        return false;
                    }
                    // the height has increased, we need to rebalance
                    Avl::rebalance(anchor, dir)
                }
            }
        }
        aux(&mut self.root, key);
        self.check();
    }

    pub fn remove(&mut self, key: K) {
        // return the leftmost node and its depth
        fn leftmost<K: Ord>(mut node: &mut Box<Node<K>>) -> (Box<Node<K>>, usize) {
            let mut depth = 0;
            while node.children[0].as_ref().unwrap().children[0].is_some() {
                node = node.children[0].as_mut().unwrap();
                depth += 1;
            }
            let mut ret = node.children[0].take().unwrap();
            node.children[0] = ret.children[1].take();
            (ret, depth)
        }
        // returns whether the height has decreased
        // we have replaced the leftmost node with its own right child
        // so we need to know the depth to know where to start from
        fn leftmost_rebalance<K: Ord>(anchor: &mut Anchor<K>, depth: usize) -> bool {
            let node = anchor.as_mut().unwrap();
            if depth > 0 {
                if !leftmost_rebalance(&mut node.children[0], depth - 1) {
                    false
                } else {
                    // we have reduced the height by one on the left, we need to rebalance
                    !Avl::rebalance(anchor, NodeDirection::Right)
                }
            } else {
                // we have reduced the height by one on the left, we need to rebalance
                !Avl::rebalance(anchor, NodeDirection::Right)
            }
        }
        // returns whether the height has decreased
        fn aux<K: Ord>(anchor: &mut Anchor<K>, key: K) -> bool {
            let Some(node) = anchor else {
                return false;
            };
            match node.dir(&key) {
                NodeDirection::None => {
                    match (node.children[0].take(), node.children[1].take()) {
                        (None, None) => {
                            *anchor = None;
                            true
                        }
                        (Some(left), None) => {
                            *anchor = Some(left);
                            true
                        }
                        (None, Some(right)) => {
                            *anchor = Some(right);
                            true
                        }
                        (Some(left), Some(mut right)) => match right.children[0] {
                            None => {
                                right.children[0] = Some(left);
                                right.longer_side = node.longer_side;
                                *anchor = Some(right);
                                // we have reduced the height by one on the right, we need to rebalance
                                !Avl::rebalance(anchor, NodeDirection::Left)
                            }
                            Some(_) => {
                                let (mut new_node, depth) = leftmost(&mut right);
                                new_node.longer_side = node.longer_side;
                                new_node.children[0] = Some(left);
                                new_node.children[1] = Some(right);
                                // we might need to rebalance some nodes in the right subtree
                                let ret = leftmost_rebalance(&mut new_node.children[1], depth);
                                *anchor = Some(new_node);
                                if ret {
                                    // we have reduced the height by one on the right, we need to rebalance
                                    !Avl::rebalance(anchor, NodeDirection::Left)
                                } else {
                                    false
                                }
                            }
                        },
                    }
                }
                dir => {
                    if !aux(&mut node.children[dir as usize], key) {
                        return false;
                    }
                    // the height has decreased, we need to rebalance
                    !Avl::rebalance(anchor, !dir)
                }
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
pub struct IterRef<'a, K> {
    stack: Vec<(bool, &'a Anchor<K>)>,
}

impl<'a, K> Iterator for IterRef<'a, K> {
    type Item = &'a K;
    fn next(&mut self) -> Option<Self::Item> {
        let stack = &mut self.stack;
        let (explored, anchor) = stack.pop()?;
        let Some(node) = anchor else {
            return self.next();
        };
        if explored {
            stack.push((false, &node.children[1]));
            Some(&node.key)
        } else {
            stack.push((true, anchor));
            stack.push((false, &node.children[0]));
            self.next()
        }
    }
}

impl<'a, K> IntoIterator for &'a Avl<K> {
    type Item = &'a K;
    type IntoIter = IterRef<'a, K>;
    fn into_iter(self) -> Self::IntoIter {
        IterRef {
            stack: vec![(false, &self.root)],
        }
    }
}

impl<K> Avl<K> {
    pub fn iter(&self) -> IterRef<K> {
        self.into_iter()
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
        let anchor = stack.pop()?;
        let Some(mut node) = anchor else {
            return self.next();
        };
        let Some(left) = node.children[0].take() else {
            stack.push(node.children[1].take());
            return Some(node.key);
        };
        stack.push(Some(node));
        stack.push(Some(left));
        self.next()
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

#[cfg(test)]
mod tests {
    use rand::seq::IteratorRandom;
    use rand::{Rng, SeedableRng};
    use std::collections::HashSet;

    #[test]
    fn big_test() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let mut tree = super::Avl::new();
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
