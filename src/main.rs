use std::rc::Rc;
use std::cell::RefCell;
use std::mem;
use std::cmp::max;
use std::ops::Not;

struct AVLNode<T: Ord> {
    value: T,
    height: usize,
    parent: Option<Rc<RefCell<AVLNode<T>>>>,
    left: Option<Rc<RefCell<AVLNode<T>>>>,
    right: Option<Rc<RefCell<AVLNode<T>>>>,
}

struct AVLTree<T: Ord> {
    root: Option<Rc<RefCell<AVLNode<T>>>>
}

#[derive(Clone, Copy)]
enum Side {
    Left,
    Right,
}

impl<T: Ord> AVLNode<T> {

    /// Returns a reference to the left or right child.
    fn child(&self, side: Side) -> &Option<Rc<RefCell<AVLNode<T>>>> {
        match side {
            Side::Left => &self.left,
            Side::Right => &self.right
        }
    }

    /// Returns a mutable reference to the left or right child.
    fn child_mut(&mut self, side: Side) -> &mut Option<Rc<RefCell<AVLNode<T>>>> {
        match side {
            Side::Left => &mut self.left,
            Side::Right => &mut self.right,
        }
    }

    /// Returns a mutable reference to the parent.
    fn parent_mut(&mut self) -> &mut Option<Rc<RefCell<AVLNode<T>>>> {
        &mut self.parent
    }

    fn is_left_child(&self) -> bool {
        match self.parent {
            None => false,
            Some(ref p) => {
                p.borrow().child(Side::Left).as_ref().unwrap().borrow().value == self.value
            }
        }
    }

    fn height(&self, side: Side) -> usize {
        self.child(side).as_ref().map_or(0, |n| n.borrow().height)
    }

    /// Recomputes the `height` field.
    fn update_height(&mut self) {
        self.height = 1 + max(self.height(Side::Left), self.height(Side::Right));
    }

    fn balance_factor(&self) -> i8 {
        let (left, right) = (self.height(Side::Left), self.height(Side::Right));
        if left < right {
            (right - left) as i8
        }
        else {
            -((left-right) as i8)
        }
    }

    /*
    fn find_replacement_node(&self) -> &AVLNode<T> {
        match self.child(Side::Left) {
            None => { // get replacement from right subtree
                let mut n = self.child(Side::Right).as_ref().unwrap();
                loop {
                    match n.left {
                        None => break n,
                        Some(ref left_node) => n = left_node,
                    }
                }
            },
            Some(ref child) => { // get replacement from left subtree
                let mut n = self.child(Side::Left).as_ref().unwrap();
                loop {
                    match n.right {
                        None => break n,
                        Some(ref right_node) => n = right_node,
                    }
                }
            }
        }
    }
    */

    fn replacement(&mut self) -> Option<Rc<RefCell<AVLNode<T>>>> {
        match self.child(Side::Left) {
            None => { // search for replacement in the right subtree
                let mut next = self.child_mut(Side::Right).clone();
                let mut curr = None;
                while let Some(node) = next {
                    curr = Some(Rc::clone(&node));
                    next = node.borrow_mut().child_mut(Side::Left).clone();
                }
                return curr;
            },
            Some(_) => {
                let mut next = self.child_mut(Side::Left).clone();
                let mut curr = None;
                while let Some(node) = next {
                    curr = Some(Rc::clone(&node));
                    next = node.borrow_mut().child_mut(Side::Right).clone();
                }
                return curr;
            }
        }
    }

    fn rotate(&mut self, side: Side) {
        let subtree = self.child_mut(!side).take().unwrap();
        *self.child_mut(!side) = subtree.borrow_mut().child_mut(side).take();
        self.update_height();
        mem::swap(self, &mut subtree.borrow_mut());
        mem::swap(self.parent_mut(), subtree.borrow_mut().parent_mut());
        *self.child_mut(side) = Some(subtree);
        self.update_height();
    }

}

fn rebalance<T: Ord>(r_node: Option<Rc<RefCell<AVLNode<T>>>>) {

    let mut next = r_node.clone();
    let mut b = 0;

    while let Some(node_ref) = next {
        let n = node_ref.borrow_mut();
        next = n.parent.clone();
        if next.is_none() {
            break;
        }
        let mut p = next.as_ref().unwrap().borrow_mut();
        if n.is_left_child() {
            if p.balance_factor() > 0 { // balance factor of p temporarily becomes +2. rotation is needed.
                let mut z = p.child(Side::Right).as_ref().unwrap().borrow_mut();
                b = z.balance_factor();
                if b < 0 {
                    z.rotate(Side::Right);
                    drop(z);
                    p.rotate(Side::Left);
                }
                else {
                    drop(z);
                    p.rotate(Side::Left);
                }
            }
            else {
                if p.balance_factor() == 0 { // p's height remains unchanged, no need to continue
                    p.update_height(); // not actually needed
                    break;
                }
                else { // P's height is decreased by one as n subtree was the tall one.
                    p.update_height();
                    continue;
                }
            }
        }
        else { // n is right child
            if p.balance_factor() < 0 { // balance factor of p temporarily becomes -2 -> rotation
                let mut z = p.child_mut(Side::Left).as_ref().unwrap().borrow_mut();
                let b = z.balance_factor();
                if b > 0 {
                    z.rotate(Side::Left);
                    drop(z);
                    p.rotate(Side::Right);
                }
                else {
                    drop(z);
                    p.rotate(Side::Right);
                }
            }
            else {
                if p.balance_factor() == 0 { // p's height is unchanged. no need to continue rebalancing.
                    p.update_height();
                    break;
                }
                else { // p's height is decreased by one. need to continue rebalancing with parent.
                    p.update_height();
                    continue;
                }
            }
        }

        // reached only after rotation
        if b == 0 { // the height at P hasn't changed, no need to continue further up
            break;
        }
    }
}

impl Not for Side {
    type Output = Side;

    fn not(self) -> Self::Output {
        match self {
            Side::Left => Side::Right,
            Side::Right => Side::Left,
        }
    }
}

fn main() {
    println!("Hello");
}


impl<T: Ord> AVLTree<T> {

    fn remove(&mut self, value: T) -> bool {
        let node_opt = Rc::clone(&self.root);
        loop {
            match node_opt {
                None => false,
                Some(node) => {
                    match value.cmp(&node.borrow().value) {
                        Ordering::Equal => break,
                        Ordering::Greater => node_opt = node.borrow().left,
                        Ordering::Less => node_opt = node.borrow().right
                    }
                }
            }
        }


        let n = node_opt.clone().unwrap().borrow_mut();
        if n.is_leaf() {
            let p = n.parent_mut();
            let was_only_child = n.is_only_child();
            p.remove_child(r);
            if was_only_child {
                p.update_height();
                p.rebalance();
            }
            //if n.is_left_child() {
            //    p.left = None;
            //}
            //else {
            //    p.right = None;
            //}
        }
        else { // n is not leaf. we need replacement.
            let r = n.replacement().clone().unwrap().borrow_mut();
            n.value = r.value;
            let p = r.parent_mut();
            let was_only_child = r.is_only_child();
            p.remove_child(r);
            if was_only_child {
                p.update_height();
                p.rebalance();
            }
        }

        return true;
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replacement_node() {
        /*
        let mut root = AVLNode {
            value: 1,
            height: 2,
            parent: None,
            right: None,
            left: None
        };
        let mut level1_node1 = AVLNode {
            value: 2,
            height: 1,
            parent: None,
            right: None,
            left: None
        };
        let mut level1_node2 = AVLNode {
            value: 3,
            height: 1,
            parent: None,
            right: None,
            left: None
        };
        let mut level2_node1 = AVLNode {
            value: 4,
            height: 0,
            parent: None,
            right: None,
            left: None
        };
        let mut level2_node2 = AVLNode {
            value: 5,
            height: 0,
            parent: None,
            right: None,
            left: None
        };
        let mut level2_node3 = AVLNode {
            value: 6,
            height: 0,
            parent: None,
            right: None,
            left: None
        };
        let mut level2_node4 = AVLNode {
            value: 7,
            height: 0,
            parent: None,
            right: None,
            left: None
        };

        level1_node1.left = Some(Box::new(level2_node1));
        level1_node1.right = Some(Box::new(level2_node2));
        level1_node2.left = Some(Box::new(level2_node3));
        level1_node2.right = Some(Box::new(level2_node4));
        root.left = Some(Box::new(level1_node1));
        root.right = Some(Box::new(level1_node2));

        //let node = root.left.as_ref().unwrap();
        let replacement_node = root.find_replacement_node();
        assert_eq!(replacement_node.value, 5);
        */

        let level3_node1 = Rc::new(RefCell::new(AVLNode {
            value: 4,
            height: 0,
            parent: None,
            right: None,
            left: None
        }));

        let level3_node2 = Rc::new(RefCell::new(AVLNode {
            value: 5,
            height: 0,
            parent: None,
            right: None,
            left: None
        }));
        let level3_node3 = Rc::new(RefCell::new(AVLNode {
            value: 6,
            height: 0,
            parent: None,
            right: None,
            left: None
        }));
        let level3_node4 = Rc::new(RefCell::new(AVLNode {
            value: 7,
            height: 0,
            parent: None,
            right: None,
            left: None
        }));
        let level2_node1 = Rc::new(RefCell::new(AVLNode {
            value: 2,
            height: 1,
            parent: None,
            right: Some(Rc::clone(&level3_node2)),
            left: Some(Rc::clone(&level3_node1))
        }));

        let level2_node2 = Rc::new(RefCell::new(AVLNode {
            value: 3,
            height: 1,
            parent: None,
            right: Some(Rc::clone(&level3_node4)),
            left: Some(Rc::clone(&level3_node3))
        }));
        level3_node1.borrow_mut().parent = Some(Rc::clone(&level2_node1));
        level3_node2.borrow_mut().parent = Some(Rc::clone(&level2_node1));
        level3_node3.borrow_mut().parent = Some(Rc::clone(&level2_node2));
        level3_node4.borrow_mut().parent = Some(Rc::clone(&level2_node2));
        let root = Rc::new(RefCell::new(AVLNode {
            value: 1,
            height: 2,
            parent: None,
            right: Some(Rc::clone(&level2_node2)),
            left: Some(Rc::clone(&level2_node1))
        }));
        level2_node1.borrow_mut().parent = Some(Rc::clone(&root));
        level2_node2.borrow_mut().parent = Some(Rc::clone(&root));

        let mut replacement = root.borrow_mut().replacement();
        assert_eq!(replacement.is_none(), false);
        assert_eq!(replacement.unwrap().borrow().value, 5);

        level2_node1.borrow_mut().left = None;
        level2_node1.borrow_mut().right = None;
        replacement = root.borrow_mut().replacement();
        assert_eq!(replacement.is_none(), false);
        assert_eq!(replacement.unwrap().borrow().value, 2);

        level2_node1.borrow_mut().left = Some(Rc::clone(&level3_node1));
        level2_node1.borrow_mut().right = Some(Rc::clone(&level3_node2));

        root.borrow_mut().rotate(Side::Right);
        assert_eq!(2, root.borrow().value);
        //println!("root value: {}", root.borrow().value);
    }
}
