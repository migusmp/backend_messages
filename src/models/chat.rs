use std::collections::HashMap;

use axum::extract::ws::Message;
use tokio::sync::broadcast;

#[derive(Default)]
pub struct ChatState {
    pub rooms: HashMap<String, Room>, // Mapa de salas donde la clave es el room_id y el valor es un canal broadcast
}

#[derive(Clone)]
pub struct Room {
    pub users: HashMap<String, broadcast::Sender<Message>>,
}

impl ChatState {
    // Crear una nueva sala.
    pub fn create_room(&mut self, room_id: String) {
        self.rooms.insert(
            room_id,
            Room {
                users: HashMap::new(),
            },
        );
    }

    pub fn join_room(&mut self, room_id: &str, user_id: String) -> broadcast::Sender<Message> {
        // Asegurar que la sala exista
        let room = self
            .rooms
            .entry(room_id.to_string())
            .or_insert_with(|| Room {
                users: HashMap::new(),
            });

        // Si el usuario ya existe, devolver su canal existente
        if let Some(existing_sender) = room.users.get(&user_id) {
            existing_sender.clone()
        } else {
            // Crear un nuevo canal y añadirlo al mapa de usuarios
            let (tx, _rx) = broadcast::channel(52);
            room.users.insert(user_id, tx.clone());
            tx
        }
    }

    pub fn leave_room(&mut self, room_id: &str, user_id: &str) {
        if let Some(room) = self.rooms.get_mut(room_id) {
            room.users.remove(user_id);
        }
    }

    pub fn get_room_user_count(&self, room_id: &String) -> Option<usize> {
        self.rooms.get(room_id).map(|room| room.users.len())
    }

    pub async fn active_rooms(&self) -> (Vec<String>, Vec<String>, usize) {
        let mut active_rooms = vec![];
        let mut users_active = vec![];

        for (room_id, room) in self.rooms.clone() {
            active_rooms.push(room_id);

            for (user, _sender) in room.users {
                if users_active.contains(&user) {
                    continue;
                }
                users_active.push(user);
            }
        }

        (active_rooms, users_active, self.rooms.len())
    }
}
