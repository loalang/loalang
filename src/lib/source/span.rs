use crate::*;

#[derive(Clone, Debug)]
pub struct Span {
    pub start: Location,
    pub end: Location,
}

impl Span {
    pub fn new(start: Location, end: Location) -> Span {
        Span { start, end }
    }

    pub fn over(start: Span, end: Span) -> Span {
        Span {
            start: start.start,
            end: end.end,
        }
    }

    pub fn all_of(source: &Arc<Source>) -> Span {
        Span::new(
            Location::at_offset(source, 0),
            Location::at_offset(source, source.len()),
        )
    }

    pub fn at_range(source: &Arc<Source>, range: std::ops::Range<usize>) -> Span {
        Span::new(
            Location::at_offset(source, range.start),
            Location::at_offset(source, range.end),
        )
    }

    pub fn at_end_of(source: &Arc<Source>) -> Span {
        let end = Location::at_end_of(source);
        Span::new(end.clone(), end)
    }

    pub fn through(&self, other: &Span) -> Span {
        Span::over(self.clone(), other.clone())
    }

    pub fn contains_location(&self, location: &Location) -> bool {
        if self.start.uri != location.uri {
            return false;
        }

        self.start.offset <= location.offset && self.end.offset >= location.offset
    }

    pub fn len(&self) -> usize {
        if self.end.offset < self.start.offset {
            0
        } else {
            self.end.offset - self.start.offset
        }
    }
}

impl PartialEq for Span {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start && self.end == other.end
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.start)
    }
}
