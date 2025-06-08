pub mod home;
pub mod topcat;
pub mod favorite;
pub mod auth;

use dioxus::prelude::*;
use crate::components::navbar::Navbar;
use crate::pages::home::Home;
use crate::pages::topcat::TopCat;
use crate::pages::favorite::Favorite;
use crate::pages::auth::Auth;



#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Navbar)]
    #[route("/")]
    Home {},
    #[route("/top_cat/")]
    TopCat {},
    #[route("/favorite/")]
    Favorite {},

}
