use crate::docs::*;
use loa::semantics::Analysis;
use loa::syntax::{DeclarationKind, Node};

struct Chars {
    pub subject: Vec<char>,
    pub cursor: isize,
    pub peek_cursor: usize,
}

impl Chars {
    pub fn new(subject: String) -> Chars {
        Chars {
            subject: subject.chars().collect(),
            cursor: -1,
            peek_cursor: 0,
        }
    }

    pub fn peek(&self) -> Option<&char> {
        self.subject.get(self.peek_cursor)
    }

    pub fn reset_view(&mut self) {
        self.peek_cursor = (self.cursor + 1) as usize
    }

    pub fn peek_next(&mut self) -> Option<&char> {
        self.peek_cursor += 1;
        self.peek()
    }

    pub fn next(&mut self) -> Option<char> {
        self.cursor += 1;
        if self.peek_cursor <= self.cursor as usize {
            self.reset_view();
        }
        self.subject.get(self.cursor as usize).cloned()
    }

    pub fn save(&self) -> (usize, isize) {
        (self.peek_cursor, self.cursor)
    }

    pub fn restore(&mut self, (p, c): (usize, isize)) {
        self.peek_cursor = p;
        self.cursor = c;
    }
}

pub struct Parser {
    reference_env: Option<(Analysis, Node)>,
}

impl Parser {
    pub fn new(reference_env: Option<(Analysis, Node)>) -> Parser {
        Parser { reference_env }
    }

    pub fn parse_markup(&self, source: String) -> Markup {
        let mut blocks = vec![];

        let mut chars = Chars::new(source);

        loop {
            match self.parse_block(&mut chars) {
                None => break,
                Some(block) => blocks.push(block),
            }
        }

        Markup { blocks }
    }

    fn parse_block(&self, chars: &mut Chars) -> Option<MarkupBlock> {
        chars.reset_view();

        // Move past leading whitespace
        while chars
            .peek()
            .cloned()
            .map(char::is_whitespace)
            .unwrap_or(false)
        {
            chars.next();
        }

        let mut elements = vec![];
        loop {
            if self.sees_end_of_block(chars) {
                break;
            } else {
                elements.push(self.parse_element(chars));
            }
        }
        if elements.is_empty() {
            None
        } else {
            Some(MarkupBlock::Paragraph { elements })
        }
    }

    fn parse_element(&self, chars: &mut Chars) -> MarkupElement {
        if self.sees_inline(chars) {
            self.parse_inline_element(chars)
                .unwrap_or_else(|| self.parse_text_element(chars))
        } else {
            self.parse_text_element(chars)
        }
    }

    fn parse_inline_element(&self, chars: &mut Chars) -> Option<MarkupElement> {
        if self.sees_bold(chars) {
            self.parse_bold_element(chars)
        } else if self.sees_italic(chars) {
            self.parse_italic_element(chars)
        } else if self.sees_link(chars) {
            self.parse_link_element(chars)
        } else {
            None
        }
    }

    fn parse_text_element(&self, chars: &mut Chars) -> MarkupElement {
        let mut value = String::new();
        value.push(chars.next().unwrap());
        while !self.sees_end_of_block(chars) && !self.sees_inline(chars) {
            value.push(chars.next().unwrap());
        }
        MarkupElement::Text { value }
    }

    fn parse_bold_element(&self, chars: &mut Chars) -> Option<MarkupElement> {
        let save_point = chars.save();

        let mut value = String::new();
        chars.next().unwrap();
        while !self.sees_end_of_block(chars) && !self.sees_bold(chars) {
            value.push(chars.next().unwrap());
        }
        if self.sees_bold(chars) {
            chars.next().unwrap();
            Some(MarkupElement::Bold { value })
        } else {
            chars.restore(save_point);
            None
        }
    }

    fn parse_italic_element(&self, chars: &mut Chars) -> Option<MarkupElement> {
        let save_point = chars.save();

        let mut value = String::new();
        chars.next().unwrap();
        while !self.sees_end_of_block(chars) && !self.sees_italic(chars) {
            value.push(chars.next().unwrap());
        }
        if self.sees_italic(chars) {
            chars.next().unwrap();
            Some(MarkupElement::Italic { value })
        } else {
            chars.restore(save_point);
            None
        }
    }

    fn parse_link_element(&self, chars: &mut Chars) -> Option<MarkupElement> {
        let value = self.parse_matching(chars, '[', ']')?;
        match self.parse_matching(chars, '(', ')') {
            None => Some(self.resolve_link(value.clone(), value)),
            Some(to) => Some(self.resolve_link(value, to)),
        }
    }

    fn resolve_link(&self, value: String, to: String) -> MarkupElement {
        self.find_declaration(to.clone())
            .unwrap_or(MarkupElement::Link { value, to })
    }

    fn find_declaration(&self, name: String) -> Option<MarkupElement> {
        let (ref analysis, ref anchor) = self.reference_env?;

        let declaration =
            analysis
                .navigator
                .find_declaration_above(anchor, to.clone(), DeclarationKind::Any)?;
    }

    fn parse_matching(&self, chars: &mut Chars, open: char, close: char) -> Option<String> {
        if chars.peek().map(|c| *c != open).unwrap_or(true) {
            return None;
        }

        let save_point = chars.save();

        let mut result = String::new();
        chars.next();

        while chars.peek().map(|c| *c != close).unwrap_or(false) {
            result.push(chars.next().unwrap());
        }

        if chars.peek().map(|c| *c == close).unwrap_or(false) {
            chars.next();
            Some(result)
        } else {
            chars.restore(save_point);
            return None;
        }
    }

    fn sees_end_of_block(&self, chars: &mut Chars) -> bool {
        chars.reset_view();
        match (chars.peek().cloned(), chars.peek_next()) {
            (Some('\n'), Some('\n')) | (Some('\n'), None) | (None, _) => true,
            _ => false,
        }
    }

    fn sees_inline(&self, chars: &mut Chars) -> bool {
        self.sees_bold(chars) || self.sees_italic(chars) || self.sees_link(chars)
    }

    fn sees_bold(&self, chars: &mut Chars) -> bool {
        chars.reset_view();
        match chars.peek() {
            Some('*') => true,
            _ => false,
        }
    }

    fn sees_italic(&self, chars: &mut Chars) -> bool {
        chars.reset_view();
        match chars.peek() {
            Some('_') => true,
            _ => false,
        }
    }

    fn sees_link(&self, chars: &mut Chars) -> bool {
        chars.reset_view();
        match chars.peek() {
            Some('[') => true,
            _ => false,
        }
    }
}

#[test]
fn parse_empty_string() {
    let parser = Parser::new(None);

    let examples: Vec<(&'static str, Markup)> = vec![
        ("", Markup { blocks: vec![] }),
        (
            "hello\n",
            Markup {
                blocks: vec![MarkupBlock::Paragraph {
                    elements: vec![MarkupElement::Text {
                        value: "hello".into(),
                    }],
                }],
            },
        ),
        (
            "line 1\nline 2\n\nline 4\n",
            Markup {
                blocks: vec![
                    MarkupBlock::Paragraph {
                        elements: vec![MarkupElement::Text {
                            value: "line 1\nline 2".into(),
                        }],
                    },
                    MarkupBlock::Paragraph {
                        elements: vec![MarkupElement::Text {
                            value: "line 4".into(),
                        }],
                    },
                ],
            },
        ),
        (
            "what *is* this?",
            Markup {
                blocks: vec![MarkupBlock::Paragraph {
                    elements: vec![
                        MarkupElement::Text {
                            value: "what ".into(),
                        },
                        MarkupElement::Bold { value: "is".into() },
                        MarkupElement::Text {
                            value: " this?".into(),
                        },
                    ],
                }],
            },
        ),
        (
            "a [link]",
            Markup {
                blocks: vec![MarkupBlock::Paragraph {
                    elements: vec![
                        MarkupElement::Text { value: "a ".into() },
                        MarkupElement::Link {
                            value: "link".into(),
                            to: "link".into(),
                        },
                    ],
                }],
            },
        ),
        (
            "a [link](https://example.com)",
            Markup {
                blocks: vec![MarkupBlock::Paragraph {
                    elements: vec![
                        MarkupElement::Text { value: "a ".into() },
                        MarkupElement::Link {
                            value: "link".into(),
                            to: "https://example.com".into(),
                        },
                    ],
                }],
            },
        ),
    ];

    for (source, expected) in examples {
        assert_eq!(parser.parse_markup(source.into()), expected);
    }
}
