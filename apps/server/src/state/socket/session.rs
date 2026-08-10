use crate::state::messages::{Envelope, envelope};
use indexmap::{IndexMap, IndexSet, indexset};
use prost::{Message, bytes::Bytes};
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
    pub fn join_voice(&self, channel_id: Uuid, user_id: Uuid) -> Result<(), SessionError> {
        let mut lock = self.0.write().expect("failed to lock");

        let entry = lock.voice.entry(channel_id);

        entry
            .and_modify(|list| {
                list.insert(user_id);
            })
            .or_insert_with(|| {
                indexset! {
                    user_id
                }
            });

        Ok(())
    }
    pub fn leave_voice(&self, channel_id: Uuid, user_id: Uuid) -> Result<(), SessionError> {
        let mut lock = self.0.write().expect("failed to lock");

        let entry = lock.voice.entry(channel_id);

        entry.and_modify(|list| {
            list.swap_remove(&user_id);
        });

        if lock.voice.is_empty() {
            lock.voice.swap_remove(&channel_id);
        }

        Ok(())
    }

    pub fn register(
        &self,
        user_id: Uuid,
        tx: tokio::sync::mpsc::Sender<Bytes>,
    ) -> Result<Uuid, SessionError> {
        let conn_id = uuid::Uuid::now_v7();
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

        Ok(conn_id)
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

    pub fn is_online(&self, user_id: &Uuid) -> Result<bool, SessionError> {
        let lock = self.0.read().expect("failed to lock");

        if let Some(user) = lock.users.get(user_id) {
            return Ok(!user.is_empty());
        }

        Ok(false)
    }

    pub fn broadcast(&self, msg: &Envelope) -> Result<(), SessionError> {
        Ok(())
    }

    pub fn voice_members(&self, channel_id: &Uuid) -> Result<Vec<Uuid>, SessionError> {
        let lock = self.0.read().expect("failed to lock");

        if let Some(info) = lock.voice.get(channel_id) {
            Ok(info.iter().map(Uuid::to_owned).collect::<Vec<Uuid>>())
        } else {
            Ok(Vec::default())
        }
    }

    pub fn send_to(&self, user_id: &Uuid, msg: Bytes) -> Result<(), SessionError> {
        let lock = self.0.read().expect("failed to lock");

        if let Some(user_conns) = lock.users.get(user_id) {
            for conn_id in user_conns {
                if let Some(conn) = lock.connections.get(conn_id) {
                    if let Err(err) = conn.tx.try_send(msg.clone()) {
                        log::error!("failed to send message: {}", err)
                    };
                };
            }
        };

        Ok(())
    }
}

//      let mut bytes = Bytes::new();
// msg.encode(&mut bytes);
// users
//          -> session 1
//          -> session 2
//
//
