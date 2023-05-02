use printeasy_server::{CreateShopArgs, NewPrintArgs, PageType, PrintType};

use std::path::PathBuf;

use dioxus::prelude::*;
use futures_util::StreamExt;
use reqwest::Client;
use tokio_tungstenite::tungstenite::{protocol::WebSocketConfig, Message};

const BASE_URL: &str = "localhost:3000";

#[derive(Debug, serde::Deserialize, Clone)]
struct PrintArgs {
    pub name: String,
    pub system_id: String,
    pub phone_number: String,
    pub email_id: String,
    pub file_path: PathBuf,
    pub page_type: PageType,
    pub print_type: PrintType,
}

fn main() {
    // launch the dioxus app in a webview
    dioxus_desktop::launch(app);
}

// define a component that renders a div with the text "Hello, world!"
fn app(cx: Scope) -> Element {
    let mut tmp_dir = std::env::temp_dir();
    tmp_dir.push("print_easy_tmp_dir");
    let _ = std::fs::create_dir(&tmp_dir);

    let id: &UseState<Option<String>> = use_state(cx, || None);
    let print_queue: &UseState<Vec<PrintArgs>> = use_state(cx, || Vec::default());

    let _ws_coroutine: &Coroutine<()> = use_coroutine(cx, |_| {
        let shop_id = id.to_owned();
        let print_queue = print_queue.to_owned();
        let tmp_dir = tmp_dir.clone();
        async move {
            let client = Client::new();
            let resp = client
                .post(format!("http://{BASE_URL}/api/v1/shop"))
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
                url::Url::parse(&format!("ws://{BASE_URL}/api/v1/shop/{id}/connect")).unwrap();

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
            let (_, mut read) = ws_stream.split();

            while let Some(msg) = read.next().await {
                let Ok(Message::Text(msg)) = msg else {
                    panic!("Got invalid message from server");
                };

                let item: NewPrintArgs = serde_json::from_str(&msg).unwrap();
                let mut tmp_dir = tmp_dir.clone();
                tmp_dir.push(item.file_name.unwrap_or("default.png".to_owned()));

                let image = image::load_from_memory(&item.file).unwrap();
                image.save(&tmp_dir).unwrap();

                print_queue.modify(|queue| {
                    let mut nq = queue.clone();
                    nq.push(PrintArgs {
                        name: item.name,
                        system_id: item.system_id,
                        phone_number: item.phone_number,
                        email_id: item.email_id,
                        file_path: tmp_dir,
                        page_type: item.page_type,
                        print_type: item.print_type,
                    });

                    dbg!(nq)
                });
            }
        }
    });

    cx.render(match id.as_ref() {
        Some(id) => {
            let url = format!("http://{BASE_URL}?shop_id={id}");
            let code = qrcode::QrCode::new(&url).unwrap();

            let mut tmp_dir = tmp_dir.clone();

            tmp_dir.push("qrcode.png");
            let img = code.render::<image::Luma<u8>>().build();
            let _ = std::fs::remove_file(&tmp_dir);
            img.save(&tmp_dir).unwrap();

            let src = tmp_dir.to_str();
            let src = src.unwrap().to_string();

            rsx! {
                head {
                    link { rel: "stylesheet", href: "https://unpkg.com/tailwindcss@^2.0/dist/tailwind.min.css" }
                }
                body {
                    class: "md:flex",
                    div {
                        style: "flex-grow: 1; overflow-y: scroll",
                        print_queue.iter().map(|print| {
                            let print_file = print.file_path.to_str();
                            let print_file = print_file.unwrap().to_string();
    
                            rsx! {
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
                                    img {
                                        src: "{print_file}",
                                        style: "height: 250px"
                                    }
                                }
                            }
                        })
                    }
                    div {
                        img {
                            src: "{src}",
                            style: "height: 250px; width: 250px; margin: auto"
                        }
                        span {
                            "{id}"
                        }
                    }
                }
            }
        }
        None => rsx! {
            div {
                "Connecting"
            }
        },
    })
}
