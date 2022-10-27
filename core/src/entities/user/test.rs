use crate::entities::location::SafePathBuf;
use crate::entities::user::User;
use crate::result::UserConvErr;

use rocket::request::{FromRequest, Outcome, Request};
use std::convert::TryFrom;

pub const FAKE_USER_EMAIL: &str = "some@email.com";

impl TryFrom<&SafePathBuf> for User {
    type Error = UserConvErr;

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
