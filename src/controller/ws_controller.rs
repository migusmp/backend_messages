use std::sync::Arc;

use axum::{
    extract::{
        ws::{Message, WebSocket},
        WebSocketUpgrade,
    },
    response::IntoResponse,
    Extension,
};
use tokio::sync::{broadcast, mpsc};

use crate::{models::user::Payload, state::app_state::AppState};

pub async fn handle_ws_connection(
    ws: WebSocketUpgrade,
    app_state: Arc<AppState>,
    Extension(payload): Extension<Payload>,
) -> impl IntoResponse {
    let user_id = payload.id;

    // Creamos un canal para enviar las notificaciones a este usuario.
    let (tx, rx) = mpsc::channel::<String>(100);

    app_state.add_user_connection(user_id, tx).await;
    let notifications_receiver = app_state.add_user_to_friend_notifications(user_id).await;
    let application_broadcast_receiver = app_state.add_user_to_global_broadcast().await;

    // Comprobamos si tiene mensajes pendientes y se los enviamos.
    app_state.deliver_undelivered_messages(payload.id).await;

    ws.on_upgrade(|socket| {
        handle_socket_connection(
            socket,
            app_state,
            payload,
            rx,
            notifications_receiver,
            application_broadcast_receiver,
        )
    })
}

async fn handle_socket_connection(
    mut socket: WebSocket,
    app_state: Arc<AppState>,
    payload: Payload,
    mut user_connection_rx: mpsc::Receiver<String>,
    mut friend_notifications_rx: mpsc::Receiver<String>,
    mut application_broadcast_rx: broadcast::Receiver<String>,
) {
    println!("User {} connected via WebSocket", &payload.name);

    let mut is_connected = true;

    while is_connected {
        tokio::select! {
            // Recibir mensajes desde el WebSocket.
            Some(Ok(message)) = socket.recv() => {
                match message {
                    Message::Text(text) => {
                        // Procesa el mensaje recibido si es necesario
                        println!("Received from user {}: {}", &payload.name, text);

                        // Puedes añadir lógica adicional aquí
                        // Ejemplo: manejar comandos o mensajes específicos
                    }
                    Message::Close(_) => {
                        println!("User {} disconnected", &payload.name);
                        is_connected = false;
                    }
                    _ => {}
                }
            }
            Some(notification) = friend_notifications_rx.recv() => {
                if let Err(err) = socket.send(Message::Text(notification.into())).await {
                    eprintln!("Error sending message to user {}: {:?}", &payload.name, err);
                    is_connected = false;
                }
            }

            Some(notification) = user_connection_rx.recv() => {
                if let Err(err) = socket.send(Message::Text(notification.into())).await {
                    eprintln!("Error sending message to user {}: {:?}", &payload.name, err);
                    is_connected = false;
                }
            }

            Ok(notification) = application_broadcast_rx.recv() => {
                if let Err(err) = socket.send(Message::Text(notification.into())).await {
                    eprintln!("Error sending message to user {}: {:?}", &payload.name, err);
                    is_connected = false;
                }
            }

            else => {
                println!("Connection to user {} closed unexpectedly", &payload.name);
                is_connected = false;
            }
        }
    }

    app_state.remove_user_connection(payload.id).await;
    println!("Connection to user {} removed", &payload.name);
}
