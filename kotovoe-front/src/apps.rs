use dioxus::prelude::*;
use crate::components::Route;


const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

#[component]
pub fn App() -> Element {
    static BACKGROUND: Asset = asset!("/assets/fon.png");
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }

        body {
            style: "background-image: url({BACKGROUND});
                    background-size: cover;      
                    background-position: center center;  
                    background-repeat: no-repeat;",
            
            Router::<Route> {}
        }
    }
}
