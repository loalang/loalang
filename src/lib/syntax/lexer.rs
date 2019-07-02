use crate::syntax::*;
use crate::*;
use core::iter::{Enumerate, Peekable};
use std::str::Chars;

type CharStream<'a> = Peekable<Enumerate<Chars<'a>>>;

pub fn tokenize(source: Arc<Source>) -> Vec<Token> {
    let mut chars = source.code.chars().enumerate().peekable();
    let mut tokens = vec![];

    loop {
        match next_token(&source, &mut chars) {
            None => break,
            Some(token) => tokens.push(token),
        }
    }

    tokens.push(Token {
        kind: TokenKind::EOF,
        span: Span::at_range(&source, 0..0),
    });

    tokens
}

fn next_token(source: &Arc<Source>, stream: &mut CharStream) -> Option<Token> {
    let (offset, ch) = stream.next()?;
    let kind;
    let mut end_offset;

    match ch {
        ' ' => {
            end_offset = offset;
            let mut chars = vec![ch];
            loop {
                match stream.peek() {
                    Some((_, ' ')) => {
                        let (o, c) = stream.next().unwrap();
                        end_offset = o;
                        chars.push(c);
                    }
                    _ => break,
                }
            }
            kind = TokenKind::Whitespace(chars.iter().collect());
        }
        _ => panic!("Unknown input"),
    }

    Some(Token {
        kind,
        span: Span::at_range(source, offset..end_offset),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_source() {
        let tokens = tokenize(Source::test(""));

        assert_eq!(tokens.len(), 1);
        assert_matches!(tokens[0].kind, TokenKind::EOF);
    }

    #[test]
    fn only_whitespace() {
        let tokens = tokenize(Source::test("  "));

        assert_eq!(tokens.len(), 2);
        assert_matches!(tokens[0].kind, TokenKind::Whitespace(ref s) if s == "  ");
    }
}
