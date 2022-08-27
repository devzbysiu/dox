use crate::entities::location::Location;
use crate::result::DoxErr;
use crate::use_cases::user::User;

use rocket::request::{FromRequest, Outcome, Request};
use std::convert::TryFrom;

impl TryFrom<&Location> for User {
    type Error = DoxErr;

    fn try_from(_location: &Location) -> std::result::Result<Self, Self::Error> {
        Ok(User::new("some@email.com"))
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = Box<dyn std::error::Error>;

    async fn from_request(_req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        Outcome::Success(User::new("some@email.com"))
    }
}
