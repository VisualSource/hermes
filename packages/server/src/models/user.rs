use uuid::{uuid, Uuid};

#[derive(Debug,sqlx::FromRow,serde::Serialize)]
pub struct User { 
    id: Uuid, 
    username: String, 
    #[serde(skip_serializing)]
    psd_hash: String, 
    avatar: String, 
}

impl User {
    pub async fn insert_user(username: &str, psd_hash: &str, db: &SqlitePool) -> Result<Uuid,dyn Error> {

    }
    
    pub async fn find_by_uuid(id: Uuid, db: &SqlitePool) -> Result<User, dyn Error>{
        return sqlx::query_as!(User,"SELECT * FROM users WHERE id = ?",id).fetch_one(db).await;
    }
    pub async fn find_by_username(username: &str, db: &SqlitePool) -> Result<User,dyn Error> {
        return sqlx::query_as!(User,"SELECT * FROM users WHERE username = ?",username).fetch_one(db).await;
    }
}

#[derive(Debug,serde::Serialize)]
pub enum KeyType {
    Desktop,
    Mobile,
}

#[derive(Debug,sqlx::FromRow,serde::Serialize)]
pub struct UserPublicKey { 
    id: Uuid, 
    user: Uuid, 
    public_key: String, 
}

impl UserPublicKey {
    pub async fn find_all_by_user(user: Uuid, db: &SqlitePool){}
    pub async fn find_key_by_uuid(id: Uuid, db: &SqlitePool){}
}