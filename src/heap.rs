type Anchor<K> = Option<Box<HeapNode<K>>>;

struct HeapNode<K> {
    key: K,
    children: [Anchor<K>; 2],
}

impl<K> HeapNode<K> {
    fn new(key: K) -> Self {
        HeapNode {
            key,
            children: [None, None],
        }
    }
}

pub struct Heap<K> {
    root: Anchor<K>,
    size: usize,
}

impl<K> Heap<K> {
    pub fn new() -> Self {
        Heap {
            root: None,
            size: 0,
        }
    }
}

impl<K: std::fmt::Display> Heap<K> {
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

impl<K: Ord> Heap<K> {
    fn check(&self) {
        // returns the number of nodes
        fn aux<K: Ord>(anchor: &Anchor<K>, parent_key: Option<&K>) -> usize {
            match anchor {
                None => 0,
                Some(node) => {
                    if let Some(parent_key) = parent_key {
                        assert!(node.key < *parent_key);
                    }
                    if node.children[0].is_none() {
                        assert!(node.children[1].is_none());
                    }
                    1 + aux(&node.children[0], Some(&node.key))
                        + aux(&node.children[1], Some(&node.key))
                }
            }
        }
        let size = aux(&self.root, None);
        assert_eq!(self.size, size);
    }

    pub fn insert(&mut self, key: K) {
        fn aux<K: Ord>(anchor: &mut Anchor<K>, key: K, path: &[u8], depth: usize) {
            match anchor {
                None => {
                    *anchor = Some(Box::new(HeapNode::new(key)));
                }
                Some(node) => {
                    let dir = match path[depth] {
                        b'0' => 0,
                        b'1' => 1,
                        _ => unreachable!(),
                    };
                    aux(&mut node.children[dir], key, path, depth + 1);
                    // swap if needed
                    let child = node.children[dir].as_mut().unwrap();
                    if child.key > node.key {
                        std::mem::swap(&mut child.key, &mut node.key);
                    }
                }
            }
        }
        self.size += 1;
        let path = format!("{:b}", self.size);
        let path = path.as_bytes();
        aux(&mut self.root, key, path, 1);
        self.check();
    }

    pub fn peek(&self) -> Option<&K> {
        match &self.root {
            None => None,
            Some(node) => Some(&node.key),
        }
    }

    pub fn pop(&mut self) -> Option<K> {
        fn last_key<K>(node: &mut HeapNode<K>, path: &[u8], depth: usize) -> K {
            let dir = match path[depth] {
                b'0' => 0,
                b'1' => 1,
                _ => unreachable!(),
            };
            if depth == path.len() - 1 {
                assert!(node.children[dir].as_ref().unwrap().children[0].is_none());
                assert!(node.children[dir].as_ref().unwrap().children[1].is_none());
                node.children[dir].take().unwrap().key
            } else {
                last_key(node.children[dir].as_mut().unwrap(), path, depth + 1)
            }
        }
        fn bubble_down<K: Ord>(node: &mut HeapNode<K>) {
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
        match &mut self.root {
            None => None,
            Some(node) => {
                let ret = if self.size == 1 {
                    self.size -= 1;
                    Some(self.root.take().unwrap().key)
                } else {
                    let path = format!("{:b}", self.size);
                    let path = path.as_bytes();
                    self.size -= 1;
                    let mut ret = last_key(node, path, 1);
                    std::mem::swap(&mut ret, &mut node.key);
                    bubble_down(node);
                    Some(ret)
                };
                self.check();
                ret
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::{Rng, SeedableRng};

    #[test]
    fn test() {
        let mut heap = super::Heap::new();
        for v in [4, 2, 1, 3, 5, 7, 9, 6] {
            println!("Inserting {v}");
            heap.insert(v);
        }
        while let Some(x) = heap.pop() {
            println!("Popped {x}");
        }
    }

    #[test]
    fn big_test() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let mut heap = super::Heap::new();
        let mut expected = Vec::new();

        for _ in 0..10000 {
            let x: u64 = rng.gen();
            heap.insert(x);
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
