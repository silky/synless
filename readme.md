# Synless

**This whole project is in a pre-alpha stage. Even the design
  documents are under construction at best. Synless does not yet
  exist.**

------

Synless is a hypothetical tree editor. It hopes to one day grow up to
be a real tree editor. It aims to:

- Make features and plugins much easier to write, by always knowing
  the exact structure of the document. (It can do this because it
  never has to try in vain to parse an incomplete and syntactically
  invalid program.)
- End formatting wars by delegating formatting choices to the same
  status as style files.
- Make it easy to design new structured document formats, and to
  provide an editor for them that can never create an invalid document.
- Eliminate the need for weird encoding details like escape sequences
  (I'm looking at you, quadruple backslashes).

Synless is not:

- A text editor.
- A tree editor built on top of a text editor. Seriously, there's no
  gap buffer. It's really just a tree.
- A language workbench. Synless will not help you define a language
  semantics, or perform static analysis.

Please read:

[Why Synless? And why "Synless"?](doc/why.md)

[The Synless Documentation](doc/readme.md)

[The Synless Design Documentation](doc/design.md) (for developers)

[An Incomplete Survey of Tree Editors](doc/survey.md)
