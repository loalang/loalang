use crate::*;

#[derive(Clone)]
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

    pub fn at_range(source: &Arc<Source>, range: std::ops::Range<usize>) -> Span {
        Span::new(
            Location::at_offset(source, range.start),
            Location::at_offset(source, range.end),
        )
    }

    pub fn through(&self, other: &Span) -> Span {
        Span::over(self.clone(), other.clone())
    }
}
