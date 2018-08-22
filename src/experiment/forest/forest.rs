use std::collections::HashMap;
use std::mem;
use uuid::Uuid;

use self::NodeContents::*;


// TODO: Note that it's up to the user to make sure that Trees are
// kept with the Forest they came from.

pub (super) type Id = Uuid;
fn fresh() -> Uuid {
    Uuid::new_v4()
}

pub struct Forest<Data, Leaf>{
    map: HashMap<Id, Node<Data, Leaf>>
}

struct Node<Data, Leaf> {
    parent: Option<Id>,
    contents: NodeContents<Data, Leaf>
}

enum NodeContents<Data, Leaf> {
    Leaf(Leaf),
    Branch(Data, Vec<Id>)
}

impl<D, L> Forest<D, L> { // I wish there was a `private impl`

    // Public //
    
    pub fn new() -> Forest<D, L> {
        Forest {
            map: HashMap::new()
        }
    }
    
    // Navigation //

    pub (super) fn parent(&self, id: Id) -> Option<Id> {
        self.get(id).parent
    }
    
    pub (super) fn children(&self, id: Id) -> &Vec<Id> {
        match &self.get(id).contents {
            Leaf(_) => panic!("Forest - leaf node has no children!"),
            Branch(_, children) => children
        }
    }

    pub (super) fn child(&self, id: Id, index: usize) -> Id {
        match self.children(id).get(index) {
            None => panic!("Forest - child index out of bounds. id={}, i={}", id, index),
            Some(child) => *child
        }
    }
    
    pub (super) fn root(&self, mut id: Id) -> Id {
        loop {
            match self.get(id).parent {
                None => return id,
                Some(parent) => {
                    id = parent;
                }
            }
        }
    }

    // Data Access //

    pub (super) fn is_leaf(&self, id: Id) -> bool {
        match &self.get(id).contents {
            Leaf(_)      => true,
            Branch(_, _) => false
        }
    }

    pub (super) fn data(&self, id: Id) -> &D {
        match &self.get(id).contents {
            Leaf(_) => panic!("Forest - leaf node has no data!"),
            Branch(data, _) => data
        }
    }

    pub (super) fn leaf(&self, id: Id) -> &L {
        match &self.get(id).contents {
            Leaf(leaf) => leaf,
            Branch(_, _) => panic!("Forest - branch node has no leaf!")
        }
    }

    // Data Mutation //

    pub (super) fn data_mut(&mut self, id: Id) -> &mut D {
        match &mut self.get_mut(id).contents {
            Leaf(_) => panic!("Forest - leaf node has no data!"),
            Branch(data, _) => data
        }
    }

    pub (super) fn leaf_mut(&mut self, id: Id) -> &mut L {
        match &mut self.get_mut(id).contents {
            Leaf(leaf) => leaf,
            Branch(_, _) => panic!("Forest - branch node has no leaf!")
        }
    }

    pub (super) fn children_mut(&mut self, id: Id) -> &mut Vec<Id> {
        match &mut self.get_mut(id).contents {
            Leaf(_) => panic!("Forest - leaf node has no children!"),
            Branch(_, children) => children
        }
    }

    // Forest Mutation //

    pub (super) fn create_branch(&mut self, data: D, children: Vec<Id>) -> Id {
        let id = fresh();
        let node = Node {
            parent: None,
            contents: Branch(data, children)
        };
        self.map.insert(id, node);
        id
    }

    pub (super) fn create_leaf(&mut self, leaf: L) -> Id {
        let id = fresh();
        let node = Node {
            parent: None,
            contents: Leaf(leaf)
        };
        self.map.insert(id, node);
        id
    }
    
    pub (super) fn replace_child(&mut self, parent: Id, index: usize, new_child: Id) -> Id {
        match self.children_mut(parent).get_mut(index) {
            None => panic!("Forest::replace - index out of bounds. id={}, i={}", parent, index),
            Some(child) => {
                let old_child = *child;
                *child = new_child;
                old_child
            }
        }
    }

    pub (super) fn insert_child(&mut self, parent: Id, index: usize, new_child: Id) {
        let children = self.children_mut(parent);
        if index > children.len() {
            panic!("Forest::insert - index out of bounds. id={}, i={}", parent, index);
        }
        children.insert(index, new_child);
    }

    pub (super) fn remove_child(&mut self, parent: Id, index: usize) -> Id {
        let children = self.children_mut(parent);
        if index >= children.len() {
            panic!("Forest::remove - index out of bounds. id={}, i={}", parent, index);
        }
        children.remove(index)
    }

    pub (super) fn delete_tree(&mut self, id: Id) {
        let node = self.remove(id);
        match node.contents {
            Leaf(leaf) => {
                mem::drop(leaf);
            }
            Branch(data, children) => {
                mem::drop(data);
                children.into_iter().for_each(|child| self.delete_tree(child));
            }
        };
    }

    // Private //

    fn get(&self, id: Id) -> &Node<D, L> {
        match self.map.get(&id) {
            Some(node) => node,
            None => panic!("Forest - id {} not found!", id)
        }
    }

    fn get_mut(&mut self, id: Id) -> &mut Node<D, L> {
        match self.map.get_mut(&id) {
            Some(node) => node,
            None => panic!("Forest - id {} not found!", id)
        }
    }

    fn remove(&mut self, id: Id) -> Node<D, L> {
        match self.map.remove(&id) {
            Some(node) => node,
            None => panic!("Forest - id {} not found!", id)
        }
    }
}
