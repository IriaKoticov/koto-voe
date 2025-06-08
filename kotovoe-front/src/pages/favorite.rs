use dioxus::prelude::*;
use gloo_net::http::Request;

use crate::{components::card::{Card, CardInfo}, shared::SignalUpdate};
use log::info;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
struct AlbumUsrStr {

    id:i32,
    image:String,
}


struct ResAlbumUsrStr{

    data:Resource<String>,
    usr_grade:i32,

}

#[component]
pub fn Favorite() -> Element {

    let favcat = asset!("/assets/favor.png"); 


    let animation_class: Signal<Option<String>> = use_signal(|| None);

    let id_usr = use_context::<SignalUpdate>();

    let mut cat_mas:Signal<Vec<AlbumUsrStr>> = use_signal(|| Vec::new());

   let album =  use_resource(move || async move{

        
                let url = format!("http://127.0.0.1:3000/album/{}", id_usr.sig.to_string());
                info!("{}", url);

                let resp = Request::get(&url)
                    .send() 
                    .await
                    .expect("Ошибка при отправке запроса");

                let album:Vec<AlbumUsrStr> = resp
                    .json()
                    .await
                    .expect("Ошибка при парсинге JSON");

                cat_mas.set(album);
    });

    rsx!{
        div {
            id:"center_matrix",
        if !cat_mas().is_empty(){
        div {
            id:"matrix_cat",
        

            for i in cat_mas(){
    
                div{
    
                    id:"micro_card",
                    Card{
                        
                        data: i.image,
                        id: Some(i.id),
        
                    }
                }
    
            }
        }
        }  else {


            h1 {
                id:"text_is_empty",
                "-`♡´- ~Is empty~ -`♡´-"

            }

        }    
}
div {  

    img {  
        src: favcat,
        width: "40%",    // ширина картинки (можно менять)
        height: "auto",  // сохраняет пропорции
        style: "display: block; margin: 0% auto;"


    }

}
}
    
}
