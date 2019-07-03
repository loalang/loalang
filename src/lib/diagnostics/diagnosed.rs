use crate::*;

pub use Diagnosed::*;

#[derive(Debug)]
pub enum Diagnosed<T> {
    Just(T),
    Diagnosis(T, Vec<Diagnostic>),
    Failure(Vec<Diagnostic>),
}

macro_rules! diagnose {
    ($diagnosed: expr) => (
        match $diagnosed {
            Just(t) => t,
            Diagnosis(_, d) => return Failure(d),
            Failure(d) => return Failure(d),
        }
    );

    ($diagnostics: expr, $diagnosed: expr) => (
        match $diagnosed {
            Just(t) => t,
            Diagnosis(t, d) => {
                let dd = $diagnosed;
                dd.push(d);
                Diagnosis(t, dd)
            }
            Failure(d) => {
                let dd = $diagnosed;
                dd.push(d);
                Failure(dd)
            }
        }
    )
}

#[cfg(test)]
macro_rules! assert_diagnose {
    ($diagnosed: expr) => (
        match $diagnosed {
            Just(t) => t,
            Diagnosis(_, d) => panic!("{:?}", d),
            Failure(d) => panic!("{:?}", d),
        }
    );
}
