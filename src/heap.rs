pub struct Heap<K> {
    nodes: Vec<K>,
}

impl<K> Heap<K> {
    pub fn new() -> Self {
        Heap { nodes: Vec::new() }
    }
}

impl<K> Default for Heap<K> {
    fn default() -> Self {
        Heap::new()
    }
}

fn left(index: usize) -> usize {
    2 * index + 1
}
fn right(index: usize) -> usize {
    2 * index + 2
}
fn parent(index: usize) -> usize {
    (index - 1) / 2
}

impl<K: std::fmt::Display> Heap<K> {
    pub fn print(&self) {
        fn aux<K: std::fmt::Display>(heap: &Heap<K>, index: usize, depth: usize) {
            let prefix = " ".repeat(depth);
            if index < heap.nodes.len() {
                let key = &heap.nodes[index];
                println!("{}- {}", prefix, key);
                aux(heap, left(index), depth + 1);
                aux(heap, right(index), depth + 1);
            } else {
                println!("{}-", prefix);
            }
        }
        aux(self, 0, 0);
    }
}

impl<K: Ord> Heap<K> {
    fn check(&self) {
        fn aux<K: Ord>(heap: &Heap<K>, index: usize, parent_key: Option<&K>) {
            if index < heap.nodes.len() {
                let key = &heap.nodes[index];
                // ensure order is correct
                if let Some(parent_key) = parent_key {
                    assert!(*key < *parent_key);
                }
                aux(heap, left(index), Some(key));
                aux(heap, right(index), Some(key));
            }
        }
        aux(self, 0, None);
    }

    pub fn push(&mut self, key: K) {
        fn bubble_up<K: Ord>(heap: &mut Heap<K>, index: usize) {
            if index == 0 {
                return;
            }
            if heap.nodes[index] > heap.nodes[parent(index)] {
                heap.nodes.swap(index, parent(index));
                bubble_up(heap, parent(index))
            }
        }
        self.nodes.push(key);
        bubble_up(self, self.nodes.len() - 1);
        self.check();
    }

    pub fn peek(&self) -> Option<&K> {
        self.nodes.get(0)
    }

    pub fn pop(&mut self) -> Option<K> {
        fn bubble_down<K: Ord>(heap: &mut Heap<K>, index: usize) {
            let key = &heap.nodes[index];
            let mut biggest_key = key;
            let mut biggest_index = 0;
            if let Some(left) = heap.nodes.get(left(index)) {
                if *left > *biggest_key {
                    biggest_key = left;
                    biggest_index = 1;
                }
            }
            if let Some(right) = heap.nodes.get(right(index)) {
                if *right > *biggest_key {
                    // biggest_key = right;
                    biggest_index = 2;
                }
            }
            if biggest_index != 0 {
                let child = 2 * index + biggest_index;
                heap.nodes.swap(index, child);
                bubble_down(heap, child);
            }
        }
        if self.nodes.is_empty() {
            None
        } else {
            let ret = self.nodes.swap_remove(0);
            if !self.nodes.is_empty() {
                bubble_down(self, 0);
            }
            self.check();
            Some(ret)
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
            heap.push(v);
        }
        while let Some(_) = heap.pop() {}
    }

    #[test]
    fn big_test() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let mut heap = super::Heap::new();
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
