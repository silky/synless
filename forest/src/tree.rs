use std::mem;
use std::rc::Rc;
use std::cell::{RefCell, Ref, RefMut};
use std::ops::{Deref, DerefMut};
use std::thread;

use crate::forest::{Id, RawForest};


/// All [Trees](struct.Tree.html) belong to a Forest.
///
/// It is your responsibility to ensure that Trees are kept with the
/// Forest they came from. The methods on Trees will panic if you use
/// them on a different Forest.
pub struct Forest<D, L> {
    pub (super) lock: Rc<RefCell<RawForest<D, L>>>
}

impl<D, L> Clone for Forest<D, L> {
    fn clone(&self) -> Forest<D, L> {
        Forest {
            lock: self.lock.clone()
        }
    }
}

/// A mutable reference to a node in a tree, that owns the tree.
///
/// Every node is either a leaf or a branch.
/// A branch contains an ordered list of child nodes, and a data value
/// (the type parameter `Data` or `D`). A leaf contains only a leaf
/// value (the type parameter `Leaf` or `L`).
///
/// This value owns the entire tree. When it is dropped, the tree is deleted.
///
/// It also grants write access to the tree. Use [`as_ref`](#method.as_ref) to
/// obtain a shared reference with read-only access.
///
/// All write operations mutably borrow the _entire forest_. While a tree is
/// being mutated, or when some of its data is mutably borrowed (e.g. with
/// `leaf_mut()`), _no other tree in the forest can be accessed_.
pub struct Tree<D, L> {
    pub (super) forest: Forest<D, L>,
    pub (super) root: Id, // INVARIANT: This root remains valid despite edits
    pub (super) id: Id
}

#[derive(Clone, Copy)]
pub struct Bookmark {
    pub (super) id: Id
}

impl<D, L> Forest<D, L> {
    /// Construct a new forest.
    pub fn new() -> Forest<D, L> {
        Forest {
            lock: Rc::new(RefCell::new(RawForest::new()))
        }
    }

    /// Construct a new leaf.
    pub fn new_leaf(&self, leaf: L) -> Tree<D, L> {
        let leaf_id = self.write_lock().create_leaf(leaf);
        Tree::new(self, leaf_id)
    }

    /// Construct a new branch.
    pub fn new_branch(&self, data: D, children: Vec<Tree<D, L>>) -> Tree<D, L> {
        let child_ids = children.into_iter().map(|tree| {
            let id = tree.id;
            mem::forget(tree);
            id
        }).collect();
        let branch_id = self.write_lock().create_branch(data, child_ids);
        Tree::new(self, branch_id)
    }

    pub (super) fn write_lock(&self) -> RefMut<RawForest<D, L>> {
        self.lock.try_borrow_mut().expect("Failed to obtain write lock for forest.")
    }

    pub (super) fn read_lock(&self) -> Ref<RawForest<D, L>> {
        self.lock.try_borrow().expect("Failed to obtain read lock for forest.")
    }
}

impl<D, L> Tree<D, L> {

    /// Returns `true` if this is a leaf node, and `false` if this is
    /// a branch node.
    pub fn is_leaf(&self) -> bool {
        self.forest().is_leaf(self.id)
    }

    /// Obtain a shared reference to the data value at this node.
    ///
    /// # Panics
    ///
    /// Panics if this is not a branch node. (Leaves do not have data.)
    pub fn data(&self) -> ReadData<D, L> {
        ReadData {
            guard: self.forest(),
            id: self.id
        }
    }

    /// Obtain a shared reference to the leaf value at this node.
    ///
    /// # Panics
    ///
    /// Panics if this is a branch node.
    pub fn leaf(&self) -> ReadLeaf<D, L> {
        ReadLeaf {
            guard: self.forest(),
            id: self.id
        }
    }

    /// Returns the number of children this node has.
    ///
    /// # Panics
    ///
    /// Panics if this is a leaf node.
    pub fn num_children(&self) -> usize {
        self.forest().children(self.id).len()
    }

    /// Obtain a mutable reference to the data value at this node.
    ///
    /// # Panics
    ///
    /// Panics if this is not a branch node. (Leaves do not have data.)
    pub fn data_mut(&mut self) -> WriteData<D, L> {
        WriteData {
            guard: self.forest_mut(),
            id: self.id
        }
    }

    /// Obtain a mutable reference to the leaf value at this node.
    ///
    /// # Panics
    ///
    /// Panics if this is a branch node.
    pub fn leaf_mut(&mut self) -> WriteLeaf<D, L> {
        WriteLeaf {
            guard: self.forest_mut(),
            id: self.id
        }
    }

    /// Replace the `i`th child of this node with `tree`.
    /// Returns the original child.
    /// 
    /// # Panics
    ///
    /// Panics if this is a leaf node, or if `i` is out of bounds.
    pub fn replace_child(&mut self, i: usize, tree: Tree<D, L>) -> Tree<D, L> {
        let old_tree_id = self.forest_mut().replace_child(self.id, i, tree.id);
        mem::forget(tree);
        Tree::new(&self.forest, old_tree_id)
    }

    /// Insert `tree` as the `i`th child of this node.
    /// 
    /// # Panics
    ///
    /// Panics if this is a leaf node, or if `i` is out of bounds.
    pub fn insert_child(&mut self, i: usize, tree: Tree<D, L>) {
        let id = tree.id;
        mem::forget(tree);
        self.forest_mut().insert_child(self.id, i, id);
    }

    /// Remove and return the `i`th child of this node.
    /// 
    /// # Panics
    ///
    /// Panics if this is a leaf node, or if `i` is out of bounds.
    pub fn remove_child(&mut self, i: usize) -> Tree<D, L> {
        let old_tree_id = self.forest_mut().remove_child(self.id, i);
        Tree::new(&self.forest, old_tree_id)
    }

    /// Save a bookmark to return to later.
    pub fn bookmark(&mut self) -> Bookmark {
        Bookmark {
            id: self.id
        }
    }

    /// Jump to a previously saved bookmark, as long as that
    /// bookmark's node is present somewhere in this tree. This will
    /// work even if the Tree has been modified since the bookmark was
    /// created. However, it will return `None` if the bookmark's node
    /// has since been deleted, or if it is currently located in a
    /// different tree.
    pub fn goto_bookmark(&mut self, mark: Bookmark) -> bool {
        if self.forest().is_valid(mark.id) && self.forest().root(mark.id) == self.root {
            self.id = mark.id;
            true
        } else {
            false
        }
    }

    /// Returns `true` if this is the root of the tree, and `false` if
    /// it isn't (and thus this node has a parent).
    pub fn at_root(&self) -> bool {
        match self.forest().parent(self.id) {
            None => true,
            Some(_) => false
        }
    }

    /// Go to the root of this tree.
    pub fn goto_root(&mut self) {
        self.id = self.root;
    }

    /// Go to the parent of this node.
    ///
    /// # Panics
    ///
    /// Panics if this is the root of the tree, and there is no parent.
    pub fn goto_parent(&mut self) {
        let id = self.forest().parent(self.id).expect("Forest - root node has no parent!");
        self.id = id;
    }

    /// Go to the `i`th child of this branch node.
    ///
    /// # Panics
    ///
    /// Panics if this is a leaf node, or if `i` is out of bounds.
    pub fn goto_child(&mut self, i: usize) {
        let id = self.forest().child(self.id, i);
        self.id = id;
    }

    // Private //

    pub (super) fn new(forest: &Forest<D, L>, id: Id) -> Tree<D, L> {
        Tree {
            forest: forest.clone(),
            root: id,
            id: id
        }
    }

    fn forest(&self) -> Ref<RawForest<D, L>> {
        self.forest.read_lock()
    }

    fn forest_mut(&self) -> RefMut<RawForest<D, L>> {
        self.forest.write_lock()
    }
}

impl<D, L> Drop for Tree<D, L> {
    fn drop(&mut self) {
        if !thread::panicking() {
            // If it's already panicking, let's not worry too much about cleanup up the hashmap.
            self.forest.write_lock().delete_tree(self.id);
        }
    }
}


// Derefs //

/// Provides read access to a tree's data. Released on drop.
pub struct ReadData<'f, D, L> {
    pub (super) guard: Ref<'f, RawForest<D, L>>,
    pub (super) id: Id
}

/// Provides read access to a tree's leaf. Released on drop.
pub struct ReadLeaf<'f, D, L> {
    pub (super) guard: Ref<'f, RawForest<D, L>>,
    pub (super) id: Id
}

/// Provides write access to a tree's data. Released on drop.
pub struct WriteData<'f, D, L> {
    pub (super) guard: RefMut<'f, RawForest<D, L>>,
    pub (super) id: Id
}

/// Provides write access to a tree's leaf. Released on drop.
pub struct WriteLeaf<'f, D, L> {
    pub (super) guard: RefMut<'f, RawForest<D, L>>,
    pub (super) id: Id
}

impl<'f, D, L> Deref for ReadData<'f, D, L> {
    type Target = D;
    fn deref(&self) -> &D {
        self.guard.data(self.id)
    }
}

impl<'f, D, L> Deref for ReadLeaf<'f, D, L> {
    type Target = L;
    fn deref(&self) -> &L {
        self.guard.leaf(self.id)
    }
}

impl<'f, D, L> Deref for WriteData<'f, D, L> {
    type Target = D;
    fn deref(&self) -> &D {
        self.guard.data(self.id)
    }
}

impl<'f, D, L> DerefMut for WriteData<'f, D, L> {
    fn deref_mut(&mut self) -> &mut D {
        self.guard.data_mut(self.id)
    }
}

impl<'f, D, L> Deref for WriteLeaf<'f, D, L> {
    type Target = L;
    fn deref(&self) -> &L {
        self.guard.leaf(self.id)
    }
}

impl<'f, D, L> DerefMut for WriteLeaf<'f, D, L> {
    fn deref_mut(&mut self) -> &mut L {
        self.guard.leaf_mut(self.id)
    }
}
