use rocket::fairing::Fairing;
use surrealdb::{
    engine::remote::ws::{Client, Ws, Wss},
    opt::auth::{Database, Jwt, Root},
    RecordId, Surreal,
};

#[derive(serde::Serialize)]
pub struct CreateUser {
    pub email: String,
    pub password: String,
}

#[derive(serde::Deserialize)]
pub struct User {
    pub id: surrealdb::RecordId,
    pub email: String,
    pub password: String,
}

#[derive(serde::Serialize)]
pub struct CreateSession {
    pub user: RecordId,
    pub created: i64,
}

#[derive(serde::Deserialize)]
pub struct Session {
    pub id: RecordId,
    pub user: RecordId,
    pub created: i64,
}

#[derive(serde::Serialize)]
pub struct CreateGroup {
    pub owner: RecordId,
    pub name: String,
    pub members: Vec<RecordId>,
    pub created: i64,
}

#[derive(serde::Deserialize)]
pub struct Group {
    pub id: RecordId,
    pub owner: RecordId,
    pub name: String,
    pub members: Vec<RecordId>,
    pub created: i64,
}

#[derive(serde::Serialize)]
pub struct CreateMessage {
    pub group: RecordId,
    pub author: RecordId,
    pub text: String,
    pub created: i64,
}

#[derive(serde::Deserialize)]
pub struct Message {
    pub id: RecordId,
    pub group: RecordId,
    pub author: RecordId,
    pub text: String,
    pub created: i64,
}

pub struct DBConnection {
    surreal: Surreal<Client>,
}

impl DBConnection {
    pub async fn new(addr: String) -> Result<Self, surrealdb::Error> {
        let db = Surreal::new::<Ws>(addr).await?;
        // FIXME: username and password fields
        db.signin(Root {
            username: "root",
            password: "root",
        })
        .await?;
        db.use_ns("testing").use_db("chatter").await?;

        Ok(Self { surreal: db })
    }

    /// Prepares the database for usage with `chatter`
    pub async fn prepare(&self) {
        self.surreal
            .query("DEFINE TABLE user")
            .query("DEFINE TABLE token")
            .query("DEFINE TABLE group")
            .query("DEFINE TABLE message")
            .await
            .expect("Failed to prepare database");
    }

    pub async fn get_user(&self, id: &str) -> Result<Option<User>, surrealdb::Error> {
        self.surreal.select(("user", id)).await
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, surrealdb::Error> {
        let mut res = self
            .surreal
            .query(format!("SELECT * FROM user WHERE email = \"{email}\""))
            .await?;

        res.take(0)
    }

    pub async fn create_user(
        &self,
        id: &str, // Username
        user: CreateUser,
    ) -> Result<Option<User>, surrealdb::Error> {
        self.surreal.create(("user", id)).content(user).await
    }

    pub async fn get_session(&self, id: &str) -> Result<Option<Session>, surrealdb::Error> {
        self.surreal.select(("session", id)).await
    }

    pub async fn create_session(
        &self,
        id: &str,
        session: CreateSession,
    ) -> Result<Option<Session>, surrealdb::Error> {
        self.surreal.create(("session", id)).content(session).await
    }

    pub async fn remove_session(&self, id: RecordId) -> Result<Option<Session>, surrealdb::Error> {
        self.surreal.delete(id).await
    }

    pub async fn get_messages(
        &self,
        group: &RecordId,
        count: u64,
        offset: u64,
    ) -> Result<Vec<Message>, surrealdb::Error> {
        let mut res = self
            .surreal
            .query(format!(
                "SELECT * FROM message WHERE group = {group} ORDER created DESC START {offset} LIMIT {count}"
            ))
            .await?;

        res.take(0)
    }

    pub async fn create_message(
        &self,
        message: CreateMessage,
    ) -> Result<Option<Message>, surrealdb::Error> {
        self.surreal.create("message").content(message).await
    }

    pub async fn get_group(&self, id: &str) -> Result<Option<Group>, surrealdb::Error> {
        self.surreal.select(("group", id)).await
    }

    pub async fn get_groups_by_member(
        &self,
        member: RecordId,
        offset: u64,
        count: u64,
    ) -> Result<Vec<Group>, surrealdb::Error> {
        let mut res = self
            .surreal
            .query(format!(
                "SELECT * FROM group WHERE members contains {member} START {offset} LIMIT {count}"
            ))
            .await?;

        res.take(0)
    }

    pub async fn create_group(
        &self,
        group: CreateGroup,
    ) -> Result<Option<Group>, surrealdb::Error> {
        self.surreal.create("group").content(group).await
    }

    pub async fn add_member_to_group(
        &self,
        group: RecordId,
        member: RecordId,
    ) -> Result<Option<Group>, surrealdb::Error> {
        let mut res = self
            .surreal
            .query(format!("UPDATE {group} SET members += {member}"))
            .await?;

        res.take(0)
    }

    pub async fn remove_member_from_group(
        &self,
        group: RecordId,
        member: RecordId,
    ) -> Result<Option<Group>, surrealdb::Error> {
        let mut res = self
            .surreal
            .query(format!("UPDATE {group} SET members -= {member}"))
            .await?;

        res.take(0)
    }
}

impl Fairing for DBConnection {
    fn info(&self) -> rocket::fairing::Info {
        rocket::fairing::Info {
            name: "database_connection",
            kind: rocket::fairing::Kind::Ignite,
        }
    }
}
