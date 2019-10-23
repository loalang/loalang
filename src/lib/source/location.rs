use crate::*;

#[derive(Clone, Debug)]
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

    pub fn at_end_of(source: &Arc<Source>) -> Location {
        Self::at_offset(source, source.code.len())
    }

    pub fn at_position(source: &Arc<Source>, line: usize, character: usize) -> Option<Location> {
        let mut chars: Vec<char> = source.code.chars().collect();
        let mut lines: Vec<&mut [char]> = chars.split_mut(|c| *c == '\n').collect();

        if lines.len() < line {
            warn!(
                "Tried to get position on line {} but the source had {} lines.",
                line,
                lines.len()
            );
            return None;
        }

        let lines_before = &mut lines[..line];
        if lines_before.len() == 0 {
            return Some(Location {
                uri: source.uri.clone(),
                offset: 0,
                line: 1,
                character: 1,
            });
        }
        let last_line_before = &mut lines_before[lines_before.len() - 1];
        if last_line_before.len() < character - 1 {
            warn!(
                "Tried to get position on character {} but the line had {} characters.",
                character,
                last_line_before.len()
            );
            return None;
        }
        let mut new_last_line: Vec<_> = last_line_before[..character - 1].iter().cloned().collect();
        *last_line_before = new_last_line.as_mut_slice();

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

        Some(Location {
            uri: source.uri.clone(),
            offset,
            line,
            character,
        })
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}:{}", self.uri, self.line, self.character)
    }
}

impl PartialEq for Location {
    fn eq(&self, other: &Self) -> bool {
        self.uri == other.uri && self.offset == other.offset
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
