//! Portable voice-room state.
//!
//! This module is the one piece of the spike that is NOT throwaway — if the
//! spike answers "yes, the stack works", this reducer lifts into the real
//! overlay client unchanged. Pure: no I/O, no tracing, no iced, no nng.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum VoiceEvent {
    RoomJoined { room_id: String, room_name: String },
    RoomLeft,
    UserJoined { user_id: String, display_name: String },
    UserLeft { user_id: String },
    UserSpeaking { user_id: String, speaking: bool },
    UserMuted { user_id: String, muted: bool },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Participant {
    pub user_id: String,
    pub display_name: String,
    pub speaking: bool,
    pub muted: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RoomState {
    pub active_room: Option<ActiveRoom>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveRoom {
    pub id: String,
    pub name: String,
    pub participants: Vec<Participant>,
}

impl RoomState {
    pub fn apply(&mut self, event: VoiceEvent) {
        match event {
            VoiceEvent::RoomJoined { room_id, room_name } => {
                self.active_room = Some(ActiveRoom {
                    id: room_id,
                    name: room_name,
                    participants: Vec::new(),
                });
            }
            VoiceEvent::RoomLeft => {
                self.active_room = None;
            }
            VoiceEvent::UserJoined { user_id, display_name } => {
                if let Some(room) = self.active_room.as_mut() {
                    if !room.participants.iter().any(|p| p.user_id == user_id) {
                        room.participants.push(Participant {
                            user_id,
                            display_name,
                            speaking: false,
                            muted: false,
                        });
                    }
                }
            }
            VoiceEvent::UserLeft { user_id } => {
                if let Some(room) = self.active_room.as_mut() {
                    room.participants.retain(|p| p.user_id != user_id);
                }
            }
            VoiceEvent::UserSpeaking { user_id, speaking } => {
                if let Some(room) = self.active_room.as_mut() {
                    if let Some(p) = room.participants.iter_mut().find(|p| p.user_id == user_id) {
                        p.speaking = speaking;
                    }
                }
            }
            VoiceEvent::UserMuted { user_id, muted } => {
                if let Some(room) = self.active_room.as_mut() {
                    if let Some(p) = room.participants.iter_mut().find(|p| p.user_id == user_id) {
                        p.muted = muted;
                    }
                }
            }
        }
    }
}
