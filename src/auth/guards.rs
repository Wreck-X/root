use crate::auth::AuthContext;
use async_graphql::{Context, Error, Guard, Result};

/// Guard that requires any authentication
pub struct AuthGuard;

impl Guard for AuthGuard {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        tracing::info!("{:?}", ctx.data::<AuthContext>());
        let auth = ctx.data::<AuthContext>().map_err(|_| {
            Error::new("Authentication context not found. This is an internal server error.")
        })?;

        if auth.is_authenticated() {
            Ok(())
        } else {
            Err(Error::new(
                "Authentication required to access this resource.",
            ))
        }
    }
}

/// Guard that requires admin role
pub struct AdminGuard;

impl Guard for AdminGuard {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        let auth = ctx.data::<AuthContext>().map_err(|_| {
            Error::new("Authentication context not found. This is an internal server error.")
        })?;

        if auth.is_admin() {
            Ok(())
        } else {
            Err(Error::new("Admin privileges required for this operation"))
        }
    }
}

/// Guard that requires either admin or bot role
pub struct AdminOrBotGuard;

impl Guard for AdminOrBotGuard {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        let auth = ctx.data::<AuthContext>().map_err(|_| {
            Error::new("Authentication context not found. This is an internal server error.")
        })?;

        if auth.is_bot() || auth.is_admin() {
            Ok(())
        } else {
            Err(Error::new(
                "Admin or Bot privileges required for this operation",
            ))
        }
    }
}
