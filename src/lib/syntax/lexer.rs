use crate::syntax::*;
use crate::*;
use core::iter::{Enumerate, Peekable};
use std::str::Chars;

type CharStream<'a> = Peekable<Enumerate<Chars<'a>>>;

pub fn is_valid_symbol(string: &String) -> bool {
    let source = Source::new(URI::Exact("tmp".into()), string.clone());
    let tokens = tokenize(source);

    tokens.len() == 1 && matches!(tokens[0].kind, TokenKind::SimpleSymbol(_))
}

pub fn is_valid_selector(string: &String) -> bool {
    is_valid_symbol(string) || is_valid_binary_selector(string) || is_valid_keyword_selector(string)
}

pub fn is_valid_binary_selector(string: &String) -> bool {
    let source = Source::new(URI::Exact("tmp".into()), string.clone());
    let tokens = tokenize(source);

    use TokenKind::*;

    tokens.len() == 1 && matches!(tokens[0].kind, Plus | Slash | EqualSign | OpenAngle | CloseAngle)
}

pub fn is_valid_keyword_selector(string: &String, length: usize) -> bool {
    let source = Source::new(URI::Exact("tmp".into()), string.clone());
    let tokens = tokenize(source);

    if tokens.len() != length * 2 {
        return false;
    }

    for i in 0..length-1 {
        let kw_index = i * 2;
        let colon_index = kw_index + 1;

        if !matches!(tokens[kw_index], SimpleSymbol(_)) {
            return false;
        }

        if !matches!(tokens[colon_index], Colon) {
            return false;
        }
    }

    return true;
}

pub fn tokenize(source: Arc<Source>) -> Vec<Token> {
    let mut chars = source.code.chars().enumerate().peekable();
    let mut end_offset = 0;
    let mut tokens = vec![];

    loop {
        match next_token(&source, &mut chars) {
            None => break,
            Some(token) => {
                end_offset = token.span.end.offset;
                tokens.push(token)
            }
        }
    }

    tokens.push(Token {
        kind: TokenKind::EOF,
        span: Span::at_range(&source, end_offset..end_offset),
    });

    tokens
}

fn next_token(source: &Arc<Source>, stream: &mut CharStream) -> Option<Token> {
    let (offset, ch) = stream.next()?;
    let kind;
    let mut end_offset = offset;

    let peek = stream.peek();
    let next_ch = peek.map(|(_, c)| *c).unwrap_or('\0');

    match (ch, next_ch) {
        // Whitespace
        (s, _) if matches!(s, ' ' | '\n' | '\r' | '\t') => {
            let mut chars = vec![ch];
            loop {
                match stream.peek() {
                    Some((_, s)) if matches!(s, ' ' | '\n' | '\r' | '\t') => {
                        let (o, c) = stream.next().unwrap();
                        end_offset = o;
                        chars.push(c);
                    }
                    _ => break,
                }
            }
            kind = TokenKind::Whitespace(chars.iter().collect());
        }

        // LineComment
        ('/', '/') => {
            let (o, _) = stream.next().unwrap();
            end_offset = o;
            let mut chars = vec![];
            loop {
                match stream.peek() {
                    Some((_, '\n')) | None => break,
                    Some((_, _)) => {
                        let (o, c) = stream.next().unwrap();
                        end_offset = o;
                        chars.push(c);
                    }
                }
            }
            kind = TokenKind::LineComment(chars.iter().collect());
        }

        // SimpleInteger
        (n, _) if n.is_numeric() => {
            let mut chars = vec![ch];
            loop {
                match stream.peek() {
                    Some((_, s)) if s.is_numeric() || *s == '_' => {
                        let (o, c) = stream.next().unwrap();
                        end_offset = o;
                        chars.push(c);
                    }
                    _ => break,
                }
            }
            kind = TokenKind::SimpleInteger(chars.iter().collect());
        }

        // SimpleSymbol
        (n, _) if n.is_alphabetic() || n == '_' => {
            let mut chars = vec![ch];
            loop {
                match stream.peek() {
                    Some((_, s)) if s.is_alphanumeric() || *s == '_' => {
                        let (o, c) = stream.next().unwrap();
                        end_offset = o;
                        chars.push(c);
                    }
                    _ => break,
                }
            }
            loop {
                match stream.peek() {
                    Some((_, '\'')) => {
                        let (o, c) = stream.next().unwrap();
                        end_offset = o;
                        chars.push(c);
                    }
                    _ => break,
                }
            }
            match chars.iter().collect::<String>().as_str() {
                "_" => kind = TokenKind::Underscore,

                "as" => kind = TokenKind::AsKeyword,
                "in" => kind = TokenKind::InKeyword,
                "is" => kind = TokenKind::IsKeyword,
                "out" => kind = TokenKind::OutKeyword,
                "inout" => kind = TokenKind::InoutKeyword,
                "class" => kind = TokenKind::ClassKeyword,
                "private" => kind = TokenKind::PrivateKeyword,
                "public" => kind = TokenKind::PublicKeyword,
                "namespace" => kind = TokenKind::NamespaceKeyword,
                "self" => kind = TokenKind::SelfKeyword,
                "import" => kind = TokenKind::ImportKeyword,
                "export" => kind = TokenKind::ExportKeyword,
                "partial" => kind = TokenKind::PartialKeyword,

                lexeme => kind = TokenKind::SimpleSymbol(lexeme.into()),
            }
        }

        // Plus
        ('+', _) => kind = TokenKind::Plus,

        // Colon
        (':', _) => kind = TokenKind::Colon,

        // Comma
        (',', _) => kind = TokenKind::Comma,

        // Period
        ('.', _) => kind = TokenKind::Period,

        // Slash
        ('/', _) => kind = TokenKind::Slash,

        // Arrow
        ('-', '>') => {
            let (o, _) = stream.next().unwrap();
            end_offset = o;
            kind = TokenKind::Arrow;
        }

        // FatArrow
        ('=', '>') => {
            let (o, _) = stream.next().unwrap();
            end_offset = o;
            kind = TokenKind::FatArrow;
        }

        // EqualSign
        ('=', _) => kind = TokenKind::EqualSign,

        // (Open/Close)Angle
        ('<', _) => kind = TokenKind::OpenAngle,
        ('>', _) => kind = TokenKind::CloseAngle,

        // (Open/Close)Curly
        ('{', _) => kind = TokenKind::OpenCurly,
        ('}', _) => kind = TokenKind::CloseCurly,

        // Unknown
        (c, _) => {
            kind = TokenKind::Unknown(c);
        }
    }

    Some(Token {
        kind,
        span: Span::at_range(source, offset..end_offset + 1),
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

    #[test]
    fn line_comment() {
        let tokens = tokenize(Source::test("  // line comment here\n  "));

        assert_eq!(tokens.len(), 4);
        assert_matches!(tokens[0].kind, TokenKind::Whitespace(ref s) if s == "  ");
        assert_eq!(tokens[0].span.start.offset, 0);
        assert_eq!(tokens[0].span.end.offset, 2);
        assert_matches!(tokens[1].kind, TokenKind::LineComment(ref s) if s == " line comment here");
        assert_eq!(tokens[1].span.start.offset, 2);
        assert_eq!(tokens[1].span.end.offset, 22);
        assert_matches!(tokens[2].kind, TokenKind::Whitespace(ref s) if s == "\n  ");
        assert_eq!(tokens[2].span.start.offset, 22);
        assert_eq!(tokens[2].span.end.offset, 25);
    }
}
