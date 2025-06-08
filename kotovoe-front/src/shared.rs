use dioxus::prelude::Signal;
use serde::{Deserialize,Serialize};

#[derive(Clone, PartialEq,Deserialize,Serialize)]
pub struct Usr {
    pub id:i32
}

#[derive(serde::Deserialize, Debug)]
pub struct LoginResponse {
    pub user_id: i32,
}

#[derive(Clone, PartialEq)]
pub struct SignalUsr {
    pub user: Usr,
    pub flag: bool,
}


#[derive(Clone, PartialEq)]
pub struct SignalUpdate{pub sig:Signal<i32>}
