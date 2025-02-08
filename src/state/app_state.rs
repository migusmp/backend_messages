use rdkafka::{
    producer::{FutureProducer, FutureRecord},
    ClientConfig,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::{broadcast, mpsc, Mutex};

pub struct AppState {
    pub global_broadcast: broadcast::Sender<String>,
    pub user_connections: Arc<Mutex<HashMap<i32, mpsc::Sender<String>>>>,
    pub friend_notifications: Arc<Mutex<HashMap<i32, mpsc::Sender<String>>>>,
    pub undelivered_messages: Arc<Mutex<HashMap<i32, Vec<String>>>>,
    pub config: AppConfig,
}

pub struct AppConfig {
    pub app_name: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            app_name: String::from("Migus App"),
        }
    }
}
pub struct KafkaProducer {
    producer: FutureProducer,
}

impl KafkaProducer {
    // Inicializamos el productor de Kafka solo una vez
    pub fn new() -> Self {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", "localhost:9092")
            .create()
            .expect("Error al crear el productor de Kafka");

        KafkaProducer { producer }
    }

    // Método para enviar una notificación
    pub async fn send_notification(&self, topic: &str, key: &str, payload: &str) {
        let record = FutureRecord::to(topic).key(key).payload(payload);

        // Intentamos enviar el mensaje, reintentando en caso de error transitorio
        let result = self.producer.send(record, Duration::from_secs(0)).await;

        if let Err((e, _)) = result {
            eprintln!("❌ Error enviando mensaje a Kafka: {:?}", e);
            // Aquí podrías implementar un mecanismo de reintento
        }
    }
}

// Crear una instancia global de KafkaProducer, que puede ser compartida en todo el programa
lazy_static::lazy_static! {
    static ref KAFKA_PRODUCER: Arc<Mutex<KafkaProducer>> = Arc::new(Mutex::new(KafkaProducer::new()));
}

#[derive(Serialize, Deserialize)]
pub struct FriendNotification {
    pub type_msg: String,
    pub status: String,
    pub user_id: i32,
    pub user_name: String,
    pub message: String,
}

impl AppState {
    pub fn new() -> Self {
        let (global_broadcast, _) = broadcast::channel::<String>(1000);
        Self {
            global_broadcast,
            user_connections: Arc::new(Mutex::new(HashMap::new())),
            friend_notifications: Arc::new(Mutex::new(HashMap::new())),
            undelivered_messages: Arc::new(Mutex::new(HashMap::new())),
            config: AppConfig::default(),
        }
    }

    pub async fn add_user_connection(&self, user_id: i32, sender: mpsc::Sender<String>) {
        let mut connections = self.user_connections.lock().await;
        connections.insert(user_id, sender);
    }

    pub async fn remove_user_connection(&self, user_id: i32) {
        let mut connections = self.user_connections.lock().await;
        connections.remove(&user_id);
    }

    // Método para añadir un usuario conectado a las notificaciones.
    pub async fn add_user_to_friend_notifications(&self, user_id: i32) -> mpsc::Receiver<String> {
        let (sender, receiver) = mpsc::channel::<String>(100); // Crear canal para el usuario
        let mut notifications = self.friend_notifications.lock().await;
        notifications.insert(user_id, sender); // Agregar el canal a la lista de usuarios conectados
        receiver // Devolver el receptor para que el usuario pueda recibir mensajes
    }

    pub async fn send_friend_notification_to_kafka(
        friend_id: i32,
        sender_name: String,
        sender_id: i32,
    ) {
        let notification = json!({
            "type": "friend_request",
            "friend_id": friend_id,
            "sender_id": sender_id,
            "sender_name": sender_name,
        });

        let payload = serde_json::to_string(&notification).unwrap();
        let key = friend_id.to_string();

        let producer = KAFKA_PRODUCER.lock().await;
        producer
            .send_notification("notificaciones", &key, &payload)
            .await;
    }
    // pub async fn send_friend_notification(
    //     &self,
    //     friend_id: i32,
    //     user_name: String,
    //     user_id: i32,
    // ) -> Result<(), String> {
    //     let notifications = self.friend_notifications.lock().await;
    //     let message = format!("{} send to you a friend request!", user_name);
    //
    //     let message = FriendNotification {
    //         type_msg: "FR".to_string(),
    //         user_id,
    //         user_name,
    //         status: "pending".to_string(),
    //         message,
    //     };
    //
    //     let json_message = match serde_json::to_string(&message) {
    //         Ok(msg) => msg,
    //         Err(_) => return Err(format!("Failed to serialize the notification")),
    //     };
    //
    //     if let Some(sender) = notifications.get(&friend_id) {
    //         match sender.try_send(json_message.clone()) {
    //             Ok(_) => Ok(()),
    //             Err(_e) => {
    //                 // Si no se puede enviar almacenamos el mensaje para cuando el usuario se
    //                 // conecte.
    //                 self.store_undelivered_message(friend_id, json_message)
    //                     .await;
    //                 Err(format!("Unable to send notification to {}", friend_id))
    //             }
    //         }
    //     } else {
    //         self.store_undelivered_message(friend_id, json_message)
    //             .await;
    //         Err(format!("El usuario {} no esta conectado", friend_id))
    //     }
    // }

    async fn store_undelivered_message(&self, user_id: i32, message: String) {
        let mut undelivered = self.undelivered_messages.lock().await;
        undelivered.entry(user_id).or_default().push(message);
    }

    pub async fn deliver_undelivered_messages(&self, user_id: i32) {
        let mut undelivered = self.undelivered_messages.lock().await;
        if let Some(messages) = undelivered.remove(&user_id) {
            let notifications = self.friend_notifications.lock().await;
            if let Some(sender) = notifications.get(&user_id) {
                for message in messages {
                    let _ = sender.try_send(message);
                }
            }
        }
    }

    pub async fn accept_friend_notification(
        &self,
        friend_id: i32,
        user_name: String,
        user_id: i32,
    ) -> Result<(), String> {
        let notifications = self.friend_notifications.lock().await;
        let message = format!("{} accept your friend request!", user_name);

        let message = FriendNotification {
            type_msg: "AFR".to_string(),
            user_id,
            user_name,
            status: "success".to_string(),
            message,
        };

        let json_message = match serde_json::to_string(&message) {
            Ok(msg) => msg,
            Err(_) => return Err(format!("Failed to serialize the notification")),
        };

        if let Some(sender) = notifications.get(&friend_id) {
            match sender.try_send(json_message.clone()) {
                Ok(_) => Ok(()),
                Err(_e) => {
                    self.store_undelivered_message(friend_id, json_message)
                        .await;
                    Err(format!("No se pudo enviar la notificacion a {}", friend_id))
                }
            }
        } else {
            self.store_undelivered_message(friend_id, json_message)
                .await;
            Err(format!("El usuario {} no esta conectado", friend_id))
        }
    }
    pub async fn accept_friend_notification_kafka(friend_id: i32, user_name: String, user_id: i32) {
        // Crear el mensaje de aceptación
        let message = json!({
            "type": "friend_request_accepted",  // Tipo de notificación
            "friend_id": friend_id,            // ID del amigo
            "user_name": user_name,            // Nombre del usuario que acepta
            "user_id": user_id,                // ID del usuario que acepta
        });

        // Convertir el mensaje a un String
        let payload = serde_json::to_string(&message).unwrap();

        // Definir la clave del mensaje (friend_id)
        let key = friend_id.to_string();

        println!("📢 Enviando notificación de aceptación de solicitud de amistad...");

        // Obtener el productor de Kafka de manera eficiente
        let producer = KAFKA_PRODUCER.lock().await;
        producer
            .send_notification("notificaciones", &key, &payload)
            .await;

        println!(
            "📢 Notificación de aceptación de solicitud de amistad enviada para el usuario {}",
            friend_id
        );
    }
    pub async fn add_user_to_global_broadcast(&self) -> broadcast::Receiver<String> {
        // Crea un nuevo receptor para el broadcast
        let (_tx, rx) = broadcast::channel::<String>(100); // Buffer de 100 mensajes

        rx // Devolver el receptor para que el usuario pueda escuchar los mensajes
    }

    // Método para enviar un mensaje global a todos los usuarios
    pub async fn send_global_broadcast(&self, message: String) {
        let _ = self.global_broadcast.send(message); // Enviar el mensaje al canal global
    }
}
