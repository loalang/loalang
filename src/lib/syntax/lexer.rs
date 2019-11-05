use crate::syntax::*;
use crate::*;
use core::iter::Enumerate;
use peekmore::{PeekMore, PeekMoreIterator};
use std::str::EncodeUtf16;

type CharStream<'a> = PeekMoreIterator<Enumerate<EncodeUtf16<'a>>>;

pub fn is_valid_symbol(string: &String) -> bool {
    let source = Source::new(SourceKind::Module, URI::Exact("tmp".into()), string.clone());
    let tokens = tokenize(source);

    tokens.len() == 2 && matches!(tokens[0].kind, TokenKind::SimpleSymbol(_))
}

pub fn is_valid_binary_selector(string: &String) -> bool {
    let source = Source::new(SourceKind::Module, URI::Exact("tmp".into()), string.clone());
    let mut tokens = tokenize(source);
    tokens.pop();

    use TokenKind::*;

    for token in tokens {
        if !matches!(
            token.kind,
            Plus | Slash | EqualSign | OpenAngle | CloseAngle
        ) {
            return false;
        }
    }
    true
}

pub fn is_valid_keyword_selector(string: &String, length: usize) -> bool {
    let source = Source::new(SourceKind::Module, URI::Exact("tmp".into()), string.clone());
    let tokens = tokenize(source);

    if tokens.len() != length * 2 + 1 {
        return false;
    }

    use TokenKind::*;

    for i in 0..length - 1 {
        let kw_index = i * 2;
        let colon_index = kw_index + 1;

        if !matches!(tokens[kw_index].kind, SimpleSymbol(_)) {
            return false;
        }

        if !matches!(tokens[colon_index].kind, Colon) {
            return false;
        }
    }

    return true;
}

pub fn tokenize(source: Arc<Source>) -> Vec<Token> {
    let mut chars = source.code.encode_utf16().enumerate().peekmore();
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

const SLASH: u16 = '/' as u16;
const NUL: u16 = '\0' as u16;
const SPACE: u16 = ' ' as u16;
const NEWLINE: u16 = '\n' as u16;
const CARRIAGE_RETURN: u16 = '\r' as u16;
const TAB: u16 = '\t' as u16;
const UNDERSCORE: u16 = '_' as u16;
const APOSTROPHE: u16 = '\'' as u16;
const PLUS: u16 = '+' as u16;
const COLON: u16 = ':' as u16;
const COMMA: u16 = ',' as u16;
const PERIOD: u16 = '.' as u16;
const DASH: u16 = '-' as u16;
const OPEN_ANGLE: u16 = '<' as u16;
const CLOSE_ANGLE: u16 = '>' as u16;
const OPEN_CURLY: u16 = '{' as u16;
const CLOSE_CURLY: u16 = '}' as u16;
const EQUAL_SIGN: u16 = '=' as u16;

fn next_token(source: &Arc<Source>, stream: &mut CharStream) -> Option<Token> {
    let (offset, ch) = stream.next()?;
    let kind;
    let mut end_offset = offset;

    let peek = stream.peek();
    let next_ch = peek.map(|(_, c)| *c).unwrap_or(NUL);

    match (ch, next_ch) {
        // Whitespace
        (s, _) if matches!(s, SPACE | NEWLINE | CARRIAGE_RETURN | TAB) => {
            let mut chars = vec![ch];
            loop {
                match stream.peek() {
                    Some((_, s)) if matches!(*s, SPACE | NEWLINE | CARRIAGE_RETURN | TAB) => {
                        let (o, c) = stream.next().unwrap();
                        end_offset = o;
                        chars.push(c);
                    }
                    _ => break,
                }
            }
            kind = TokenKind::Whitespace(characters_to_string(chars.into_iter()));
        }

        // LineComment
        (SLASH, SLASH) => {
            let (o, _) = stream.next().unwrap();
            end_offset = o;
            let mut chars = vec![];
            loop {
                match stream.peek() {
                    Some((_, NEWLINE)) | None => break,
                    Some((_, _)) => {
                        let (o, c) = stream.next().unwrap();
                        end_offset = o;
                        chars.push(c);
                    }
                }
            }
            kind = TokenKind::LineComment(characters_to_string(chars.into_iter()));
        }

        // SimpleInteger & SimpleFloat
        (n, _) if (n as u8 as char).is_numeric() => {
            let mut chars = vec![ch];
            let mut is_float = false;
            loop {
                match stream.peek() {
                    Some((_, s)) if (*s as u8 as char).is_numeric() || *s == UNDERSCORE => {
                        let (o, c) = stream.next().unwrap();
                        end_offset = o;
                        chars.push(c);
                    }
                    Some((_, PERIOD)) if !is_float => {
                        stream.move_next();
                        is_float = match stream.peek() {
                            Some((_, ns)) if (*ns as u8 as char).is_numeric() => true,
                            _ => false,
                        };
                        stream.reset_view();
                        if !is_float {
                            break;
                        } else {
                            let (o, c) = stream.next().unwrap();
                            end_offset = o;
                            chars.push(c);
                        }
                    }
                    _ => break,
                }
            }
            if is_float {
                kind = TokenKind::SimpleFloat(characters_to_string(chars.into_iter()));
            } else {
                kind = TokenKind::SimpleInteger(characters_to_string(chars.into_iter()));
            }
        }

        // SimpleSymbol
        (n, _) if (n as u8 as char).is_alphabetic() || n == UNDERSCORE => {
            let mut chars = vec![ch];
            loop {
                match stream.peek() {
                    Some((_, s)) if (*s as u8 as char).is_alphanumeric() || *s == UNDERSCORE => {
                        let (o, c) = stream.next().unwrap();
                        end_offset = o;
                        chars.push(c);
                    }
                    _ => break,
                }
            }
            loop {
                match stream.peek() {
                    Some((_, APOSTROPHE)) => {
                        let (o, c) = stream.next().unwrap();
                        end_offset = o;
                        chars.push(c);
                    }
                    _ => break,
                }
            }
            match characters_to_string(chars.into_iter()).as_str() {
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
        (PLUS, _) => kind = TokenKind::Plus,

        // Colon
        (COLON, _) => kind = TokenKind::Colon,

        // Comma
        (COMMA, _) => kind = TokenKind::Comma,

        // Period
        (PERIOD, _) => kind = TokenKind::Period,

        // Slash
        (SLASH, _) => kind = TokenKind::Slash,

        // Arrow
        (DASH, CLOSE_ANGLE) => {
            let (o, _) = stream.next().unwrap();
            end_offset = o;
            kind = TokenKind::Arrow;
        }

        // FatArrow
        (EQUAL_SIGN, CLOSE_ANGLE) => {
            let (o, _) = stream.next().unwrap();
            end_offset = o;
            kind = TokenKind::FatArrow;
        }

        // EqualSign
        (EQUAL_SIGN, _) => kind = TokenKind::EqualSign,

        // (Open/Close)Angle
        (OPEN_ANGLE, _) => kind = TokenKind::OpenAngle,
        (CLOSE_ANGLE, _) => kind = TokenKind::CloseAngle,

        // (Open/Close)Curly
        (OPEN_CURLY, _) => kind = TokenKind::OpenCurly,
        (CLOSE_CURLY, _) => kind = TokenKind::CloseCurly,

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
