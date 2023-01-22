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

    fn check(&self) {
        // returns the number of nodes in the subtree
        fn aux<V>(anchor: &Anchor<V>, parent_priority: Option<u64>) -> usize {
            match anchor {
                None => 0,
                Some(node) => {
                    // check heap invariant
                    if let Some(parent_priority) = parent_priority {
                        assert!(node.priority < parent_priority);
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

    /*
    pub fn remove(&mut self, index: usize) -> V {}

    pub fn pop(&mut self) -> V {
        self.remove(self.size - 1)
    }
    */
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

#[cfg(test)]
mod tests {
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
    }
}
