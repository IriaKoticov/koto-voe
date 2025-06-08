use serde::Deserialize;
use dioxus::prelude::*;
use serde::Serialize;
use gloo_net::http::Request;


#[derive(Deserialize, Debug)]
struct CatApi {
    id:i32,
    image: String,
}

#[server]
pub async fn get_resurse_img() -> Result<String, ServerFnError> {
    use reqwest;

    let response = reqwest::get("http://127.0.0.1:3000/api/cat")
        .await
        .unwrap()
        .json::<CatApi>()
        .await;
    Ok(response.unwrap().image)
}



