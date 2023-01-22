type Anchor<V> = Option<Box<ImplTreapNode<V>>>;

struct ImplTreapNode<V> {
    value: V,
    priority: u64,
    count: usize,
    children: [Anchor<V>; 2],
}

impl<V> ImplTreapNode<V> {
    fn new(value: V) -> Self {
        ImplTreapNode {
            value,
            priority: rand::random(),
            count: 1,
            children: [None, None],
        }
    }
}

pub struct ImplTreap<V> {
    root: Anchor<V>,
    size: usize,
}

impl<V> ImplTreap<V> {
    pub fn new() -> Self {
        ImplTreap {
            root: None,
            size: 0,
        }
    }

    #[cfg(test)]
    fn check(&self) {
        // returns the number of nodes in the subtree
        fn aux<V>(anchor: &Anchor<V>, parent_priority: Option<u64>) -> usize {
            match anchor {
                None => 0,
                Some(node) => {
                    // check heap invariant
                    if let Some(parent_priority) = parent_priority {
                        assert!(node.priority <= parent_priority);
                    }
                    // recurse
                    let mut count = 0;
                    count += aux(&node.children[0], Some(node.priority));
                    count += 1;
                    count += aux(&node.children[1], Some(node.priority));
                    assert_eq!(count, node.count);
                    count
                }
            }
        }
        aux(&self.root, None);
    }

    fn rotate(anchor: &mut Anchor<V>, dir: usize) {
        let mut old_parent = anchor.take().unwrap();
        let mut new_parent = old_parent.children[dir].take().unwrap();
        old_parent.count -= new_parent.count;
        assert!(new_parent.priority > old_parent.priority);
        let grand_child = new_parent.children[1 - dir].take();
        let grand_child_count = grand_child.as_ref().map_or(0, |node| node.count);
        new_parent.count -= grand_child_count;
        old_parent.children[dir] = grand_child;
        old_parent.count += grand_child_count;
        new_parent.count += old_parent.count;
        new_parent.children[1 - dir] = Some(old_parent);
        *anchor = Some(new_parent);
    }

    pub fn insert(&mut self, index: usize, value: V) {
        // returns true when rebalancing might be needed
        fn aux<V>(anchor: &mut Anchor<V>, mut index: usize, value: V) -> bool {
            let current_index = if let Some(node) = anchor {
                if let Some(child) = &node.children[0] {
                    child.count
                } else {
                    0
                }
            } else {
                0
            };
            if index == current_index {
                let mut new_node = Box::new(ImplTreapNode::new(value));
                if let Some(mut node) = anchor.take() {
                    new_node.children[0] = node.children[0].take();
                    new_node.children[1] = Some(node);
                }
                *anchor = Some(new_node);
                true
            } else {
                let dir = if index < current_index {
                    0
                } else {
                    // index > current_index
                    index -= current_index + 1;
                    1
                };
                let node = anchor.as_mut().unwrap();
                if !aux(&mut node.children[dir], index, value) {
                    node.count += 1;
                    return false;
                }
                node.count += 1;
                let child = node.children[dir].as_ref().unwrap();
                if child.priority > node.priority {
                    ImplTreap::rotate(anchor, dir);
                    true
                } else {
                    false
                }
            }
        }
        aux(&mut self.root, index, value);
        self.size += 1;
    }

    pub fn push(&mut self, value: V) {
        self.insert(self.size, value);
    }

    pub fn remove(&mut self, index: usize) -> V {
        fn leftmost<V>(mut node: &mut ImplTreapNode<V>) -> Box<ImplTreapNode<V>> {
            if node.children[0].as_ref().unwrap().children[0].is_some() {
                let ret = leftmost(node.children[0].as_mut().unwrap());
                node.count -= 1;
                ret
            } else {
                let mut ret = node.children[0].take().unwrap();
                assert!(ret.children[0].is_none());
                node.count -= 1;
                node.children[0] = ret.children[1].take();
                ret
            }
        }
        fn bubble_down<V>(mut anchor: &mut Anchor<V>) {
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
                ImplTreap::rotate(anchor, max_priority_dir);
                anchor = &mut anchor.as_mut().unwrap().children[1 - max_priority_dir];
            }
        }
        fn aux<V>(anchor: &mut Anchor<V>, mut index: usize) -> V {
            match anchor {
                None => unreachable!(),
                Some(node) => {
                    let current_index = if let Some(child) = &node.children[0] {
                        child.count
                    } else {
                        0
                    };
                    if index == current_index {
                        let mut node = anchor.take().unwrap();
                        let ret = node.value;
                        match (node.children[0].take(), node.children[1].take()) {
                            (None, None) => (),
                            (Some(child), None) | (None, Some(child)) => *anchor = Some(child),
                            (Some(left), Some(mut right)) => match right.children[0] {
                                None => {
                                    right.children[0] = Some(left);
                                    right.count = node.count - 1;
                                    *anchor = Some(right);
                                    bubble_down(anchor);
                                }
                                Some(_) => {
                                    let mut new_node = leftmost(&mut right);
                                    new_node.count = node.count - 1;
                                    new_node.children[0] = Some(left);
                                    new_node.children[1] = Some(right);
                                    *anchor = Some(new_node);
                                    bubble_down(anchor);
                                }
                            },
                        }
                        ret
                    } else {
                        let dir = if index < current_index {
                            0
                        } else {
                            // index > current_index
                            index -= current_index + 1;
                            1
                        };
                        let ret = aux(&mut node.children[dir], index);
                        node.count -= 1;
                        ret
                    }
                }
            }
        }
        assert!(index < self.size);
        let ret = aux(&mut self.root, index);
        self.size -= 1;
        ret
    }

    pub fn pop(&mut self) -> Option<V> {
        if self.size == 0 {
            None
        } else {
            Some(self.remove(self.size - 1))
        }
    }
}

impl<V: std::fmt::Display> ImplTreap<V> {
    pub fn print_vec(&self) {
        fn aux<V: std::fmt::Display>(anchor: &Anchor<V>, mut index: usize) -> usize {
            match anchor {
                None => index,
                Some(node) => {
                    index = aux(&node.children[0], index);
                    println!("[{}]: {}", index, node.value);
                    index += 1;
                    aux(&node.children[1], index)
                }
            }
        }
        aux(&self.root, 0);
    }

    pub fn print_tree(&self) {
        fn aux<V: std::fmt::Display>(anchor: &Anchor<V>, depth: usize) {
            let prefix = "    ".repeat(depth);
            match anchor {
                None => println!("{}-", prefix),
                Some(node) => {
                    println!("{}- {} ({})", prefix, node.value, node.priority);
                    aux(&node.children[0], depth + 1);
                    aux(&node.children[1], depth + 1);
                }
            }
        }
        aux(&self.root, 0);
    }
}

// non-consuming iterator
enum ExplorationState {
    Unexplored,
    LeftYielded,
}

pub struct IterRef<'a, V> {
    stack: Vec<(ExplorationState, &'a ImplTreapNode<V>)>,
}

impl<'a, V> IterRef<'a, V> {
    fn new(treap: &'a ImplTreap<V>) -> Self {
        match &treap.root {
            None => IterRef { stack: vec![] },
            Some(node) => IterRef {
                stack: vec![(ExplorationState::Unexplored, node)],
            },
        }
    }
}

impl<'a, V> Iterator for IterRef<'a, V> {
    type Item = &'a V;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some((state, node)) = self.stack.pop() {
            match state {
                ExplorationState::Unexplored => {
                    self.stack.push((ExplorationState::LeftYielded, node));
                    if let Some(child) = &node.children[0] {
                        self.stack.push((ExplorationState::Unexplored, child));
                    }
                    self.next()
                }
                ExplorationState::LeftYielded => {
                    if let Some(child) = &node.children[1] {
                        self.stack.push((ExplorationState::Unexplored, child));
                    }
                    Some(&node.value)
                }
            }
        } else {
            None
        }
    }
}

impl<'a, V> IntoIterator for &'a ImplTreap<V> {
    type IntoIter = IterRef<'a, V>;
    type Item = &'a V;
    fn into_iter(self) -> Self::IntoIter {
        IterRef::new(self)
    }
}

impl<V> ImplTreap<V> {
    pub fn iter(&self) -> IterRef<V> {
        self.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use rand::{Rng, SeedableRng};

    #[test]
    fn test() {
        let mut treap = super::ImplTreap::new();
        treap.print_tree();
        treap.print_vec();

        for i in 1..10 {
            println!("Inserting {i}");
            treap.push(i);
            treap.print_tree();
            treap.print_vec();
            treap.check();
        }

        while let Some(x) = treap.pop() {
            println!("Removed {x:?}");
            treap.print_tree();
            treap.print_vec();
            treap.check();
        }
    }

    #[test]
    fn big_test() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let mut treap = super::ImplTreap::new();
        let mut expected = Vec::new();

        // add some
        for _ in 0..10000 {
            let x: u64 = rng.gen();
            println!("Pushing {x}");
            treap.push(x);
            treap.check();
            expected.push(x);
        }
        let actual: Vec<_> = treap.iter().copied().collect();
        assert_eq!(actual, expected);

        // remove some
        for _ in 0..1000 {
            let i = rng.gen_range(0..expected.len() - 1);
            println!("Removing at index {i}");
            treap.remove(i);
            treap.check();
            expected.remove(i);
        }
        let actual: Vec<_> = treap.iter().copied().collect();
        assert_eq!(actual, expected);
    }
}
