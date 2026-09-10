use crate::state::messages::Envelope;
use indexmap::{IndexMap, IndexSet, indexset};
use prost::{Message, bytes::Bytes};
use std::sync::RwLock;
use tokio::sync::mpsc::Sender;
use uuid::Uuid;

#[derive(Debug)]
pub struct Conn {
    id: Uuid,
    user_id: Uuid,

    tx: Sender<Bytes>,
}

#[derive(Debug, Default)]
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

    #[error("no connections available")]
    NoConns,
}

#[derive(Debug, Default)]
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
    pub fn leave_voice(&self, channel_id: &Uuid, user_id: &Uuid) -> Result<(), SessionError> {
        let mut lock = self.0.write().expect("failed to lock");

        let entry = lock.voice.entry(*channel_id);

        entry.and_modify(|list| {
            list.swap_remove(user_id);
        });

        if lock.voice.get(channel_id).is_some_and(|x| x.is_empty()) {
            lock.voice.swap_remove(channel_id);
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

    pub fn broadcast(&self, targets: &[Uuid], msg: Envelope) -> Result<(), SessionError> {
        // Collect senders under the read lock so it is released before any
        // unregister below takes the write lock (std RwLock is not reentrant).
        let data = {
            let lock = self.0.read().expect("poisoned lock");

            let mut results = Vec::<(Uuid, Sender<Bytes>)>::default();

            for user_id in targets {
                if let Some(user) = lock.users.get(user_id) {
                    for conns_id in user {
                        if let Some(conn) = lock.connections.get(conns_id) {
                            results.push((*conns_id, conn.tx.clone()));
                        }
                    }
                }
            }

            results
        };

        if data.is_empty() {
            return Ok(());
        }

        let bytes = Bytes::from(msg.encode_to_vec());
        for (conn_id, tx) in data {
            // A full channel means the client is not draining fast enough. A
            // client that silently misses events is worse than one that knows
            // it is gone, so drop the connection: unregistering releases the
            // last Sender, the writer task sees the channel close and sends a
            // close frame, and the client reconnects and resyncs.
            if let Err(err) = tx.try_send(bytes.clone()) {
                log::warn!("dropping connection {}: {}", conn_id, err);
                self.unregister(&conn_id)?;
            }
        }

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

    pub fn send_to(&self, user_id: &Uuid, msg: Envelope) -> Result<(), SessionError> {
        let conns = {
            let lock = self.0.read().expect("failed to lock");

            let mut conns = Vec::<(Uuid, Sender<Bytes>)>::default();

            if let Some(user_conns) = lock.users.get(user_id) {
                for conn_id in user_conns {
                    if let Some(conn) = lock.connections.get(conn_id) {
                        conns.push((*conn_id, conn.tx.clone()));
                    };
                }
            };
            conns
        };

        if conns.is_empty() {
            return Err(SessionError::NoConns);
        }

        let bytes = Bytes::from(msg.encode_to_vec());

        for (id, tx) in conns {
            if let Err(err) = tx.try_send(bytes.clone()) {
                log::warn!("dropping connection {}: {}", id, err);
                self.unregister(&id)?
            };
        }

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
