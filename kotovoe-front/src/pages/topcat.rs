use dioxus::{html::audio::src, prelude::*};
use gloo_net::http::Request;

use crate::{components::card::{Card, CardInfo}, shared::SignalUpdate};
use log::info;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
struct CatGradeAll_to_front {
    image: String,
    grade_avg: f64,
}

fn get_cat_medal(n:usize) -> String{

    match n {
    0 => {return "🥇".to_string()},
    1 => {return "🥈".to_string()},
    2 => {return "🥉".to_string()},
    _ => {return "".to_string()},
    }

}


#[component]
pub fn TopCat() -> Element {

    let hapcat = asset!("/assets/happy_cat.png"); 

    let mut top_cat_mas:Signal<Vec<CatGradeAll_to_front>> = use_signal(|| Vec::new());


    let top =  use_resource(move || async move{

        
                let resp = Request::get("http://127.0.0.1:3000/tops")
                    .send() 
                    .await
                    .expect("Ошибка при отправке запроса");

                let cat_vec:Vec<CatGradeAll_to_front> = resp
                    .json()
                    .await
                    .expect("Ошибка при парсинге JSON");

                top_cat_mas.set(cat_vec);
    });


    rsx! {
        div {
            id: "top-cats-page",

            h1 { "🏆 Топ Котиков 🏆" }

            div {
                id: "top-cats-grid",
                for (n,i) in top_cat_mas().iter().enumerate() {

                        div {
                            id: "cats-scard",
                            div {
                                id: "medal",
                                "{get_cat_medal(n)}"
                            
                            
                            }
                            Card {
    
                                data: i.image.clone(),
                                id: None,
    
                            }

                        }
                    }                
                }
                div {  
                    id:"hap_cat_div",

                    img { src : hapcat,
                        width: "25%",    // ширина картинки (можно менять)
                        height: "auto",  // сохраняет пропорции
                        style: "display: block; margin: 10px auto;"
                    
                     }


                }
            }
        }
}
