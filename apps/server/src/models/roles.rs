use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Role {
    id: String,
    server_id: Uuid,
    name: String,
    mask: i64,
}

/// a single record that links a user to a role
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct RoleMember {
    role_id: String,
    member_id: Uuid,
}

impl Role {
    pub async fn create(
        db: &SqlitePool,
        server_id: Uuid,
        name: String,
        mask: i64,
    ) -> Result<Role, sqlx::Error> {
        todo!()
    }
    pub async fn delete(db: &SqlitePool, id: String) -> Result<(), sqlx::Error> {
        todo!()
    }
    pub async fn patch(db: &SqlitePool, name: String) -> Result<(), sqlx::Error> {
        todo!()
    }
    pub async fn get(db: &SqlitePool) -> Result<Role, sqlx::Error> {
        todo!()
    }

    pub async fn get_all_by_server(db: &SqlitePool, server_id: Uuid) {
        todo!()
    }
}

impl RoleMember {
    pub async fn create(db: &SqlitePool, member_id: Uuid) -> Result<RoleMember, sqlx::Error> {
        todo!()
    }
    pub async fn delete(db: &SqlitePool, id: String) -> Result<(), sqlx::Error> {
        todo!()
    }
}
