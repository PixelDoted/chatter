#![allow(private_interfaces)]

use rocket::{
    http::{Cookie, CookieJar, Status},
    serde::json::Json,
    State,
};
use zeroize::Zeroize;

#[derive(serde::Deserialize)]
struct LoginCredentials<'a> {
    email: &'a str,
    password: String,
}

#[derive(serde::Deserialize)]
struct RegisterCredentials<'a> {
    username: &'a str,
    email: &'a str,
    password: String,
}

#[post("/login", format = "json", data = "<credentials>")]
pub async fn login_req(
    mut credentials: Json<LoginCredentials<'_>>,
    cookies: &CookieJar<'_>,
    database: &State<db::DBConnection>,
) -> (Status, &'static str) {
    let user = match database.get_user_by_email(credentials.email).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return (Status::BadRequest, "There is no user with that email.");
        }
        Err(e) => {
            error!("Database: {e:?}");
            return (Status::InternalServerError, "Internal Database Error");
        }
    };

    // Hash and Zeroize the password
    if crypto::verify_password(credentials.password.as_bytes(), &user.password).is_err() {
        return (Status::BadRequest, "Incorrect password.");
    }
    credentials.password.zeroize();

    // Create Token
    let timestamp = chrono::Utc::now().timestamp_millis();
    let token = crypto::generate_token();
    match database
        .create_session(
            &token,
            db::CreateSession {
                user: user.id,
                created: timestamp,
            },
        )
        .await
    {
        Ok(Some(_)) => (),
        Ok(None) => {
            return (Status::InternalServerError, "Failed to create token.");
        }
        Err(e) => {
            error!("Database: {e:?}");
            return (Status::InternalServerError, "Internal Database Error");
        }
    }

    cookies.add(Cookie::new("session", token));
    (Status::Ok, "")
}

#[post("/register", format = "json", data = "<credentials>")]
pub async fn register_req(
    mut credentials: Json<RegisterCredentials<'_>>,
    cookies: &CookieJar<'_>,
    database: &State<db::DBConnection>,
) -> (Status, &'static str) {
    // Hash and Zeroize the password
    let hashed_password = crypto::hash_password(credentials.password.as_bytes());
    credentials.password.zeroize();

    // Make sure there is no user with the provided email
    match database.get_user_by_email(credentials.email).await {
        Ok(Some(_)) => {
            return (Status::Conflict, "A user with that email already exists.");
        }
        Ok(None) => (),
        Err(e) => {
            error!("Database: {e:?}");
            return (Status::InternalServerError, "Internal Database Error");
        }
    }

    // Create the user
    let user = match database
        .create_user(
            credentials.username,
            db::CreateUser {
                email: credentials.email.to_string(),
                password: hashed_password,
            },
        )
        .await
    {
        Ok(Some(user)) => user,
        Ok(None) => {
            return (Status::InternalServerError, "Failed to create user.");
        }
        Err(e) => {
            error!("Database: {e:?}");
            return (Status::InternalServerError, "Internal Database Error");
        }
    };

    // Create Token
    let timestamp = chrono::Utc::now().timestamp_millis();
    let token = crypto::generate_token();
    match database
        .create_session(
            &token,
            db::CreateSession {
                user: user.id,
                created: timestamp,
            },
        )
        .await
    {
        Ok(Some(_)) => (),
        Ok(None) => {
            return (Status::InternalServerError, "Failed to create token.");
        }
        Err(e) => {
            error!("Database: {e:?}");
            return (Status::InternalServerError, "Internal Database Error");
        }
    }

    cookies.add(Cookie::new("session", token));
    (Status::Ok, "")
}
