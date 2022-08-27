#[cfg(not(test))]
mod real;
#[cfg(test)]
mod test;

#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
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
