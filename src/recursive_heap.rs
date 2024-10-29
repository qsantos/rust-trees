type Anchor<K> = Option<Box<Node<K>>>;

struct Node<K> {
    key: K,
    children: [Anchor<K>; 2],
}

impl<K> Node<K> {
    fn new(key: K) -> Self {
        Node {
            key,
            children: [None, None],
        }
    }
}

pub struct RecursiveHeap<K> {
    root: Anchor<K>,
    size: usize,
}

impl<K> RecursiveHeap<K> {
    pub fn new() -> Self {
        RecursiveHeap {
            root: None,
            size: 0,
        }
    }
}

impl<K> Default for RecursiveHeap<K> {
    fn default() -> Self {
        RecursiveHeap::new()
    }
}

impl<K: std::fmt::Display> RecursiveHeap<K> {
    pub fn print(&self) {
        fn aux<K: std::fmt::Display>(anchor: &Anchor<K>, indent: usize) {
            let prefix = "    ".repeat(indent);
            if let Some(node) = anchor {
                println!("{}- {}", prefix, node.key);
                aux(&node.children[0], indent + 1);
                aux(&node.children[1], indent + 1);
            } else {
                println!("{}-", prefix);
            }
        }
        aux(&self.root, 0);
    }
}

/// Gives a path to the element at the given element in a complete binary tree
///
/// This is done by iterating over the bits of `index + 1` from the most
/// significant bit (MSB) to the least significant bit (LSB). The leading 1 is ignored.
///
/// The return value can be iterated by looking iteratively at the LSB and
/// dividing by 2. To get the bits in the right order, they are reversed, and
/// the leading zeros are removed.
///
/// We remove the leading 1 by dividing by 2 after the shift to avoid shifting
/// more than possible.
fn binary_path_to(mut index: usize) -> usize {
    index += 1;
    (index.reverse_bits() >> index.leading_zeros()) / 2
}

impl<K: Ord> RecursiveHeap<K> {
    fn check(&self) {
        // returns the number of nodes
        fn aux<K: Ord>(anchor: &Anchor<K>, parent_key: Option<&K>) -> usize {
            let Some(node) = anchor else {
                return 0;
            };
            // ensure order is correct
            if let Some(parent_key) = parent_key {
                assert!(node.key < *parent_key);
            }
            // ensure the tree is complete
            if node.children[0].is_none() {
                assert!(node.children[1].is_none());
            }
            // recurse and return the height
            1 + aux(&node.children[0], Some(&node.key)) + aux(&node.children[1], Some(&node.key))
        }
        let size = aux(&self.root, None);
        assert_eq!(self.size, size);
    }

    pub fn push(&mut self, key: K) {
        fn bubble_up<K: Ord>(anchor: &mut Anchor<K>, key: K, path: usize) {
            let Some(node) = anchor else {
                *anchor = Some(Box::new(Node::new(key)));
                return;
            };
            let dir = path % 2;
            bubble_up(&mut node.children[dir], key, path / 2);
            // swap if needed
            let child = node.children[dir].as_mut().unwrap();
            if child.key > node.key {
                std::mem::swap(&mut child.key, &mut node.key);
            }
        }
        let path = binary_path_to(self.size);
        bubble_up(&mut self.root, key, path);
        self.size += 1;
        self.check();
    }

    pub fn peek(&self) -> Option<&K> {
        Some(&self.root.as_ref()?.key)
    }

    pub fn pop(&mut self) -> Option<K> {
        fn last_key<K>(node: &mut Node<K>, path: usize) -> K {
            let dir = path % 2;
            if node.children[dir].as_ref().unwrap().children[0].is_none() {
                node.children[dir].take().unwrap().key
            } else {
                last_key(node.children[dir].as_mut().unwrap(), path / 2)
            }
        }
        fn bubble_down<K: Ord>(node: &mut Node<K>) {
            let mut biggest_dir = 2;
            let mut biggest = &node.key;
            for dir in [0, 1] {
                if let Some(child) = &node.children[dir] {
                    if child.key > *biggest {
                        biggest_dir = dir;
                        biggest = &child.key;
                    }
                }
            }
            if biggest_dir != 2 {
                let child = node.children[biggest_dir].as_mut().unwrap();
                std::mem::swap(&mut node.key, &mut child.key);
                bubble_down(child);
            }
        }
        let node = self.root.as_mut()?;
        let ret = if self.size == 1 {
            self.size -= 1;
            Some(self.root.take().unwrap().key)
        } else {
            self.size -= 1;
            let path = binary_path_to(self.size);
            let mut ret = last_key(node, path);
            std::mem::swap(&mut ret, &mut node.key);
            bubble_down(node);
            Some(ret)
        };
        self.check();
        ret
    }
}

#[cfg(test)]
mod tests {
    use rand::{Rng, SeedableRng};

    #[test]
    fn test() {
        let mut heap = super::RecursiveHeap::new();
        for v in [4, 2, 1, 3, 5, 7, 9, 6] {
            heap.push(v);
        }
        while heap.pop().is_some() {}
    }

    #[test]
    fn big_test() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let mut heap = super::RecursiveHeap::new();
        let mut expected = Vec::new();

        for _ in 0..10000 {
            let x: u64 = rng.gen();
            heap.push(x);
            expected.push(x);
        }
        expected.sort();
        expected.reverse();

        let mut actual = Vec::new();
        while let Some(x) = heap.pop() {
            actual.push(x);
        }
        assert_eq!(actual, expected);
    }
}
