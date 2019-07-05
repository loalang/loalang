use crate::*;

pub use Diagnosed::*;

#[derive(Debug)]
pub enum Diagnosed<T> {
    Just(T),
    Diagnosis(T, Vec<Diagnostic>),
    Failure(Vec<Diagnostic>),
}

macro_rules! diagnose {
    ($diagnosed: expr) => {
        match $diagnosed {
            Just(t) => t,
            Diagnosis(_, d) => return Failure(d),
            Failure(d) => return Failure(d),
        }
    };

    ($diagnostics: expr, $diagnosed: expr) => {
        match $diagnosed {
            Just(t) => Just(t),
            Diagnosis(t, d) => {
                let mut dd = $diagnostics;
                dd.extend(d);
                Diagnosis(t, dd)
            }
            Failure(d) => {
                let mut dd = $diagnostics;
                dd.extend(d);
                Failure(dd)
            }
        }
    };
}

impl<T> Diagnosed<T> {
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Diagnosed<U> {
        match self {
            Just(t) => Just(f(t)),
            Diagnosis(t, d) => Diagnosis(f(t), d),
            Failure(d) => Failure(d),
        }
    }

    pub fn flat_map<U, F: FnOnce(T) -> Diagnosed<U>>(self, f: F) -> Diagnosed<U> {
        match self {
            Just(t) => f(t),
            Diagnosis(t, d) => diagnose!(d, f(t)),
            Failure(d) => Failure(d),
        }
    }

    #[cfg(test)]
    pub fn unwrap(self) -> T {
        match self {
            Just(t) => t,
            Diagnosis(_, d) | Failure(d) => panic!("Diagnostics: {:?}", d),
        }
    }
}

#[cfg(test)]
macro_rules! assert_diagnose {
    ($diagnosed: expr) => {
        match $diagnosed {
            Just(t) => t,
            Diagnosis(_, d) => panic!("{:?}", d),
            Failure(d) => panic!("{:?}", d),
        }
    };
}
