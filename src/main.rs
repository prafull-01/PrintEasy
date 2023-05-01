use axum::{
    body,
    extract::{
        ws::{Message, WebSocket},
        DefaultBodyLimit, Multipart, Path, State, WebSocketUpgrade,
    },
    http::StatusCode,
    response::{IntoResponse, Response},
    routing, Json, Router,
};
use std::{collections::HashSet, net::SocketAddr, sync::Arc};
use tokio::sync::mpsc;
use tower_http::services::ServeDir;
use uuid::Uuid;

use printeasy_server::{CreateShopArgs, NewPrintArgs, PageType, PrintType};

#[derive(Debug, serde::Serialize)]
pub enum PrintResponse {
    Cost(usize),
    Printed,
}

#[derive(Debug)]
pub struct Shop {
    page_capabilities: HashSet<PageType>,
    print_capabilities: HashSet<PrintType>,
    tx: Option<mpsc::Sender<(NewPrintArgs, mpsc::Sender<PrintResponse>)>>,
}

#[derive(Debug, Default)]
pub struct AppState {
    print_jobs: dashmap::DashMap<Uuid, mpsc::Receiver<PrintResponse>>,
    shops: dashmap::DashMap<Uuid, Shop>,
}

#[tokio::main]
async fn main() {
    // build our application with a route
    let app = Router::new()
        .nest_service("/", ServeDir::new("formvalidation"))
        .route("/api/v1/shop", routing::post(create_shop))
        .route(
            "/api/v1/shop/:shop_id",
            routing::post(init_print_job_for_shop),
        )
        .route("/api/v1/shop/ws/:ws_id", routing::get(connect_print_job))
        .route("/api/v1/shop/:shop_id/connect", routing::get(connect_shop))
        .with_state(Arc::new(AppState::default()))
        .layer(DefaultBodyLimit::max(30 * 1000 * 1000));

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn create_shop(
    State(state): State<Arc<AppState>>,
    args: Json<CreateShopArgs>,
) -> Json<serde_json::Value> {
    let id = Uuid::new_v4();

    state.shops.insert(
        id,
        Shop {
            page_capabilities: args.0.page_capabilities,
            print_capabilities: args.0.print_capabilities,
            tx: None,
        },
    );

    Json(serde_json::json!({
        "id": id,
    }))
}

async fn init_print_job_for_shop(
    State(state): State<Arc<AppState>>,
    Path(printer_id): Path<Uuid>,
    mut data: Multipart,
) -> Response {
    let mut args = NewPrintArgs::default();
    while let Some(mut field) = data.next_field().await.unwrap() {
        match field.name() {
            Some("name") => args.name = field.text().await.unwrap(),
            Some("system_id") => args.system_id = field.text().await.unwrap(),
            Some("phone_number") => args.phone_number = field.text().await.unwrap(),
            Some("email_id") => args.email_id = field.text().await.unwrap(),
            Some("file") => {
                while let Some(data) = field.chunk().await.unwrap() {
                    args.file.extend(data);
                }
            }
            Some("page_type") => args.page_type = field.text().await.unwrap().try_into().unwrap(),
            Some("print_type") => args.print_type = field.text().await.unwrap().try_into().unwrap(),
            Some(_) | None => {}
        }
    }

    if let Some(shop) = state.shops.get(&printer_id) {
        if let Some(tx) = &shop.tx {
            let (resp_tx, resp_rx) = mpsc::channel(10);

            let id = Uuid::new_v4();
            state.print_jobs.insert(id, resp_rx);

            tx.send((args, resp_tx)).await.unwrap();

            Json(serde_json::json!({
                "ws_id": id,
            }))
            .into_response()
        } else {
            // TODO: Shop was found but it is not conncted
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(body::boxed(body::Empty::new()))
                .unwrap()
        }
    } else {
        // TODO: Shop was not found
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(body::boxed(body::Empty::new()))
            .unwrap()
    }
}

async fn connect_print_job(
    State(state): State<Arc<AppState>>,
    Path(ws_id): Path<Uuid>,
    ws: WebSocketUpgrade,
) -> Response {
    if let Some((_, print_rx)) = state.print_jobs.remove(&ws_id) {
        ws.on_upgrade(move |socket| async move {
            handle_print_job(socket, state, ws_id, print_rx).await
        })
    } else {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(body::boxed(body::Empty::new()))
            .unwrap()
    }
}

async fn handle_print_job(
    mut socket: WebSocket,
    state: Arc<AppState>,
    ws_id: Uuid,
    mut rx: mpsc::Receiver<PrintResponse>,
) {
    while let Some(resp) = rx.recv().await {
        if socket
            .send(Message::Text(serde_json::to_string(&resp).unwrap()))
            .await
            .is_err()
        {
            state.print_jobs.insert(ws_id, rx);
            break;
        }
    }
}

async fn connect_shop(
    State(state): State<Arc<AppState>>,
    Path(shop_id): Path<Uuid>,
    ws: WebSocketUpgrade,
) -> Response {
    let (tx, rx) = mpsc::channel(100);
    if let Some(mut shop) = state.shops.get_mut(&shop_id) {
        shop.tx = Some(tx);

        ws.on_upgrade(move |socket| async { handle_shop(socket, rx).await })
    } else {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(body::boxed(body::Empty::new()))
            .unwrap()
    }
}

async fn handle_shop(
    mut socket: WebSocket,
    mut rx: mpsc::Receiver<(NewPrintArgs, mpsc::Sender<PrintResponse>)>,
) {
    while let Some((print_args, resp_tx)) = rx.recv().await {
        if socket
            .send(Message::Text(serde_json::to_string(&print_args).unwrap()))
            .await
            .is_err()
        {
            break;
        }

        let Some(cost) = socket.recv().await else {
            break;
        };
        let Ok(Message::Text(cost_json)) = cost else {
            break;
        };

        let cost: serde_json::Value = serde_json::from_str(&cost_json).unwrap();
        resp_tx
            .send(PrintResponse::Cost(cost["cost"].as_u64().unwrap() as usize))
            .await
            .unwrap();

        let Some(printed) = socket.recv().await else {
            break;
        };
        let Ok(Message::Text(printed)) = printed else {
            break;
        };
        if printed == "Printed" {
            resp_tx.send(PrintResponse::Printed).await.unwrap();
        }
    }

    while let Some(msg) = socket.recv().await {
        let msg = if let Ok(msg) = msg {
            msg
        } else {
            // client disconnected
            return;
        };

        if socket.send(msg).await.is_err() {
            // client disconnected
            return;
        }
    }
}
