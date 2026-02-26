use uuid::{NoContext, Timestamp};

pub struct AuthorizationCode {
    pub id: uuid::Uuid,
    pub code: String,
    pub client_id: String,
    pub user_id: uuid::Uuid,
    pub redirect_uri: String,
    pub scope: String,
    pub created_at: time::OffsetDateTime,
    pub expires_at: time::OffsetDateTime,
    pub used: bool,

    pub code_challenge: String,
    pub code_challenge_method: String,
}

impl AuthorizationCode {
    pub fn new() -> Self {
        let ts = Timestamp::now(NoContext);

        Self {
            id: uuid::Uuid::new_v7(ts),
            code: todo!(),
            client_id: todo!(),
            user_id: todo!(),
            redirect_uri: todo!(),
            scope: todo!(),
            created_at: todo!(),
            expires_at: todo!(),
            used: todo!(),
            code_challenge: todo!(),
            code_challenge_method: todo!(),
        }
    }

    pub fn is_expired(&self) -> bool {
        time::UtcDateTime::now() > self.expires_at
    }

    pub fn is_valid(&self) -> bool {
        !self.used && !self.is_expired()
    }
}

pub struct AuthorizationRequest {
    response_type: String,
    client_id: String,
    redirect_uri: String,
    scope: Option<String>,
    state: Option<String>,
    code_challenge: String,
    code_challenge_method: String,
}
