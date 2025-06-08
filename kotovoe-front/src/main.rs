mod components;
mod pages;
mod server_api;
use crate::pages::auth;
use crate::pages::Route;
use gloo_storage::{Storage, LocalStorage};
use gloo_net::http::Request;
use dioxus::prelude::*;
mod shared;
use shared::{SignalUsr, Usr, SignalUpdate};
use serde::{Deserialize,Serialize};
use log::info;

#[derive(Deserialize,Serialize,Debug)]
struct UsrIdToJWT {
    user_id: i32,
}

async fn fetch_protected() -> Result<UsrIdToJWT, gloo_net::Error> {
    let response = Request::get("http://127.0.0.1:3000/protect")
        .header("Accept", "application/json")
        .credentials(web_sys::RequestCredentials::Include) // <<< ЭТО ГЛАВНОЕ
        .send()
        .await?;

    info!("{:?}",response   );
    let data = response.json::<UsrIdToJWT>().await?;
    Ok(data)
}


fn main() {

    console_log::init_with_level(log::Level::Debug).expect("init log");

    dioxus::launch(App);
}
const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

#[component]
pub fn App() -> Element {
    
    
    let auth:Signal<bool> = use_signal(|| false );
    let user_id = use_signal(|| 0 );

    // use_context_provider(|| user_id);
    
    // let mut user_id = user_id.clone();
    use_effect(move || {

        let mut auth = auth.clone();
        let mut usr_id = user_id.clone();

        spawn(async move {

            let id = fetch_protected().await;

            info!("{:?}",id);


            if let Ok(token) = LocalStorage::get::<bool>("token") {
                auth.set(true);
            }

            if let Ok(id) = LocalStorage::get::<i32>("usr"){
                
                usr_id.set(id);
                
            }
        });
    });

    use_context_provider(|| SignalUpdate{sig : user_id.clone()});

    
    static BACKGROUND: Asset = asset!("/assets/background2.png");
        rsx! {
            document::Link { rel: "icon", href: FAVICON }
            document::Link { rel: "stylesheet", href: MAIN_CSS }


            body {
                style: "background-image: url({BACKGROUND});
                        background-size: cover;
                        background-position: center center;
                        background-repeat: no-repeat;",

            if auth(){

                    Router::<Route> {}
                    
                    } else {
                        
                        auth::Auth { authp: auth.clone() }
                    
                    }
                }
        }
    }
