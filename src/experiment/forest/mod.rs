mod forest;
mod tree;
mod subtree_ref;
mod subtree_mut;

///! Synless is a tree editor. Here are the trees.

pub use self::forest::Forest;
pub use self::tree::Tree;
pub use self::subtree_ref::SubtreeRef;
pub use self::subtree_mut::SubtreeMut;


#[cfg(test)]
mod test {
    use super::*;

    fn family(f: &mut Forest<&'static str, &'static str>) -> Tree {
        let leaves = vec!(Tree::new_leaf(f, "elder"),
                          Tree::new_leaf(f, "younger"));
        Tree::new_branch(f, "parent", leaves)
    }
    
    fn mirror(f: &mut Forest<u32, u32>, height: u32, id: u32) -> Tree {
        if height == 0 {
            Tree::new_leaf(f, id)
        } else {
            let mut children = vec!();
            for i in 0..height {
                children.push(mirror(f, i, id + 2_u32.pow(i)));
            }
            Tree::new_branch(f, id, children)
        }
    }

    impl<'a> SubtreeRef<'a> {
        fn sum(&self, f: &Forest<u32, u32>) -> u32 {
            if self.is_leaf(f) {
                *self.leaf(f)
            } else {
                let mut sum = *self.data(f);
                for child in self.children(f) {
                    sum += child.sum(f);
                }
                sum
            }
        }
    }

    #[test]
    fn test_leaves() {
        let mut f: Forest<(), u32> = Forest::new();
        // Begin with a leaf of 2
        let mut tree = Tree::new_leaf(&mut f, 2);
        assert!(tree.as_mut(&mut f).is_leaf(&f)); // check SubtreeMut
        assert_eq!(*tree.as_mut(&mut f).leaf(&f), 2);
        // Mutate it to be 3
        *tree.as_mut(&mut f).leaf_mut(&mut f) = 3;
        assert!(tree.as_ref(&f).is_leaf(&f)); // check SubtreeRef
        assert_eq!(*tree.as_ref(&f).leaf(&f), 3);
        tree.delete(&mut f);
    }

    #[test]
    fn test_data() {
        let mut f: Forest<u32, ()> = Forest::new();
        // Begin with data of 2
        let mut tree = Tree::new_branch(&mut f, 2, vec!());
        assert!(!tree.as_ref(&f).is_leaf(&f)); // check SubtreeRef
        assert_eq!(*tree.as_ref(&f).data(&f), 2);
        // Mutate it to be 3
        *tree.as_mut(&mut f).data_mut(&mut f) = 3;
        assert!(!tree.as_mut(&mut f).is_leaf(&f)); // check SubtreeMut
        assert_eq!(*tree.as_mut(&mut f).data(&f), 3);
        tree.delete(&mut f);
    }

    #[test]
    fn test_num_children() {
        let mut f: Forest<(), ()> = Forest::new();
        let leaves = vec!(Tree::new_leaf(&mut f, ()),
                          Tree::new_leaf(&mut f, ()),
                          Tree::new_leaf(&mut f, ()));
        let mut tree = Tree::new_branch(&mut f, (), leaves);
        assert_eq!(tree.as_ref(&f).num_children(&f), 3);
        assert_eq!(tree.as_mut(&mut f).num_children(&f), 3);
        tree.delete(&mut f);
    }

    #[test]
    fn test_navigation_ref() {
        let mut f: Forest<&'static str, &'static str> = Forest::new();
        let tree = family(&mut f);
        assert_eq!(*tree.as_ref(&f).child(&f, 0).leaf(&f), "elder");
        assert_eq!(*tree.as_ref(&f).child(&f, 1).leaf(&f), "younger");
        assert_eq!(*tree.as_ref(&f).child(&f, 0).parent(&f).unwrap().data(&f), "parent");
        assert!(tree.as_ref(&f).child(&f, 0).parent(&f).unwrap().parent(&f).is_none());
        let children: Vec<&'static str> = tree.as_ref(&f)
            .children(&f)
            .map(|child| *child.leaf(&f))
            .collect();
        assert_eq!(children, vec!("elder", "younger"));
        // Cleanup
        tree.delete(&mut f);
    }

    #[test]
    fn test_navigation_mut() {
        let mut f: Forest<&'static str, &'static str> = Forest::new();
        let mut tree = family(&mut f);
        {
            let mut cursor = tree.as_mut(&mut f);
            cursor.goto_child(&mut f, 1);
            assert_eq!(*cursor.leaf(&f), "younger");
            assert!(cursor.goto_parent(&mut f));
            assert!(!cursor.goto_parent(&mut f));
            assert_eq!(*cursor.data(&f), "parent");
            cursor.goto_child(&mut f, 0);
            assert_eq!(*cursor.leaf(&f), "elder");
        }
        // Cleanup
        tree.delete(&mut f);
    }

    #[test]
    fn test_bookmark_ref() {
        let mut f: Forest<&'static str, &'static str> = Forest::new();
        let mut tree = family(&mut f);
        let mut other_tree = Tree::new_leaf(&mut f, "stranger");
        let bookmark = tree.as_ref(&f).child(&f, 1).bookmark(&f);
        assert!(other_tree.as_ref(&f).lookup_bookmark(&f, bookmark).is_none());
        assert!(!other_tree.as_mut(&mut f).goto_bookmark(&mut f, bookmark));
        assert_eq!(*tree.as_ref(&f)
                   .lookup_bookmark(&f, bookmark).unwrap()
                   .leaf(&f),
                   "younger");
        {
            let mut cursor = tree.as_mut(&mut f);
            cursor.goto_child(&mut f, 0);
            assert!(cursor.goto_bookmark(&mut f, bookmark));
            assert_eq!(*cursor.leaf(&f), "younger");
        }
        // Cleanup
        tree.delete(&mut f);
        other_tree.delete(&mut f);
    }

    #[test]
    fn test_bookmark_mut() {
        let mut f: Forest<&'static str, &'static str> = Forest::new();
        let mut tree = family(&mut f);
        let mut other_tree = Tree::new_leaf(&mut f, "stranger");
        let bookmark = {
            let mut cursor = tree.as_mut(&mut f);
            cursor.goto_child(&mut f, 1);
            cursor.bookmark(&mut f)
        };
        assert!(other_tree.as_ref(&f).lookup_bookmark(&f, bookmark).is_none());
        assert!(!other_tree.as_mut(&mut f).goto_bookmark(&mut f, bookmark));
        assert_eq!(*tree.as_ref(&f)
                   .lookup_bookmark(&f, bookmark).unwrap()
                   .leaf(&f),
                   "younger");
        {
            let mut cursor = tree.as_mut(&mut f);
            cursor.goto_child(&mut f, 0);
            assert!(cursor.goto_bookmark(&mut f, bookmark));
            assert_eq!(*cursor.leaf(&f), "younger");
        }
        // Cleanup
        tree.delete(&mut f);
        other_tree.delete(&mut f);
    }

    #[test]
    fn test_bookmark_deleted() {
        let mut f: Forest<&'static str, &'static str> = Forest::new();
        let mut tree = family(&mut f);
        let bookmark = tree.as_ref(&f).child(&f, 1).bookmark(&f);
        let child = tree.as_mut(&mut f).remove_child(&mut f, 1);
        child.delete(&mut f);
        assert!(tree.as_ref(&f).lookup_bookmark(&f, bookmark).is_none());
        assert!(!tree.as_mut(&mut f).goto_bookmark(&mut f, bookmark));
        tree.delete(&mut f);
    }

    #[test]
    fn test_replace_child() {
        let mut f: Forest<&'static str, &'static str> = Forest::new();
        let mut tree = family(&mut f);
        let old_imposter = Tree::new_leaf(&mut f, "oldImposter");
        let young_imposter = Tree::new_leaf(&mut f, "youngImposter");
        let elder = tree.as_mut(&mut f).replace_child(&mut f, 0, old_imposter);
        let younger = tree.as_mut(&mut f).replace_child(&mut f, 1, young_imposter);
        assert_eq!(*elder.as_ref(&f).leaf(&f), "elder");
        assert_eq!(*younger.as_ref(&f).leaf(&f), "younger");
        assert_eq!(tree.as_ref(&f).num_children(&f), 2);
        assert_eq!(*tree.as_ref(&f).child(&f, 0).leaf(&f), "oldImposter");
        assert_eq!(*tree.as_ref(&f).child(&f, 1).leaf(&f), "youngImposter");
        tree.delete(&mut f);
        elder.delete(&mut f);
        younger.delete(&mut f);
    }

    #[test]
    fn test_remove_child() {
        let mut f: Forest<&'static str, &'static str> = Forest::new();
        let mut tree = family(&mut f);
        // Remove elder child from the family
        let elder = tree.as_mut(&mut f).remove_child(&mut f, 0);
        assert_eq!(*elder.as_ref(&f).leaf(&f), "elder");
        assert!(elder.as_ref(&f).parent(&f).is_none());
        assert_eq!(tree.as_ref(&f).num_children(&f), 1);
        assert_eq!(*tree.as_ref(&f).child(&f, 0).leaf(&f), "younger");
        assert_eq!(*tree.as_ref(&f).child(&f, 0).parent(&f).unwrap().data(&f), "parent");
        // Remove younger child from the family
        let younger = tree.as_mut(&mut f).remove_child(&mut f, 0);
        assert_eq!(*younger.as_ref(&f).leaf(&f), "younger");
        assert!(younger.as_ref(&f).parent(&f).is_none());
        assert_eq!(tree.as_ref(&f).num_children(&f), 0);
        // Cleanup
        tree.delete(&mut f);
        elder.delete(&mut f);
        younger.delete(&mut f);
    }

    #[test]
    fn test_insert_child() {
        let mut f: Forest<&'static str, &'static str> = Forest::new();
        let mut tree = family(&mut f);
        let malcolm = Tree::new_leaf(&mut f, "Malcolm");
        let reese = Tree::new_leaf(&mut f, "Reese");
        let dewey = Tree::new_leaf(&mut f, "Dewey");
        tree.as_mut(&mut f).insert_child(&mut f, 1, malcolm); // Malcolm is in the middle
        tree.as_mut(&mut f).insert_child(&mut f, 0, reese);
        tree.as_mut(&mut f).insert_child(&mut f, 4, dewey);
        let children: Vec<&'static str> = tree.as_ref(&f)
            .children(&f)
            .map(|child| *child.leaf(&f))
            .collect();
        assert_eq!(children, vec!("Reese", "elder", "Malcolm", "younger", "Dewey"));
        assert_eq!(*tree.as_ref(&f).child(&f, 0).parent(&f).unwrap().data(&f), "parent");
        assert_eq!(*tree.as_ref(&f).child(&f, 1).parent(&f).unwrap().data(&f), "parent");
        tree.delete(&mut f);
    }

    #[test]
    fn comprehensive_exam() {
        let mut f: Forest<u32, u32> = Forest::new();
        let mut tree = mirror(&mut f, 3, 0);
        let mut canada = Tree::new_branch(&mut f, 721, vec!());
        let mut mexico = Tree::new_leaf(&mut f, 3767);
        assert_eq!(f.tree_count(), 8+1+1);
        // tree:
        //       0
        //     / |  \
        //    1  2    4
        //       |   / \
        //       3  5   6
        //              |
        //              7

        // Test SubtreeRef
        let (mark2, mark4) = {
            // Data Access
            assert_eq!(tree.as_ref(&f).sum(&f), 28);
            assert_eq!(tree.as_ref(&f).num_children(&f), 3);

            // Navigation, Data Access
            let node5 = tree.as_ref(&f).child(&f, 2).child(&f, 0);
            assert!(node5.is_leaf(&f));
            assert_eq!(*node5.leaf(&f), 5);
            let node4 = node5.parent(&f).unwrap();
            assert_eq!(*node4.data(&f), 4);
            assert!(node5
                    .parent(&f).unwrap()
                    .parent(&f).unwrap()
                    .parent(&f)
                    .is_none());

            // Bookmarks: successful lookup
            let subtree = tree.as_ref(&f).child(&f, 1);
            let mark5 = node5.bookmark(&f);
            assert_eq!(*subtree
                       .lookup_bookmark(&f, mark5).unwrap()
                       .parent(&f).unwrap()
                       .data(&f), 4);
            let mark4 = node4.bookmark(&f);
            assert_eq!(*node5
                       .lookup_bookmark(&f, mark4).unwrap()
                       .parent(&f).unwrap()
                       .child(&f, 1)
                       .data(&f), 2);
            
            // Bookmarks: failing lookup
            assert!(canada.as_ref(&f).lookup_bookmark(&f, mark5).is_none());
            let mark_mexico = mexico.as_ref(&f).bookmark(&f);
            assert!(node4.lookup_bookmark(&f, mark_mexico).is_none());
            
            // Save some bookmarks for later testing
            let mark2 = tree.as_ref(&f).child(&f, 1).bookmark(&f);
            let mark4 = node4.bookmark(&f);
            (mark2, mark4)
        };
        
        // Test SubtreeMut
        {
            // To start
            let mut cursor = tree.as_mut(&mut f);
            //  tree: 0
            //      / |  \
            //     1  2*   4*
            //        |   / \
            //        3  5   6
            //               |
            //               7
            //  canada: 721
            //  mexico: 3767

            // Navigate
            assert!(!cursor.is_leaf(&f));
            cursor.goto_child(&mut f, 1);
            assert_eq!(*cursor.data(&f), 2);
            // Data Mutation
            *cursor.data_mut(&mut f) = 22;
            assert_eq!(*cursor.data(&f), 22);
            assert_eq!(cursor.num_children(&f), 1);
            // Navigate
            assert!(cursor.goto_parent(&mut f));
            let mark0 = cursor.bookmark(&mut f);
            assert!(!cursor.goto_parent(&mut f));
            
            // Cut
            let mut snip = cursor.remove_child(&mut f, 1);
            //  tree: 0+
            //       / \
            //      1    4*
            //          / \
            //         5   6
            //             |
            //             7
            //  snip: 2*
            //        |
            //        3
            //  canada: 721
            //  mexico: 3767
            assert_eq!(*snip.as_ref(&f).data(&f), 22);
            assert_eq!(cursor.as_ref(&f).sum(&f), 23);
            assert_eq!(f.tree_count(), 10);
            
            // Paste
            cursor.goto_child(&mut f, 1);
            cursor.insert_child(&mut f, 1, snip);
            cursor.insert_child(&mut f, 3, mexico);
            //  tree: 0+
            //       /  \
            //      1     4* _
            //          / | \  \
            //         5 22* 6 3767
            //            |  |
            //            3  7
            //  canada: 721
            
            // Leaf Mutation
            cursor.goto_child(&mut f, 3);
            assert!(cursor.is_leaf(&f));
            assert_eq!(*cursor.leaf(&f), 3767);
            let mark3767 = cursor.bookmark(&mut f);
            *cursor.leaf_mut(&mut f) = 376;
            assert_eq!(*cursor.leaf(&f), 376);
            assert!(cursor.goto_parent(&mut f));
            assert!(!cursor.is_leaf(&f));
            //  tree: 0+
            //       /  \
            //      1     4* _
            //          / | \  \
            //         5 22* 6 376+
            //            |  |
            //            3  7
            //  canada: 721

            // Replace
            snip = cursor.replace_child(&mut f, 1, canada);
            assert!(snip.as_ref(&f).parent(&f).is_none());
            cursor.goto_child(&mut f, 1);
            assert_eq!(*cursor.data(&f), 721);
            cursor.goto_parent(&mut f);
            assert_eq!(*cursor.data(&f), 4);
            // Further mucking
            mexico = cursor.remove_child(&mut f, 3);
            assert!(mexico.as_ref(&f).parent(&f).is_none());
            snip.as_mut(&mut f).insert_child(&mut f, 0, mexico);
            canada = snip;
            cursor.goto_child(&mut f, 2);
            //  tree: 0+
            //       / \
            //      1   4*
            //        / | \
            //       5 721 6
            //             |
            //             7
            // canada: 22*
            //        / \
            //       3  376+

            // Bookmarks after mutation
            assert!( ! cursor.goto_bookmark(&mut f, mark2));
            assert_eq!(*cursor.data(&f), 6);
            assert!(cursor.goto_bookmark(&mut f, mark4));
            assert_eq!(*cursor.data(&f), 4);
            assert_eq!(*canada.as_ref(&f)
                       .lookup_bookmark(&f, mark3767).unwrap()
                       .leaf(&f),
                       376);
            assert!( ! canada.as_mut(&mut f).goto_bookmark(&mut f, mark0));
        }
        // Some final bookmark checks
        assert!(tree.as_ref(&f).child(&f, 0).lookup_bookmark(&f, mark2).is_none());
        assert_eq!(tree.as_ref(&f)
                   .child(&f, 0)
                   .lookup_bookmark(&f, mark4).unwrap()
                   .sum(&f),
                   743);
        // Summation checks
        assert_eq!(tree.as_ref(&f).sum(&f), 744);
        assert_eq!(canada.as_ref(&f).sum(&f), 401);

        // Cleanup
        canada.delete(&mut f);
        tree.delete(&mut f);
        assert_eq!(f.tree_count(), 0);
    }

    // Error Testing //

    #[test]
    #[should_panic(expected="a tree was not recycled")]
    fn test_recycling_panic() {
        let mut f: Forest<(), ()> = Forest::new();
        Tree::new_leaf(&mut f, ());
        // Oops I dropped it.
    }
    
    #[test]
    #[should_panic(expected="leaf node has no children")]
    fn test_num_chilren_panic() {
        let mut f: Forest<(), ()> = Forest::new();
        let tree = Tree::new_leaf(&mut f, ());
        tree.as_ref(&f).num_children(&f);
    }

    #[test]
    #[should_panic(expected="leaf node has no data")]
    fn test_data_panic() {
        let mut f: Forest<(), ()> = Forest::new();
        let tree = Tree::new_leaf(&mut f, ());
        tree.as_ref(&f).data(&f);
    }

    #[test]
    #[should_panic(expected="branch node has no leaf")]
    fn test_leaf_panic() {
        let mut f: Forest<(), ()> = Forest::new();
        let mut tree = Tree::new_branch(&mut f, (), vec!());
        tree.as_mut(&mut f).leaf_mut(&mut f);
    }

    #[test]
    #[should_panic(expected="leaf node has no children")]
    fn test_navigation_panic_leaf_ref() {
        let mut f: Forest<(), ()> = Forest::new();
        let tree = Tree::new_leaf(&mut f, ());
        tree.as_ref(&f).child(&f, 0);
    }

    #[test]
    #[should_panic(expected="leaf node has no children")]
    fn test_navigation_panic_leaf_mut() {
        let mut f: Forest<(), ()> = Forest::new();
        let mut tree = Tree::new_leaf(&mut f, ());
        tree.as_mut(&mut f).goto_child(&mut f, 0);
    }

    #[test]
    #[should_panic(expected="child index out of bounds")]
    fn test_navigation_panic_oob_ref() {
        let mut f: Forest<(), ()> = Forest::new();
        let tree = Tree::new_branch(&mut f, (), vec!());
        tree.as_ref(&f).child(&f, 0);
    }

    #[test]
    #[should_panic(expected="child index out of bounds")]
    fn test_navigation_panic_oob_mut() {
        let mut f: Forest<(), ()> = Forest::new();
        let mut tree = Tree::new_branch(&mut f, (), vec!());
        tree.as_mut(&mut f).goto_child(&mut f, 0);
    }

    #[test]
    #[should_panic(expected="child index out of bounds")]
    fn test_insert_panic_oob() {
        let mut f: Forest<&'static str, &'static str> = Forest::new();
        let mut tree = family(&mut f);
        let leaf = Tree::new_leaf(&mut f, "");
        tree.as_mut(&mut f).insert_child(&mut f, 3, leaf);
    }

    #[test]
    #[should_panic(expected="child index out of bounds")]
    fn test_remove_panic_oob() {
        let mut f: Forest<&'static str, &'static str> = Forest::new();
        let mut tree = family(&mut f);
        tree.as_mut(&mut f).remove_child(&mut f, 2);
    }

    #[test]
    #[should_panic(expected="child index out of bounds")]
    fn test_replace_panic_oob() {
        let mut f: Forest<&'static str, &'static str> = Forest::new();
        let mut tree = family(&mut f);
        let leaf = Tree::new_leaf(&mut f, "");
        tree.as_mut(&mut f).replace_child(&mut f, 2, leaf);
    }
}