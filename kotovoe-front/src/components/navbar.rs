use dioxus::prelude::*;
use crate::pages::Route;


#[component]
pub fn Navbar() -> Element {
    rsx! {
        nav {
            id: "navbar",
            div {
                id: "left_nav_div",
                Link { to: Route::TopCat {}, "Cats in top" }
            }
            div {
                id: "middle_nav_div",
                Link { to: Route::Home {}, "Home" }
            }
            div {
                id: "right_nav_div",
                Link { to: Route::Favorite {}, "My like cat" }
            }
        }
        Outlet::<Route> {}
    }
}

