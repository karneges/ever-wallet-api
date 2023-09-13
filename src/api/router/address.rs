use axum::{
    routing::{get, post},
    Router,
};

use crate::api::controllers;

pub fn router() -> Router {
    Router::new()
        .route("/check", post(controllers::post_address_check))
        .route("/add", post(controllers::post_add_account_subscription))
        .route("/create", post(controllers::post_address_create))
        .route("/:address", get(controllers::get_address_balance))
        .route("/:address/info", get(controllers::get_address_info))
}
