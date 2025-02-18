use std::sync::Arc;

use axum::async_trait;
use axum::body::{boxed, Body, Full};
use axum::extract::{FromRequest, OriginalUri, RequestParts};
use axum::http::Request;
use axum::middleware::Next;
use axum::response::IntoResponse;
use http::{Method, StatusCode};

use crate::models::*;
use crate::services::*;

pub async fn verify_auth(
    req: Request<Body>,
    next: Next<Body>,
    auth_service: Arc<AuthService>,
) -> impl IntoResponse {
    match check_api_key(req, auth_service).await {
        Ok(req) => next.run(req).await,
        Err(err) => {
            log::error!("Failed to check auth. Err: {:?}", &err);
            Rejection("Failed to authorize".to_string(), StatusCode::UNAUTHORIZED).into_response()
        }
    }
}

async fn check_api_key(
    req: Request<Body>,
    auth_service: Arc<AuthService>,
) -> anyhow::Result<Request<Body>> {
    let api_key_opt = req.headers().get("api-key");

    let api_key =  match &api_key_opt {
        Some(key) => key.clone(),
        None => anyhow::bail!("Api key doesn't provided")
    }.to_str().unwrap();
    let Key {service_id, ..} = auth_service.get_key(api_key).await?;
    let mut parts = RequestParts::new(req);



    // Forward service id to request handler
    parts.extensions_mut().insert(IdExtractor(service_id));

    Ok(Request::from_request(&mut parts).await.expect("can't fail"))
}

pub struct IdExtractor(pub ServiceId);

#[async_trait]
impl<B> FromRequest<B> for IdExtractor
where
    B: Send, // required by `async_trait`
{
    type Rejection = Rejection;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let extensions = req.extensions();

        let id: Option<&IdExtractor> = extensions.get();
        match id {
            Some(service_id) => Ok(IdExtractor(service_id.0)),
            None => Err(Rejection(
                "Service id not found".to_string(),
                StatusCode::UNAUTHORIZED,
            )),
        }
    }
}

pub struct Rejection(String, StatusCode);

impl IntoResponse for Rejection {
    fn into_response(self) -> axum::response::Response {
        (self.1, self.0).into_response()
    }
}
