mod chat;
mod session;
mod user;

use rocket::{
    http::{ContentType, CookieJar},
    response::Redirect,
};

#[macro_use]
extern crate rocket;

#[launch]
#[tokio::main]
async fn rocket() -> _ {
    let db_address =
        std::env::var("DB_ADDRESS").expect("`DB_ADDRESS` environment variable not provided");
    // let db_token_path =
    //     std::env::var("DB_TOKEN_PATH").expect("`DB_TOKEN` environment variable not provided");
    // let db_token = std::fs::read_to_string(db_token_path)
    //     .expect("couldn't read 'Json Web Token' from File at '{db_token_path}'");

    let db = db::DBConnection::new(db_address).await.unwrap();
    db.prepare().await;

    rocket::build()
        .mount(
            "/",
            routes![
                index,
                login_page,
                register_page,
                chat::home_page,
                chat::group_page,
                chat::group::get,
                chat::group::create,
                chat::group::member,
                chat::message::get,
                chat::message::send,
                style,
                user::login_req,
                user::register_req
            ],
        )
        .manage(db)
}

#[get("/")]
fn index(cookies: &CookieJar<'_>) -> Redirect {
    // TODO: Show index page

    if cookies.get("session").is_some() {
        Redirect::to(uri!("/chat"))
    } else {
        Redirect::to(uri!("/login"))
    }
}

#[get("/login")]
fn login_page() -> (ContentType, &'static [u8]) {
    (
        ContentType::HTML,
        include_bytes!("../../content/login.html"),
    )
}

#[get("/register")]
fn register_page() -> (ContentType, &'static [u8]) {
    (
        ContentType::HTML,
        include_bytes!("../../content/register.html"),
    )
}

#[get("/style.css")]
fn style() -> (ContentType, &'static [u8]) {
    (ContentType::CSS, include_bytes!("../../content/style.css"))
}
