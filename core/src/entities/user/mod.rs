#[cfg(not(test))]
mod real;
#[cfg(test)]
mod test;

use fake::{Dummy, Fake};

#[cfg(test)]
pub use test::FAKE_USER_EMAIL;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Hash, Dummy)]
pub struct User {
    pub email: String,
}

impl User {
    pub fn new<S: Into<String>>(email: S) -> Self {
        Self {
            email: email.into(),
        }
    }
}
