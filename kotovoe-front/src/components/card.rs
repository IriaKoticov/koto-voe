use dioxus::prelude::*;
use log::info;

use web_sys::window;

#[derive(PartialEq, Clone, Props)]
pub struct CardInfo {
    pub data: String,
    pub id: Option<i32>
}

#[component]
pub fn Card(props: CardInfo) -> Element {

    let ke = props.id.map(|id| id.to_string());

    let ke_for_click = ke.clone();

    rsx! {
        div {
            
            id: "Card",
            key: "{ke.clone().unwrap_or_default()}",
                button {
                
                class: "delete-button",
                onclick: move |_| {

                    // info!("cat in loop wich id = {}",ke_for_click.clone().unwrap_or_else(|| "unknown".to_string()) );

                    let id = ke_for_click.clone().unwrap_or_else(|| "unknown".to_string());
                    
                    if id != "unknown"{
                    spawn(
                        async move {

                        let url = format!("http://localhost:3000/album/d/{}", id);
                        
                        let resp = reqwest::Client::new()
                        .delete(&url)
                        .send()
                        .await;
                        
                        info!("{:?}", resp);

                    });
                    };

                    if let Some(win) = window() {
                            let _ = win.location().reload();
                        }

                    },
                    "X"
                } 

            img {src: props.data }

        }
    }

}
