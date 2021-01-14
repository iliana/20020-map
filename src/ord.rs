use std::cmp::Ordering;
use std::fmt;

#[derive(Clone, Copy)]
pub struct OrdF64(pub f64);

impl fmt::Debug for OrdF64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for OrdF64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl PartialEq for OrdF64 {
    fn eq(&self, other: &OrdF64) -> bool {
        self.0.eq(&other.0)
    }
}

impl Eq for OrdF64 {}

impl PartialOrd for OrdF64 {
    fn partial_cmp(&self, other: &OrdF64) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for OrdF64 {
    fn cmp(&self, other: &OrdF64) -> Ordering {
        self.0.partial_cmp(&other.0).unwrap()
    }
}
