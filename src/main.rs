use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use socketioxide::{
    extract::{Data, SocketRef, State},
    SocketIo,
};
use tokio::{net::TcpListener, sync::RwLock};
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

const PORT: i32 = 4000;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct User {
    name: String,
    emoji: String,
    sid: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ChatData {
    to: String,
    from: String,
    msg: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct CreateGroupData {
    sids: Vec<String>,
    name: String,
    id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GroupChatData {
    room: String,
    speaker: String,
    msg: String,
}

type UserStore = HashMap<String, User>;

async fn on_connect(s: SocketRef) {
    println!("A user connected {}", s.id);

    s.on(
        "user-join",
        |s: SocketRef, Data::<User>(user), users: State<RwLock<UserStore>>| async move {
            if user.name.is_empty() {
                return;
            }
            println!("User {} => {} {} joined", s.id, user.emoji, user.name);
            let new_user = User {
                name: user.name,
                emoji: user.emoji,
                sid: Some(s.id.to_string()),
            };
            s.join(s.id).ok(); // Hack: join its own room for private messaging
            let mut binding = users.write().await;
            binding.insert(s.id.to_string(), new_user);
            let items: Vec<(String, User)> = binding
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            // If you provide array-like data (tuple, vec, arrays), it will be considered as multiple arguments.
            // Therefore if you want to send an array as the first argument of the payload, you need to wrap it in an array or a tuple.
            s.broadcast().emit("contacts", (items.clone(),)).ok();
            s.emit("contacts", (items,)).ok(); // Emit to itself as well
        },
    );

    s.on("chat", |s: SocketRef, Data::<ChatData>(data)| async move {
        let to = data.to.clone();
        s.to(to).emit("chat", data).ok();
    });

    // Create Room
    s.on(
        "create-group",
        |s: SocketRef, Data::<CreateGroupData>(data)| async move {
            for socket_id in data.sids.clone() {
                s.within(socket_id).join(data.id.clone()).ok();
            }
            let room_id = data.id.clone();
            let room_name = data.name.clone();
            s.within(data.id.clone()).emit("create-group", data).ok();
            println!("Room {} => {} created", room_id, room_name)
        },
    );

    s.on(
        "group-chat",
        |s: SocketRef, Data::<GroupChatData>(data)| async move {
            let room = data.room.clone();
            s.to(room).emit("group-chat", data).ok();
        },
    );
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start_users: RwLock<UserStore> = HashMap::new().into();
    let (layer, io) = SocketIo::builder().with_state(start_users).build_layer();
    io.ns("/", on_connect);

    let app = axum::Router::new().layer(
        ServiceBuilder::new()
            .layer(CorsLayer::permissive())
            .layer(layer),
    );
    let listener = TcpListener::bind(format!("0.0.0.0:{}", PORT))
        .await
        .unwrap();
    println!("Chat server serving at localhost:{}", PORT);
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
