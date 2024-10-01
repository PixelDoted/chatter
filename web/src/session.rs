use rocket::{http::CookieJar, State};

/// Returns `Some(true)` when the session is valid
/// Returns `Some(false)` when the session is invalid
/// Returns `None` when a database error occured
///
/// NOTE: when a database error occurs the error is printed to stdout
pub async fn verify(
    cookies: &CookieJar<'_>,
    database: &State<db::DBConnection>,
) -> Option<Option<db::Session>> {
    let Some(session) = cookies.get("session") else {
        return Some(None);
    };

    match database.get_session(session.value()).await {
        Ok(Some(token)) => {
            let now = chrono::Utc::now();
            let date = chrono::DateTime::from_timestamp_millis(token.created)
                .expect("Failed to convert timestamp into DateTime");

            let num_days_since = now.signed_duration_since(date).num_days();
            if num_days_since > 30 {
                if let Err(e) = database.remove_session(token.id).await {
                    error!("Database: {e:?}");
                }

                cookies.remove("session");
                return Some(None); // Invalid Session
            }

            // TODO: Check if the token validity will end soon and if so replace this token with a new token

            Some(Some(token)) // Valid Session
        }
        Ok(None) => Some(None), // Invalid Session
        Err(e) => {
            error!("[Session Token] Database: {e:?}");
            None // Database Error
        }
    }
}
