use crate::entities::location::Location;
use crate::helpers::PathRefExt;
use crate::result::DoxErr;
use crate::use_cases::user::User;

use async_once_cell::OnceCell;
use jwks_client::keyset::KeyStore;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use std::convert::TryFrom;
use std::path::PathBuf;
use tracing::{debug, error};

impl TryFrom<&Location> for User {
    type Error = DoxErr;

    fn try_from(location: &Location) -> std::result::Result<Self, Self::Error> {
        let Location::FS(paths) = location;
        let path = paths.get(0).ok_or(DoxErr::EmptyLocation)?;
        let path: &PathBuf = path.as_ref();
        let parent_dir = path.parent().ok_or(DoxErr::InvalidPath)?;
        let parent_name = parent_dir.filename();
        let user_email = base64::decode(parent_name)?;
        let user_email = String::from_utf8(user_email)?;
        Ok(User::new(user_email))
    }
}

async fn key_store() -> &'static KeyStore {
    static INSTANCE: OnceCell<KeyStore> = OnceCell::new();

    INSTANCE
        .get_or_init(async {
            KeyStore::new_from("https://www.googleapis.com/oauth2/v3/certs".into())
                .await
                .expect("failed to create key store")
        })
        .await
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = DoxErr;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let token = req.headers().get("authorization").next();
        if token.is_none() {
            return Outcome::Failure((Status::Unauthorized, DoxErr::MissingToken));
        }
        let token = token.unwrap(); // can unwrap, because checked earlier
        let key_store = key_store().await;
        match key_store.verify(token) {
            Ok(jwt) => match jwt.payload().get_str("email") {
                Some(email) => {
                    debug!("name={:?}", email);
                    Outcome::Success(User::new(email))
                }
                None => {
                    error!("Invalid idToken, missing 'email' field");
                    Outcome::Failure((Status::BadRequest, DoxErr::InvalidIdToken))
                }
            },
            Err(e) => {
                error!("Could not verify token. Reason: {:?}", e);
                Outcome::Failure((Status::Unauthorized, DoxErr::TokenVerification))
            }
        }
    }
}
