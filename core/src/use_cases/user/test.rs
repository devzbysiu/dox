use crate::entities::location::SafePathBuf;
use crate::result::DoxErr;
use crate::use_cases::user::User;

use rocket::request::{FromRequest, Outcome, Request};
use std::convert::TryFrom;

pub const FAKE_USER_EMAIL: &str = "some@email.com";

impl TryFrom<&SafePathBuf> for User {
    type Error = DoxErr;

    fn try_from(_location: &SafePathBuf) -> std::result::Result<Self, Self::Error> {
        Ok(User::new(FAKE_USER_EMAIL))
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = Box<dyn std::error::Error>;

    async fn from_request(_req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        Outcome::Success(User::new(FAKE_USER_EMAIL))
    }
}
