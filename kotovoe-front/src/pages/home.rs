use dioxus::html::img;
use dioxus::prelude::*;
use gloo_timers::future::TimeoutFuture;
use serde_json::error::Category;
use crate::server_api::get_resurse_img;
use crate::components::card::{Card, CardInfo};

use serde::{Serialize,Deserialize};
use dioxus::prelude::*;
use gloo_net::http::Request;

use crate::shared::{SignalUpdate};
use log::info;

// fn image_to_data_url(bytes: &[u8], mime_type: &str) -> String {
//     let encoded = encode(bytes);
//     format!("data:{};base64,{}", mime_type, encoded)
// }
#[derive(Deserialize, Debug, Clone)]
struct CatApi {
    id:i32,
    image: String,
}

#[derive(Serialize,Deserialize, Debug, Clone)]
struct SwipeInput{

    usr_id : i32,
    cat_id : i32,
    swipe_chose: bool,

}

#[derive(Debug, Deserialize, Serialize)]
struct AlbumUsrInput {
    id_usr: i32,
    id_cat: i32,
    usr_grade: i32,
}

#[component]
pub fn Home() -> Element {
    
    let crown = asset!("/assets/crown.png");
    let boxs = asset!("/assets/box.png");
    
    let cat = use_signal(|| CatApi{id:0,image:"".to_string()});
    
    let cat_obj = use_resource(move || async move {
        
        let mut cat_id = cat.clone();

        let response = reqwest::get("http://127.0.0.1:3000/api/cat")
        .await
        .unwrap()
        .json::<CatApi>()
        .await;

        cat_id.set(response.unwrap().clone());

        cat_id().image

    });

    let animation_class: Signal<Option<String>> = use_signal(|| None);
    let usr = use_context::<SignalUpdate>();
    
    let drop_signal = {

        let mut animation_class = animation_class.clone();
        let mut cat_obj = cat_obj.clone();
        let usr = usr.clone();
        let cat = cat.clone();
        move |_evt: Event<MouseData>| {
            animation_class.set(Some("swipe-left".to_string()));
            
            spawn(async move { 
                info!("{:?} {}", usr.sig, cat().id);

                let user_input = SwipeInput {
                    usr_id: usr.sig.to_string().parse().unwrap(),
                    cat_id: cat().id,
                    swipe_chose: false,
                };

                let response = Request::post("http://localhost:3000/swipe")
                .header("Content-Type", "application/json")
                .body(serde_json::to_string(&user_input).unwrap())
                .unwrap()
                .send()
                .await;

                info!("{:?}", response);

                TimeoutFuture::new(800).await;
                cat_obj.restart();
                TimeoutFuture::new(100).await;
                animation_class.set(Some("restart".to_string()));
                TimeoutFuture::new(200).await;
                animation_class.set(None);


            });



        }
    };

    let save_signal = {

        let mut animation_class = animation_class.clone();
        let mut cat_obj = cat_obj.clone();

        let usr = usr.clone();
        let cat = cat.clone();
        move |_evt: Event<MouseData>| {
            animation_class.set(Some("swipe-right".to_string()));

            spawn(async move {

                let user_input = SwipeInput {
                    usr_id: usr.sig.to_string().parse().unwrap(),
                    cat_id: cat().id,
                    swipe_chose: true,
                };

                let user_chose = AlbumUsrInput {
                        id_usr: usr.sig.to_string().parse().unwrap(),
                        id_cat: cat().id,
                        usr_grade: 5,
                    };

                let response = Request::post("http://localhost:3000/swipe")
                .header("Content-Type", "application/json")
                .body(serde_json::to_string(&user_input).unwrap())
                .unwrap()
                .send()
                .await;

                let response = Request::post("http://localhost:3000/album/")
                .header("Content-Type", "application/json")
                .body(serde_json::to_string(&user_chose).unwrap())
                .unwrap()
                .send()
                .await;

                info!("{:?}", response);

                TimeoutFuture::new(800).await;
                cat_obj.restart();
                TimeoutFuture::new(100).await;
                animation_class.set(Some("restart".to_string()));
                TimeoutFuture::new(200).await;
                animation_class.set(None);
            });
        }
    };


    rsx! {
        div {id:"home",
                div {
                    id:"home_left",
                    div {
                        button {  
                            onclick:drop_signal,  
                            img {  
                                src: boxs
                            }
                        }
                        p{"Drop this card!"}
    
                        }
                    }
                    
                    match cat_obj.read().as_ref() {
                        Some(data) => rsx! {
                            div {
                                id: "home_card_slot",
                                class: animation_class,
                                Card {
                                    data: data.clone(),
                                    id: None,
                                    // animation:animation_class
                                }
                                div {
                                id: "card_text",
                                div { "This is a card with cat!" }
                                div { "Will you add it to the album?" }
                                div { "👉👈" }

                            }
                            }
                        },
                        _ => rsx! {
                            div { "loading..." }
                        },
                    }

                div {
                    id:"home_right",
                    div {
                        button {    
                            onclick:save_signal,
                            img {  
                                src: crown
                            }
                        }
                        p{"Go to my collection"}
                    }
                }
            }

        }
}