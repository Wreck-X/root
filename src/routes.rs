use async_graphql::{http::GraphiQLSource, EmptySubscription, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::{Extension, Query as AxumQuery, State},
    middleware,
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use crate::auth::middleware::auth_middleware;
use crate::auth::oauth::GitHubOAuthService;
use crate::auth::AuthContext;
use crate::graphql::{Mutation, Query};

async fn graphql_handler(
    State(schema): State<Schema<Query, Mutation, EmptySubscription>>,
    Extension(auth_context): Extension<AuthContext>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema
        .execute(req.into_inner().data(auth_context))
        .await
        .into()
}

pub fn setup_router(
    schema: Schema<Query, Mutation, EmptySubscription>,
    cors: CorsLayer,
    is_dev: bool,
    pool: Arc<PgPool>,
) -> Router {
    let pool_for_middleware = pool.clone();

    let mut router = Router::new()
        .route("/", post(graphql_handler))
        .route("/auth/github", get(github_oauth_init))
        .route("/auth/github/callback", get(github_oauth_callback));

    if is_dev {
        tracing::info!("GraphiQL playground enabled at /graphiql");
        router = router.route("/graphiql", get(graphiql).post(graphql_handler));
    }

    router
        .layer(middleware::from_fn(move |req, next| {
            auth_middleware(pool_for_middleware.clone(), req, next)
        }))
        .layer(cors)
        .with_state(schema)
}

async fn graphiql() -> impl IntoResponse {
    Html(
        GraphiQLSource::build()
            .endpoint("/graphiql")
            .subscription_endpoint("/ws")
            .finish(),
    )
}

// OAuth handlers

/// Initiates GitHub OAuth flow
async fn github_oauth_init() -> Result<Redirect, String> {
    let oauth_service =
        GitHubOAuthService::new().map_err(|e| format!("Failed to initialize OAuth: {}", e))?;

    let (auth_url, _csrf_token) = oauth_service.get_authorization_url();

    // In production grade systems, we should be storing the token on the server side and returning
    // a session ID to the frontend. When the frontend calls the server for the authorization
    // code exchange, it should include the ID [eg: as a cookie] so that the backend can verify
    // that the state parameter from the authorization server matches it. This is to prevent CSRF
    // attacks.
    //
    // For now, we'll just redirect
    Ok(Redirect::temporary(&auth_url))
}

#[derive(Deserialize)]
struct OAuthCallbackQuery {
    code: String,
    #[allow(dead_code)]
    state: Option<String>,
}

#[derive(Serialize)]
struct OAuthCallbackResponse {
    success: bool,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    redirect_url: Option<String>,
}

/// Sample GitHub OAuth callback (used for testing purposes)
async fn github_oauth_callback(
    AxumQuery(query): AxumQuery<OAuthCallbackQuery>,
) -> Json<OAuthCallbackResponse> {
    // In a real implementation, you should:
    // 0. Handle this callback in the frontend
    // 1. Verify the CSRF token (state parameter)
    // 2. Call the githubOAuthCallback GraphQL mutation with the code from the URL parameter
    // 3. Store the session token and use it for authentication.

    // For now, we'll return a response that the frontend can handle
    Json(OAuthCallbackResponse {
        success: true,
        message: format!(
            "OAuth callback received. Use code '{}' with githubOAuthCallback mutation.",
            query.code
        ),
        redirect_url: Some(format!("/graphql?code={}", query.code)),
    })
}
