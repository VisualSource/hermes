use indexmap::{IndexMap, IndexSet};
use prost::bytes::Bytes;
use std::sync::RwLock;
use uuid::Uuid;

pub struct Conn {
    id: Uuid,
    user_id: Uuid,

    tx: tokio::sync::mpsc::Sender<Bytes>,
}

pub struct Registry {
    connections: IndexMap<Uuid, Conn>,
    /// users -> connections map
    users: IndexMap<Uuid, IndexSet<Uuid>>,
    /// channel -> active users map
    voice: IndexMap<Uuid, IndexSet<Uuid>>,
}

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("lock error")]
    Lock,
}

pub struct SessionRegistry(RwLock<Registry>);

impl SessionRegistry {
    pub fn register(
        &self,
        conn_id: Uuid,
        user_id: Uuid,
        tx: tokio::sync::mpsc::Sender<Bytes>,
    ) -> Result<(), SessionError> {
        let mut lock = self.0.write().expect("failed to lock");

        lock.connections.insert(
            conn_id,
            Conn {
                tx,
                id: conn_id,
                user_id,
            },
        );

        let user_entry = lock.users.entry(user_id);

        user_entry
            .and_modify(|map| {
                map.insert(conn_id);
            })
            .or_insert_with(|| {
                indexmap::indexset! {
                    conn_id
                }
            });

        Ok(())
    }
    pub fn unregister(&self, conn_id: &Uuid) -> Result<(), SessionError> {
        let mut lock = self.0.write().expect("failed to lock");

        if let Some(conn) = lock.connections.swap_remove(conn_id) {
            let entry = lock.users.entry(conn.user_id);
            entry.and_modify(|conns| {
                conns.swap_remove(&conn.id);
            });
        }

        Ok(())
    }
}

//
// users
//          -> session 1
//          -> session 2
//
//
