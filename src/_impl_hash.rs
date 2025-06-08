use crate::Subckt;
use core::hash::Hash;
use std::borrow::Borrow;

impl Hash for Subckt<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl PartialEq for Subckt<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Subckt<'_> {}

impl Borrow<str> for Subckt<'_> {
    fn borrow(&self) -> &str {
        &self.name
    }
}
