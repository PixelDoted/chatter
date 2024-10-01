#![allow(private_interfaces)]

pub mod group;
pub mod message;

use rocket::{
    http::{ContentType, CookieJar},
    response::Redirect,
    State,
};

use crate::session;

#[derive(Responder)]
enum PageResponse<'a> {
    #[response(status = 200)]
    Ok(&'a [u8], ContentType),
    #[response(status = 303)]
    UnknownGroup(Redirect),
    #[response(status = 303)]
    Unauthorized(Redirect),
    #[response(status = 500)]
    InternalError(&'a str),
}

#[get("/chat")]
pub async fn home_page(
    cookies: &CookieJar<'_>,
    database: &State<db::DBConnection>,
) -> PageResponse<'static> {
    match session::verify(cookies, database).await {
        Some(Some(_)) => (),
        Some(None) => {
            return PageResponse::Unauthorized(Redirect::to(uri!("/login")));
        }
        None => {
            return PageResponse::InternalError("Internal Database Error, try again later.");
        }
    }

    PageResponse::Ok(
        include_bytes!("../../content/chat/home.html"),
        ContentType::HTML,
    )
}

#[get("/chat/<group>")]
pub async fn group_page(
    cookies: &CookieJar<'_>,
    database: &State<db::DBConnection>,
    group: &str,
) -> PageResponse<'static> {
    let session = match session::verify(cookies, database).await {
        Some(Some(session)) => session,
        Some(None) => return PageResponse::Unauthorized(Redirect::to(uri!("/login"))),
        None => {
            return PageResponse::InternalError("Internal Database Error, try again later.");
        }
    };

    match database.get_group(group).await {
        Ok(Some(group)) => {
            if !group.members.contains(&session.user) {
                return PageResponse::Unauthorized(Redirect::to(uri!("/chat")));
            }
        }
        Ok(None) => return PageResponse::UnknownGroup(Redirect::to(uri!("/chat"))),
        Err(e) => {
            error!("Database: {e:?}");
            return PageResponse::InternalError("Internal Database Error");
        }
    }

    PageResponse::Ok(
        include_bytes!("../../content/chat/group.html"),
        ContentType::HTML,
    )
}
