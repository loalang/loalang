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
            Diagnosis(t, d) => match f(t) {
                Just(tt) => Diagnosis(tt, d),
                Diagnosis(tt, mut dd) => {
                    dd.extend(d);
                    Diagnosis(tt, dd)
                }
                Failure(mut dd) => {
                    dd.extend(d);
                    Failure(dd)
                }
            },
            Failure(d) => Failure(d),
        }
    }

    pub fn report(&self, reporter: &dyn Reporter) {
        match self {
            Just(_) => (),
            Diagnosis(_, d) => reporter.report(d),
            Failure(d) => reporter.report(d),
        }
    }

    pub fn extract_flat_map<U, F: Fn(T) -> Diagnosed<U>>(input: Vec<T>, f: F) -> Diagnosed<Vec<U>> {
        let mut diagnostics = vec![];
        let mut o = vec![];
        for i in input {
            match f(i) {
                Just(u) => o.push(u),
                Diagnosis(u, d) => {
                    diagnostics.extend(d);
                    o.push(u);
                }
                Failure(d) => diagnostics.extend(d),
            }
        }
        if diagnostics.len() == 0 {
            return Just(o);
        }
        Diagnosis(o, diagnostics)
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
