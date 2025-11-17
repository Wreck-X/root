use crate::auth::api_key::ApiKeyService;
use crate::auth::guards::AdminGuard;
use crate::auth::oauth::GitHubOAuthService;
use crate::auth::session::SessionService;
use crate::auth::AuthContext;
use crate::models::auth::{ApiKeyResponse, AuthResponse, GitHubUser, Role};
use crate::models::member::Member;
use async_graphql::{Context, Object, Result};
use chrono_tz::Asia::Kolkata;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Default)]
pub struct AuthMutations;

#[Object]
impl AuthMutations {
    /// Complete GitHub OAuth flow and create session (for registration or login)
    #[graphql(name = "githubOAuthCallback")]
    async fn github_oauth_callback(&self, ctx: &Context<'_>, code: String) -> Result<AuthResponse> {
        let pool = ctx.data::<Arc<PgPool>>().expect("Pool must be in context.");

        let oauth_service = GitHubOAuthService::new()
            .map_err(|e| format!("Failed to initialize OAuth service: {}", e))?;

        let github_user = oauth_service
            .complete_oauth_flow(code)
            .await
            .map_err(|e| format!("OAuth flow failed: {}", e))?;

        // Check if member already exists
        let existing_member = sqlx::query_as::<_, Member>(
            "SELECT member_id, roll_no, name, email, sex, year, hostel, mac_address, discord_id,
             group_id, track, github_user, role, created_at, updated_at
             FROM Member
             WHERE github_user = $1",
        )
        .bind(&github_user.github_username)
        .fetch_optional(pool.as_ref())
        .await?;

        let member = if let Some(member) = existing_member {
            // Member exists - login
            member
        } else {
            // Member doesn't exist - register
            Self::register_member(pool.as_ref(), github_user).await?
        };

        // Create session
        let session_token = SessionService::create_session(pool.as_ref(), member.member_id)
            .await
            .map_err(|e| format!("Failed to create session: {}", e))?;

        Ok(AuthResponse {
            member,
            session_token,
        })
    }

    /// Logout - invalidate session
    #[graphql(name = "logout")]
    async fn logout(&self, ctx: &Context<'_>, session_token: String) -> Result<bool> {
        let pool = ctx.data::<Arc<PgPool>>().expect("Pool must be in context.");
        let auth = ctx
            .data::<AuthContext>()
            .expect("AuthContext must be in context.");

        if auth.is_authenticated() {
            SessionService::delete_session_by_token(pool.as_ref(), &session_token)
                .await
                .map_err(|e| format!("Failed to logout: {}", e))?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Create a new bot with API key (Admin only)
    #[graphql(name = "createBot", guard = "AdminGuard")]
    async fn create_bot(&self, ctx: &Context<'_>, name: String) -> Result<ApiKeyResponse> {
        let pool = ctx.data::<Arc<PgPool>>().expect("Pool must be in context.");
        let auth = ctx
            .data::<AuthContext>()
            .expect("AuthContext must be in context.");

        let admin_member = auth
            .user
            .as_ref()
            .ok_or("Admin member not found in context")?;

        // Create API key
        let api_key = ApiKeyService::create_api_key(pool.as_ref(), name, admin_member.member_id)
            .await
            .map_err(|e| format!("Failed to create bot: {}", e))?;

        Ok(ApiKeyResponse { api_key })
    }

    /// Delete a bot (Admin only)
    #[graphql(name = "deleteBot", guard = "AdminGuard")]
    async fn delete_bot(&self, ctx: &Context<'_>, api_key_id: i32) -> Result<bool> {
        let pool = ctx.data::<Arc<PgPool>>().expect("Pool must be in context.");

        ApiKeyService::delete_api_key(pool.as_ref(), api_key_id)
            .await
            .map_err(|e| format!("Failed to delete bot: {}", e))?;

        Ok(true)
    }
}

impl AuthMutations {
    async fn register_member(pool: &PgPool, github_user: GitHubUser) -> Result<Member> {
        let now = chrono::Utc::now().with_timezone(&Kolkata);

        let member = sqlx::query_as::<_, Member>(
            "INSERT INTO Member (name, email, github_user, role, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING member_id, roll_no, name, email, sex, year, hostel, mac_address, discord_id,
             group_id, track, github_user, role, created_at, updated_at",
        )
        .bind(github_user.name)
        .bind(github_user.email)
        .bind(github_user.github_username)
        .bind(Role::Member)
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await?;

        Ok(member)
    }
}
