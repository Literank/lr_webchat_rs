use socketioxide::{extract::SocketRef, SocketIo};
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

const PORT: i32 = 4000;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (layer, io) = SocketIo::builder().build_layer();
    io.ns("/", |s: SocketRef| println!("A user connected {}", s.id));

    let app = axum::Router::new().layer(
        ServiceBuilder::new()
            .layer(CorsLayer::permissive())
            .layer(layer),
    );
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", PORT))
        .await
        .unwrap();
    println!("Chat server serving at localhost:{}", PORT);
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
