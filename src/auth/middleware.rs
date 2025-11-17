use crate::auth::api_key::ApiKeyService;
use crate::auth::session::SessionService;
use crate::auth::AuthContext;
use axum::{
    extract::Request,
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::Response,
};
use sqlx::PgPool;
use std::sync::Arc;

pub async fn auth_middleware(
    pool: Arc<PgPool>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    let member = if let Some(auth_value) = auth_header {
        if let Some(token) = auth_value.strip_prefix("Bearer ").or(Some(auth_value)) {
            let session_member = SessionService::validate_session(&pool, token)
                .await
                .ok()
                .flatten();

            if session_member.is_some() {
                session_member
            } else {
                // If session returned None, try API key
                ApiKeyService::validate_api_key(&pool, token)
                    .await
                    .ok()
                    .flatten()
            }
        } else {
            None
        }
    } else {
        None
    };

    // Inject auth context into request extensions
    request.extensions_mut().insert(AuthContext::new(member));

    Ok(next.run(request).await)
}
