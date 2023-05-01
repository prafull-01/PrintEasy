use printeasy_server::{CreateShopArgs, NewPrintArgs, PageType, PrintType};

use dioxus::prelude::*;
use futures_util::StreamExt;
use reqwest::Client;
use tokio_tungstenite::tungstenite::{protocol::WebSocketConfig, Message};

fn main() {
    // launch the dioxus app in a webview
    dioxus_desktop::launch(app);
}

// define a component that renders a div with the text "Hello, world!"
fn app(cx: Scope) -> Element {
    let id: &UseState<Option<String>> = use_state(cx, || None);
    let print_queue: &UseState<Vec<NewPrintArgs>> = use_state(cx, || Vec::default());

    let _ws_coroutine: &Coroutine<()> = use_coroutine(cx, |_| {
        let shop_id = id.to_owned();
        let print_queue = print_queue.to_owned();
        async move {
            let client = Client::new();
            let resp = client
                .post("http://localhost:3000/api/v1/shop")
                .json(&CreateShopArgs {
                    page_capabilities: [PageType::A3, PageType::A4, PageType::A5]
                        .into_iter()
                        .collect(),
                    print_capabilities: [PrintType::Colored, PrintType::BlackAndWhite]
                        .into_iter()
                        .collect(),
                })
                .send()
                .await
                .unwrap()
                .json::<serde_json::Value>()
                .await
                .unwrap();
            let id = resp["id"].as_str().unwrap().to_owned();
            shop_id.set(Some(id.clone()));

            let url =
                url::Url::parse(&format!("ws://localhost:3000/api/v1/shop/{id}/connect")).unwrap();

            let (ws_stream, _) = tokio_tungstenite::connect_async_with_config(
                url,
                Some(WebSocketConfig {
                    max_send_queue: None,
                    max_message_size: None,
                    max_frame_size: None,
                    accept_unmasked_frames: false,
                }),
            )
            .await
            .expect("Failed to connect");
            let (_, read) = ws_stream.split();

            read.for_each(|message| async {
                let Ok(Message::Text(msg)) = message else {
                    panic!("Got invalid message from server");
                };

                print_queue.modify(|queue| {
                    let mut nq: Vec<NewPrintArgs> = vec![serde_json::from_str(&msg).unwrap()];
                    nq.extend(queue.into_iter().map(|arg| arg.clone()));
                    nq
                });
            })
            .await;
        }
    });

    cx.render(match id.as_ref() {
        Some(id) => rsx! {
            span {
                "Connected with id: {id}"
            }
            print_queue.iter().map(|print| rsx! {
                div {
                    div {
                        "Sender Name: {print.name}"
                    }
                    div {
                        "Sender System Id: {print.system_id}"
                    }
                    div {
                        "Sender Phone Number: {print.phone_number}"
                    }
                    div {
                        "Sender Email Id: {print.email_id}"
                    }
                    div {
                        "Page Type: {print.page_type:?}"
                    }
                    div {
                        "Print Type: {print.print_type:?}"
                    }
                }
            })
        },
        None => rsx! {
            div {
                "Connecting"
            }
        },
    })
}
