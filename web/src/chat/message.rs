use rocket::{http::CookieJar, serde::json::Json, State};

use crate::session;

#[derive(Responder)]
enum MessageResponse<T> {
    #[response(status = 200)]
    Ok(Json<T>),
    #[response(status = 400)]
    BadRequest(String),
    #[response(status = 401)]
    Unauthorized(String),
    #[response(status = 500)]
    InternalServerError(String),
}

#[derive(serde::Serialize)]
struct Message {
    pub id: String,
    pub group: String,
    pub author: String,
    pub text: String,
    pub created: i64,
}

#[derive(serde::Deserialize)]
struct CreateMessage<'a> {
    pub text: &'a str,
}

#[get("/chat/<group>/messages/<count>/<offset>")]
pub async fn get(
    cookies: &CookieJar<'_>,
    database: &State<db::DBConnection>,
    group: &str,
    count: u64,
    offset: u64,
) -> MessageResponse<Vec<Message>> {
    let session = match session::verify(cookies, database).await {
        Some(Some(session)) => session,
        Some(None) => return MessageResponse::Unauthorized(String::new()),
        None => return MessageResponse::InternalServerError(String::new()),
    };

    let group = match database.get_group(group).await {
        Ok(Some(group)) => group,
        Ok(None) => return MessageResponse::BadRequest("Group doesn't exist".to_string()),
        Err(e) => {
            error!("Database: {e:?}");
            return MessageResponse::InternalServerError(String::new());
        }
    };

    if !group.members.contains(&session.user) {
        return MessageResponse::Unauthorized("You are not in this group".to_string());
    }

    let db_messages = match database.get_messages(&group.id, count, offset).await {
        Ok(messages) => messages,
        Err(e) => {
            error!("Database: {e:?}");
            return MessageResponse::InternalServerError(String::new());
        }
    };

    let messages: Vec<Message> = db_messages
        .into_iter()
        .map(|msg| Message {
            id: msg.id.key().to_string(),
            group: msg.group.key().to_string(),
            author: msg.author.key().to_string(),
            text: msg.text,
            created: msg.created,
        })
        .collect();

    MessageResponse::Ok(Json(messages))
}

#[post("/chat/<group>/send", format = "json", data = "<message>")]
pub async fn send(
    cookies: &CookieJar<'_>,
    database: &State<db::DBConnection>,
    group: &str,
    message: Json<CreateMessage<'_>>,
) -> MessageResponse<Message> {
    let session = match session::verify(cookies, database).await {
        Some(Some(session)) => session,
        Some(None) => return MessageResponse::Unauthorized(String::new()),
        None => return MessageResponse::InternalServerError(String::new()),
    };

    let group = match database.get_group(group).await {
        Ok(Some(group)) => group,
        Ok(None) => return MessageResponse::BadRequest("Group doesn't exist.".to_string()),
        Err(e) => {
            error!("Database: {e:?}");
            return MessageResponse::InternalServerError(String::new());
        }
    };

    if !group.members.contains(&session.user) {
        return MessageResponse::Unauthorized("You are not in this group.".to_string());
    }

    let created = chrono::Utc::now().timestamp_millis();
    let message = match database
        .create_message(db::CreateMessage {
            group: group.id,
            author: session.user,
            text: message.text.to_string(),
            created,
        })
        .await
    {
        Ok(Some(message)) => message,
        Ok(None) => {
            return MessageResponse::InternalServerError(String::new());
        }
        Err(e) => {
            error!("Database: {e:?}");
            return MessageResponse::InternalServerError(String::new());
        }
    };

    MessageResponse::Ok(Json(Message {
        id: message.id.key().to_string(),
        group: message.group.key().to_string(),
        author: message.author.key().to_string(),
        text: message.text,
        created: message.created,
    }))
}
