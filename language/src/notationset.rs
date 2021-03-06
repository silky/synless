use std::collections::HashMap;

use pretty::Notation;
use crate::language::{ConstructName, Language, LanguageName};


pub struct NotationSet {
    name: LanguageName,
    notations: HashMap<ConstructName, Notation>
}

impl NotationSet {

    // TODO: validate against language
    pub fn new(language: &Language, notations: Vec<(ConstructName, Notation)>)
               -> NotationSet
    {
        let mut map = HashMap::new();
        for (construct, notation) in notations {
            map.insert(construct, notation);
        }
        NotationSet {
            name: language.name().to_string(),
            notations: map
        }
    }
}


#[cfg(test)]
use self::example::*;

#[cfg(test)]
mod example {
    use super::*;
    use pretty::*;
    use language::{Language, Construct, Arity};

    fn punct(s: &str) -> Notation {
        literal(s, Style::color(Color::Base0A))
    }

    fn word(s: &str) -> Notation {
        literal(s, Style::color(Color::Base0B))
    }

    fn txt() -> Notation {
        text(Style::new(Color::Base0D, Emph::underlined(), Shade::black(), false))
    }

    /// An example language for testing.
    pub fn example_language() -> (Language, NotationSet) {
        let mut language = Language::new("TestLang");

        let arity = Arity::Forest(vec!("Expr".to_string(),
                                       "Expr".to_string()),
                                  None);
        let construct = Construct::new("plus", "Expr", arity, 'p');
        language.add(construct);
        let plus_notation =
            child(0) + punct(" + ") + child(1)
            | flush(child(0)) + punct("+ ") + child(1);

        let notation = NotationSet::new(
            &language,
            vec!(("plus".to_string(), plus_notation)));
        (language, notation)
/*
        let syn = repeat(Repeat{
            empty:  empty(),
            lone:   star(),
            first:  star() + punct(", "),
            middle: star() + punct(", "),
            last:   star()
        }) | repeat(Repeat{
            empty:  empty(),
            lone:   star(),
            first:  flush(star() + punct(",")),
            middle: flush(star() + punct(",")),
            last:   star()
        });
        lang.add('a', Construct::new("args", Arity::extendable(0), syn));

        let syn = repeat(Repeat{
            empty:  punct("[]"),
            lone:   punct("[") + star() + punct("]"),
            first:  punct("[") + star() + punct(", "),
            middle: star() + punct(", "),
            last:   star() + punct("]")
        })| repeat(Repeat{
            empty:  punct("[]"),
            lone:   punct("[") + star() + punct("]"),
            first:  flush(star() + punct(",")),
            middle: flush(star() + punct(",")),
            last:   star() + punct("]")
        })| repeat(Repeat{
            empty:  punct("[]"),
            lone:   punct("[") + star() + punct("]"),
            first:  punct("[")
                + (star() + punct(", ") | flush(star() + punct(","))),
            middle: star() + punct(", ") | flush(star() + punct(",")),
            last:   star() + punct("]")
        });
        lang.add('l', Construct::new("list", Arity::extendable(0), syn));

        let syn =
            word("func ") + child(0)
            + punct("(") + child(1) + punct(") { ") + child(2) + punct(" }")
            | flush(word("func ") + child(0) + punct("(") + child(1) + punct(") {"))
            + flush(word("  ") + child(2))
            + punct("}")
            | flush(word("func ") + child(0) + punct("("))
            + flush(word("  ") + child(1) + punct(")"))
            + flush(punct("{"))
            + flush(word("  ") + child(2))
            + punct("}");
        lang.add('f', Construct::new("func", Arity::fixed(3), syn));

        let syn = if_empty_text(txt() + punct("·"), txt());
        lang.add('i', Construct::new("iden", Arity::text(), syn));

        let syn = punct("'") + txt() + punct("'");
        lang.add('s', Construct::new("strn", Arity::text(), syn));
         */
    }

/*
    pub fn example_tree<'l>(lang: &'l Language, tweak: bool) -> Tree<'l> {
        let con_func = lang.lookup_name("func");
        let con_id   = lang.lookup_name("iden");
        let con_str  = lang.lookup_name("strn");
        let con_arg  = lang.lookup_name("args");
        let con_plus = lang.lookup_name("plus");

        let foo = Tree::new_text(con_id, "foo");
        let abc = Tree::new_text(con_id, "abc");
        let def = Tree::new_text(con_id, "def");
        let args = Tree::new_forest(con_arg, vec!(abc, def));
        let abcdef1 = Tree::new_text(con_str, "abcdef");
        let abcdef2 = if tweak {
            Tree::new_text(con_str, "abc")
        } else {
            Tree::new_text(con_str, "abcdef")
        };
        let body = Tree::new_forest(con_plus, vec!(abcdef1, abcdef2));
        Tree::new_forest(con_func, vec!(foo, args, body))
    }
*/
}
