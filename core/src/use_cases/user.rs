use crate::entities::location::Location;
use crate::helpers::PathRefExt;
use crate::result::DoxErr;

use jwks_client::keyset::KeyStore;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use std::convert::TryFrom;
use tracing::{debug, error};

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

impl TryFrom<&Location> for User {
    type Error = DoxErr;

    fn try_from(location: &Location) -> std::result::Result<Self, Self::Error> {
        let Location::FileSystem(paths) = location;
        let path = paths.get(0).ok_or(DoxErr::EmptyLocation)?;
        let parent_dir = path.parent().ok_or(DoxErr::InvalidPath)?;
        let parent_name = parent_dir.filename();
        let user_email = base64::decode(parent_name)?;
        let user_email = String::from_utf8(user_email)?;
        Ok(User::new(user_email))
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = AuthError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let token = req.headers().get("authorization").next();
        if token.is_none() {
            return Outcome::Failure((Status::Unauthorized, AuthError::MissingToken));
        }
        let token = token.unwrap(); // can unwrap, because checked earlier
        let key_set = KeyStore::new_from("https://www.googleapis.com/oauth2/v3/certs".into())
            .await
            .expect("failed to create key store");
        match key_set.verify(token) {
            Ok(jwt) => match jwt.payload().get_str("email") {
                Some(email) => {
                    debug!("name={:?}", email);
                    Outcome::Success(User::new(email))
                }
                None => {
                    error!("Invalid idToken, missing 'email' field");
                    Outcome::Failure((Status::BadRequest, AuthError::InvalidIdToken))
                }
            },
            Err(e) => {
                error!("Could not verify token. Reason: {:?}", e);
                Outcome::Failure((Status::Unauthorized, AuthError::TokenVerification))
            }
        }
    }
}

#[derive(Debug)]
pub enum AuthError {
    InvalidIdToken,
    TokenVerification,
    MissingToken,
}
