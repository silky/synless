mod pretty_screen;
mod pretty_doc;
mod pretty_formatter;
mod example;
//mod locate_cursor;

//pub use self::pretty::*;
//pub use self::example::*;


#[cfg(test)]
mod tests {
    use super::pretty_formatter::PlainText;
    use super::pretty_doc::PrettyDocument;
    use super::example::{ExampleTree, make_example_tree};

    impl ExampleTree {
        fn write(&self, width: usize) -> String {
            let mut screen = PlainText::new(width);
            self.as_ref().pretty_print(&mut screen).unwrap();
            format!("{}", screen)
        }
    }

    #[test]
    fn test_lay_out() {
        let doc = make_example_tree();
        assert_eq!(doc.write(80),
                   "func foo(abc, def) { 'abcdef' + 'abcdef' }");
        assert_eq!(doc.write(42),
                   "func foo(abc, def) { 'abcdef' + 'abcdef' }");
        assert_eq!(doc.write(41),
                   "func foo(abc, def) { 'abcdef'
                     + 'abcdef' }");
        assert_eq!(doc.write(33),
                   "func foo(abc, def) { 'abcdef'
                     + 'abcdef' }");
        assert_eq!(doc.write(32),
                   "func foo(abc, def) {
  'abcdef' + 'abcdef'
}");
        assert_eq!(doc.write(21),
                   "func foo(abc, def) {
  'abcdef' + 'abcdef'
}");
        assert_eq!(doc.write(20),
                   "func foo(abc, def) {
  'abcdef'
  + 'abcdef'
}");
        assert_eq!(doc.write(19),
                   "func foo(abc,
         def) {
  'abcdef'
  + 'abcdef'
}");
        assert_eq!(doc.write(15),
                   "func foo(abc,
         def) {
  'abcdef'
  + 'abcdef'
}");
        assert_eq!(doc.write(14),
                   "func foo(
  abc, def)
{
  'abcdef'
  + 'abcdef'
}");
        assert_eq!(doc.write(12),
                   "func foo(
  abc, def)
{
  'abcdef'
  + 'abcdef'
}");
    }
}
