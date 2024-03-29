use crate::entities::location::SafePathBuf;
use crate::entities::user::User;
use crate::result::UserConvErr;

use async_once_cell::OnceCell;
use base64::engine::general_purpose::STANDARD as b64;
use base64::Engine;
use jwks_client::keyset::KeyStore;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use std::convert::TryFrom;

impl TryFrom<&SafePathBuf> for User {
    type Error = UserConvErr;

    fn try_from(path: &SafePathBuf) -> Result<Self, Self::Error> {
        let parent_name = path.parent_name();
        let user_email = b64.decode(parent_name)?;
        let user_email = String::from_utf8(user_email)?;
        Ok(User::new(user_email))
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = UserConvErr;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let token = req.headers().get("authorization").next();
        if token.is_none() {
            return Outcome::Failure((Status::Unauthorized, UserConvErr::MissingToken));
        }
        let token = token.unwrap(); // can unwrap, because checked earlier
        let key_store = key_store().await;
        match key_store.verify(token) {
            Ok(jwt) => {
                if let Some(email) = jwt.payload().get_str("email") {
                    Outcome::Success(User::new(email))
                } else {
                    Outcome::Failure((Status::BadRequest, UserConvErr::InvalidIdToken))
                }
            }
            Err(_) => Outcome::Failure((Status::Unauthorized, UserConvErr::TokenVerification)),
        }
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
