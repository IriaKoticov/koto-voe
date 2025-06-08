use dioxus::prelude::*;
use gloo_storage::{LocalStorage, Storage};
// use reqwest::{self, StatusCode};
use gloo_net::http::Request;
use gloo_timers::future::TimeoutFuture;
use web_sys::{console::assert, window};

use crate::{pages::auth, shared::{SignalUsr, Usr,LoginResponse}};
use serde::{Serialize, Deserialize};

use log::info;

#[derive(Serialize)]
struct UserInput {
    login: String,
    email: String,
    pas_hash: String,
}


#[derive(Deserialize)]
struct AuthResponse {
    usr_id: i32,
}

#[derive(Debug,Serialize,Deserialize)]
struct UserId{id:i32}



#[derive(Props, PartialEq, Clone)]
pub struct AuthProps {
    pub authp: Signal<bool>,
}

#[component]
pub fn Auth(props:AuthProps) -> Element {

    let lg = asset!("/assets/logo.png"); 
    let cat_with_threads = asset!("/assets/catwith.png");
    let fnaf = asset!("/assets/fnaf.gif");

    let mut toggled = use_signal(|| false);

    let img_src:Asset = if *toggled.read() {
        asset!("/assets/fnaf.gif")
    } else {
        asset!("/assets/catwith.png")
    };


    let mut action = use_signal(|| "Login".to_string());
    let mut username = use_signal(|| "".to_string());
    let mut password = use_signal(|| "".to_string());
    let mut email = use_signal(|| "".to_string()); 

    let authp = props.authp.clone();
    
    // let mut sigglob = props.authp.write().flag;
    
    rsx! {
        div {
            id: "AuthMainLayer",
            div {
                id:"AuthLayerForSelect",
                div {
                    img { src: lg, 
                        width: "10%",    // ширина картинки (можно менять)
                        height: "auto",  // сохраняет пропорции
                        style: "  
                            display: block;         
                            margin: 0 auto 20px;    
                            width: 120px;           
                            height: auto;           
                            object-fit: contain;    
                            cursor: default;"
                    
                    }

                }

                div {
                    id:"auth_text",
                    "{*action.read()} menu"
                }
                div {
                    id: "action_select",
                    button {
                        onclick: move |_| action.set("Login".to_string()),
                        class: if *action.read() == "Login" { "selected" } else { "" },
                        "Login"
                    }
                    button {
                        onclick: move |_| action.set("Register".to_string()),
                        class: if *action.read() == "Register" { "selected" } else { "" },
                        "Register"
                    }
                }
                div {
                    id: "AuthForm",
                    
                    form {
                        
                        onsubmit: move |evt| {
                            // evt.prevent_default();
                            
                            let current_action = action.read().clone();
                            let username = username.read().clone();
                            let password = password.read().clone();
                            let email = email.read().clone();
                            
                            
                            spawn(async move {
                            let mut authp = authp.clone();
                                
                                
                                let user_input = UserInput {
                                    login: username.clone(),
                                    email: email.clone(),
                                    pas_hash: password.clone(), 
                                };
                                
                                let url = if current_action == "Login" {
                                    "http://localhost:3000/auth"
                                } else {
                                    "http://localhost:3000/user"
                                };
                                
                                let response = Request::post(url)
                                .credentials(web_sys::RequestCredentials::Include)
                                .header("Content-Type", "application/json")
                                .body(serde_json::to_string(&user_input).unwrap())
                                .unwrap()
                                .send()
                                .await;
                            
                                
                            
                            match response {
                                Ok(resp) => {
                                    if resp.status() == 200 {
                                        
                                        info!("{:?}",resp);
                                        info!("Успешный вход: {}", resp.status());


                                        let Ok(data_str) = resp.text().await else {
                                            log::warn!("Не удалось прочитать тело");
                                            return;
                                        };

                                        if let Ok(auth_response) = serde_json::from_str::<AuthResponse>(&data_str) {
                                        log::info!("Пользователь ID: {}", auth_response.usr_id);
                                        LocalStorage::set("usr", auth_response.usr_id).unwrap();
                                        LocalStorage::set("token", true).expect("failed to save token"); 
                                        
                                    } else {
                                        log::warn!("Ответ не соответствует структуре AuthResponse: {}", data_str);
                                    }
                                 
                                    if let Some(win) = window() {
                                        let _ = win.location().reload();
                                    }
                                        
                                    } else if resp.status() == 201 {
                                        action.set("Login".to_string());
                                        if let Some(win) = window() {
                                            let _ = win.location().reload();
                                        }
                                    } else {

                                        log::warn!("Ошибка входа: {}", resp.status());
                                    }

                                }
                                Err(err) => {

                                    LocalStorage::delete("token");

                                }
                            }

                        });
                        

                    },

                    div {  
                        id:"auth_div",
                        label {
                            "Your login:"
                        }
    
                        input {
                            r#type: "text",
                            value: "{username}",
                            oninput: move |evt| username.set(evt.value().clone()),
                            id: "inputbox"
                        }
                    }
                    if *action.read() == "Register" {
                    div {  
                        id:"auth_div",
                        label {
                            "Your email:"
                        }
    
                        input {
                            r#type: "text",
                            value: "{email}",
                            oninput: move |evt| email.set(evt.value().clone()),
                            id: "inputbox"
                        }
                    }
                    }
                    div {  
                        id:"auth_div",
                        label {
                            "Your password:"
                        }
    
                        input {
                            r#type: "password",
                            value: "{password}",
                            oninput: move |evt| password.set(evt.value().clone()),
                            id: "inputbox"
                        }
                    }

                    button {
                        id:"bigbut",
                        r#type: "submit",
                        "{*action.read()}"


                    }

                    
                }
                
                
                //
                
            }
            
        }
    }

    button {  
        id:"bbt",

        onclick: move |_| {
                toggled.set(!toggled());
            },

        img {   src: img_src,
                width: "100%",    // ширина картинки (можно менять)
                height: "auto",  // сохраняет пропорции
                style: "
                    display: ablsolute;  
                    height: auto; 
                "        
         }


    }

    
}
}
