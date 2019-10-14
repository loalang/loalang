use crate::*;

pub use Diagnosed::*;

#[derive(Debug)]
pub enum Diagnosed<T> {
    Just(T),
    Diagnosis(T, Vec<Diagnostic>),
    Failure(Vec<Diagnostic>),
}

impl<T> Diagnosed<T> {
    pub fn maybe_diagnosis(t: T, diagnostics: Vec<Diagnostic>) -> Diagnosed<T> {
        if diagnostics.len() == 0 {
            Just(t)
        } else {
            Diagnosis(t, diagnostics)
        }
    }
}

impl Diagnosed<()> {
    pub fn flush(diagnostics: &mut Vec<Diagnostic>) -> Self {
        Diagnosed::maybe_diagnosis((), std::mem::replace(diagnostics, vec![]))
    }
}

/*
macro_rules! diagnose {
    ($diagnosed: expr) => {
        match $diagnosed {
            Just(t) => t,
            #[cfg(test)]
            Diagnosis(_, _) => panic!("Lost possibility for recovering!"),
            #[cfg(not(test))]
            Diagnosis(_, d) => return Failure(d),
            Failure(d) => return Failure(d),
        }
    };

    ($diagnostics: expr, $diagnosed: expr) => {
        match $diagnosed {
            Just(t) => t,
            Diagnosis(t, d) => {
                $diagnostics.extend(d);
                t
            }
            Failure(d) => {
                $diagnostics.extend(d);
                return Failure($diagnostics);
            }
        }
    };
}
*/

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

    pub fn report(self, reporter: &dyn Reporter) -> Option<T> {
        match self {
            Just(t) => Some(t),
            Diagnosis(t, d) => {
                reporter.report(&d);
                Some(t)
            }
            Failure(d) => {
                reporter.report(&d);
                None
            }
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

    pub fn diagnostics<F: FnOnce(Vec<Diagnostic>) -> ()>(self, f: F) -> Option<T> {
        match self {
            Just(t) => {
                f(vec![]);
                Some(t)
            }
            Diagnosis(t, v) => {
                f(v);
                Some(t)
            }
            Failure(v) => {
                f(v);
                None
            }
        }
    }

    #[cfg(test)]
    pub fn print(self) -> Self {
        match self {
            Just(_) => println!("Just []"),
            Diagnosis(_, ref d) => println!("Diagnosis {:?}", d),
            Failure(ref d) => println!("Failure {:?}", d),
        }
        self
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
