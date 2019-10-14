use crate::*;

#[derive(Clone)]
pub struct Location {
    pub uri: URI,
    pub offset: usize,
    pub line: usize,
    pub character: usize,
}

impl Location {
    pub fn at_offset(source: &Arc<Source>, offset: usize) -> Location {
        let chars: Vec<char> = source.code.chars().collect();
        let code_before = &chars[..offset];
        let lines: Vec<_> = code_before.split(|c| *c == '\n').collect();
        Location {
            uri: source.uri.clone(),
            offset,
            line: lines.len(),
            character: lines.last().unwrap().len() + 1,
        }
    }

    pub fn at_position(source: &Arc<Source>, line: usize, character: usize) -> Location {
        let mut chars: Vec<char> = source.code.chars().collect();
        let mut lines: Vec<&mut [char]> = chars.split_mut(|c| *c == '\n').collect();
        let lines_before = &mut lines[..line];
        if lines_before.len() == 0 {
            return Location {
                uri: source.uri.clone(),
                offset: 0,
                line: 1,
                character: 1,
            };
        }
        let mut new_last_line: Vec<_> = lines_before[lines_before.len() - 1][..character - 1]
            .iter()
            .cloned()
            .collect();
        lines_before[lines_before.len() - 1] = new_last_line.as_mut_slice();

        let mut offset = 0;
        for line in lines_before.iter() {
            for _ in line.iter() {
                offset += 1;
            }
            offset += 1;
        }
        if offset > 0 {
            offset -= 1;
        }

        Location {
            uri: source.uri.clone(),
            offset,
            line,
            character,
        }
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}:{}", self.uri, self.line, self.character)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_location_in_source() {
        let source = Source::test("hello");
        let location = Location::at_offset(&source, 0);
        assert_eq!(location.uri, source.uri);
        assert_eq!(location.offset, 0);
        assert_eq!(location.line, 1);
        assert_eq!(location.character, 1);
    }

    #[test]
    fn location_in_source() {
        let source = Source::test("hello");
        let location = Location::at_offset(&source, 3);
        assert_eq!(location.uri, source.uri);
        assert_eq!(location.offset, 3);
        assert_eq!(location.line, 1);
        assert_eq!(location.character, 4);
    }

    #[test]
    fn multiline() {
        let source = Source::test("hello\nthere");
        let location = Location::at_offset(&source, 6);
        assert_eq!(location.uri, source.uri);
        assert_eq!(location.offset, 6);
        assert_eq!(location.line, 2);
        assert_eq!(location.character, 1);
    }

    #[test]
    fn last_location() {
        let source = Source::test("hello\nthere");
        let location = Location::at_offset(&source, 11);
        assert_eq!(location.uri, source.uri);
        assert_eq!(location.offset, 11);
        assert_eq!(location.line, 2);
        assert_eq!(location.character, 6);
    }
}
