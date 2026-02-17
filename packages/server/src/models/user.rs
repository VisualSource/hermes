use sqlx::SqlitePool;
use uuid::Uuid;
use super::DatabaseError;

#[derive(Debug, sqlx::FromRow, serde::Serialize)]
pub struct User {
    id: Uuid,
    username: String,
    #[serde(skip_serializing)]
    psd_hash: String,
    avatar: String,
}

impl User {
    pub async fn insert_user(username: &str, psd_hash: &str, db: &SqlitePool) -> Result<Uuid,DatabaseError> {
        let id = Uuid::new_v7();

        sqlx::query!("INSERT INTO users VALUES (?,?,?)",id,username,psd_hash).execute(db).await?;

        return Ok(id);
    }

    pub async fn find_by_uuid(id: &Uuid, db: &SqlitePool) -> Result<Option<User>,DatabaseError> {
        let result = sqlx::query_as!(User,"SELECT * FROM users WHERE id = ?",id).fetch_optional(db).await?;
        Ok(result)
    }
    pub async fn find_by_username(username: &str, db: &SqlitePool) -> Result<Option<User>,DatabaseError> {
        let result = sqlx::query_as!(User,"SELECT * FROM users WHERE username = ?",username).fetch_optional(db).await?;

        Ok(result)
    }
}

#[derive(Debug, serde::Serialize)]
pub enum KeyType {
    Desktop,
    Mobile,
}

#[derive(Debug, sqlx::FromRow, serde::Serialize)]
pub struct UserPublicKey {
    id: Uuid,
    user: Uuid,
    public_key: String,
}

impl UserPublicKey {
    pub async fn find_all_by_user_id<DB>(user_id: &Uuid, db: &sqlx::Pool<DB>) -> Result<Vec<UserPublicKey>,DatabaseError> where DB: impl sqlx::Database {
       let result = sqlx::query_as!(UserPublicKey,"SELECT * FROM keys WHERE user = ?", user_id).fetch_all(db).await?;

       Ok(result)
    }
    pub async fn find_key_by_uuid<DB>(id: &Uuid, db: &sqlx::Pool<DB>) -> Result<Option<UserPublicKey>,DatabaseError> where DB: impl sqlx::Database {
        let result = sqlx::query_as!(UserPublicKey,"SELECT * FROM keys WHERE id = ?",id).fetch_optional(db).await?;
        Ok(result)
    }

    pub async fn insert_key<DB>(key: &str, user: &Uuid, db: &sqlx::Pool<DB>) -> Result<Uuid,DatabaseError> where DB: impl sqlx::Database {
        let id = Uuid::new_v7();

        sqlx::query!("INSERT INTO keys VALUES (?,?,?)",id,user,key).execute(db).await?;

        Ok(id)
    }

    pub async fn remove_key_by_uuid<DB>(id: &Uuid, db: &sqlx::Pool<DB>) -> Result<(),DatabaseError> where DB: impl sqlx::Database {
        sqlx::query!("DELETE FROM keys WHERE id = >",id).execute(db).await?

        Ok(())
    }
}
