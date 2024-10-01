use rocket::{http::CookieJar, serde::json::Json, State};

use crate::session;

#[derive(Responder)]
enum GroupResponse<T> {
    #[response(status = 200)]
    Ok(T),
    #[response(status = 400)]
    BadRequest(String),
    #[response(status = 401)]
    Unauthorized(String),
    #[response(status = 500)]
    InternalServerError(String),
}

#[derive(serde::Serialize)]
struct Group {
    pub id: String,
    pub name: String,
}

#[derive(serde::Deserialize)]
struct CreateGroup<'a> {
    pub name: &'a str,
}

#[derive(serde::Deserialize)]
struct ChangeMembers<'a> {
    id: &'a str,
    is_remove: bool,
}

#[get("/chat/groups/<count>/<offset>")]
pub async fn get(
    cookies: &CookieJar<'_>,
    database: &State<db::DBConnection>,
    count: u64,
    offset: u64,
) -> GroupResponse<Json<Vec<Group>>> {
    let session = match session::verify(cookies, database).await {
        Some(Some(session)) => session,
        Some(None) => return GroupResponse::Unauthorized(String::new()),
        None => return GroupResponse::InternalServerError(String::new()),
    };

    let db_groups = match database
        .get_groups_by_member(session.user, offset, count)
        .await
    {
        Ok(groups) => groups,
        Err(e) => {
            error!("Database: {e:?}");
            return GroupResponse::InternalServerError(String::new());
        }
    };

    let groups: Vec<Group> = db_groups
        .into_iter()
        .map(|group| Group {
            id: group.id.key().to_string(),
            name: group.name,
        })
        .collect();

    GroupResponse::Ok(Json(groups))
}

#[post("/chat/create", format = "json", data = "<group>")]
pub async fn create(
    cookies: &CookieJar<'_>,
    database: &State<db::DBConnection>,
    group: Json<CreateGroup<'_>>,
) -> GroupResponse<Json<Group>> {
    let session = match session::verify(cookies, database).await {
        Some(Some(session)) => session,
        Some(None) => return GroupResponse::Unauthorized(String::new()),
        None => return GroupResponse::InternalServerError(String::new()),
    };

    let created = chrono::Utc::now().timestamp_millis();
    let group = match database
        .create_group(db::CreateGroup {
            owner: session.user.clone(),
            name: group.name.to_string(),
            members: vec![session.user],
            created,
        })
        .await
    {
        Ok(Some(group)) => group,
        Ok(None) => {
            return GroupResponse::InternalServerError(String::new());
        }
        Err(e) => {
            error!("Database: {e:?}");
            return GroupResponse::InternalServerError(String::new());
        }
    };

    GroupResponse::Ok(Json(Group {
        id: group.id.key().to_string(),
        name: group.name,
    }))
}

#[post("/chat/<group>/member", format = "json", data = "<change>")]
pub async fn member(
    cookies: &CookieJar<'_>,
    database: &State<db::DBConnection>,
    group: &str,
    change: Json<ChangeMembers<'_>>,
) -> GroupResponse<()> {
    let session = match session::verify(cookies, database).await {
        Some(Some(session)) => session,
        Some(None) => return GroupResponse::Unauthorized(String::new()),
        None => return GroupResponse::InternalServerError(String::new()),
    };

    let group = match database.get_group(group).await {
        Ok(Some(group)) => group,
        Ok(None) => {
            return GroupResponse::InternalServerError(String::new());
        }
        Err(e) => {
            error!("Database: {e:?}");
            return GroupResponse::InternalServerError(String::new());
        }
    };

    if group.owner != session.user {
        return GroupResponse::Unauthorized(
            "Only the owner of a group is allowed to change members.".to_string(),
        );
    }

    let member = match database.get_user(change.id).await {
        Ok(Some(member)) => member,
        Ok(None) => {
            return GroupResponse::BadRequest("A user with that id doesn't exist.".to_string())
        }
        Err(e) => {
            error!("Database: {e:?}");
            return GroupResponse::InternalServerError(String::new());
        }
    };

    if change.is_remove {
        match database.remove_member_from_group(group.id, member.id).await {
            Ok(Some(_)) => (),
            Ok(None) => return GroupResponse::InternalServerError(String::new()),
            Err(e) => {
                error!("Database: {e:?}");
                return GroupResponse::InternalServerError(String::new());
            }
        }
    } else {
        match database.add_member_to_group(group.id, member.id).await {
            Ok(Some(_)) => (),
            Ok(None) => return GroupResponse::InternalServerError(String::new()),
            Err(e) => {
                error!("Database: {e:?}");
                return GroupResponse::InternalServerError(String::new());
            }
        }
    }

    GroupResponse::Ok(())
}
