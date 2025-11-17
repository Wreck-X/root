# Authentication System Documentation

This document describes the authentication and authorization system for Root backend.

| Section | Description |
|--------|-------------|
| [Overview](#overview) | Summary of supported authentication methods |
| [Roles and Permissions](#roles-and-permissions) | Roles (Admin, Member, Bot) and protected mutations |
| [Setup](#setup) | How to configure OAuth, env vars, and database migration |
| [OAuth Flow](#usage) | Full OAuth login flow and response structure |
| [Bot Management](#bot-management) | Bot creation and API key handling |
| [Role Based Access Control (RBAC)](#permission-checking-in-code) | GraphQL guards and access control |
| [Troubleshooting](#troubleshooting) | Common issues and how to fix them |
| [API Reference](#api-reference) | GraphQL mutations and their inputs/outputs |
| [Example](#example) | Complete OAuth authentication flow example |
| [Architecture Notes](#architecture-notes) | Middleware flow, bot member structure, key management |



## Overview

The authentication system supports three types of authentication:
1. **GitHub OAuth** - For human members who are part of the amfoss GitHub organization
2. **API Keys** - For headless bots and automated services
3. **Session Tokens** - For maintaining logged-in state after OAuth

## Roles and Permissions

### Roles

- **Admin**: Full system access, can create/manage bots, access all mutations and queries
- **Member**: Standard user permissions, authenticated via GitHub OAuth, limited access
- **Bot**: Service accounts with API key authentication, can access protected mutations

Note that unauthenticated users have essentially no access to the system.

### Protected Mutations

The following mutations require Admin or Bot role:
- `createMember`
- `markAttendance`
- `markStatusUpdate`
- `createStatusBreak`

Regular Members cannot access these mutations.

## Setup

### 1. GitHub OAuth Application

Create a GitHub OAuth application at: https://github.com/settings/developers

**Settings:**
- Application name: Root Backend (or your choice)
- Homepage URL: `http://localhost:3000` (or your domain)
- Authorization callback URL: `http://localhost:5000/auth/github/callback`

After creating, note down the **Client ID** and **Client Secret**.

### 2. Environment Variables

Add the following to your `.env` file:

```bash
# GitHub OAuth credentials
GITHUB_CLIENT_ID=your_client_id_here
GITHUB_CLIENT_SECRET=your_client_secret_here
GITHUB_REDIRECT_URL=http://localhost:5000/auth/github/callback
GITHUB_ORG_NAME=amfoss  # Organization that users must be part of
```

### 3. Database Migration

Run the authentication system migration:

```bash
sqlx migrate run
```

This creates the following tables:
- `Member` - Stores authenticated members with their details
- `Sessions` - Stores session tokens
- `ApiKeys` - Stores hashed API keys for bots

### 4. Making Your First Admin

After setting up the system, you need to create the first admin user manually:

1. Register via GitHub OAuth (see below)
2. Connect to the database:
   ```bash
   psql $ROOT_DB_URL
   ```
3. Promote your user to admin:
   ```sql
   UPDATE Member SET role = 'Admin' WHERE email = 'your@email.com';
   ```
4. Verify:
   ```sql
   SELECT member_id, name, email, role FROM Member WHERE email = 'your@email.com';
   ```

Now you can create bots and manage the system!
## Usage

### Member Registration & Login (GitHub OAuth)


1. User visits: `http://localhost:5000/auth/github`
2. Gets redirected to GitHub for authorization
3. After authorization, GitHub redirects to the frontend at: `/auth/github/callback?code=...`
4. Frontend receives the OAuth code
5. Frontend calls GraphQL mutation with the code and the backend returns the session token for the  user:

```graphql
mutation {
  githubOAuthCallback(code: "oauth_code_here") {
    member {
      memberId
      name
      email
      role
      githubUser
    }
    sessionToken
  }
}
```

**Response:**
```json
{
  "data": {
    "githubOAuthCallback": {
      "member": {
        "memberId": 1,
        "githubUser": "johndoe",
        "name": "John Doe",
        "email": "john@example.com",
        "role": "Member"
      },
      "sessionToken": "abc123...xyz789"
    }
  }
}
```

6. Frontend stores the session token (in localStorage, cookie, etc.)
7. Frontend includes token in subsequent requests via Authorization header


**Important:**
- First time users are automatically registered
- Users must be members of the amfoss GitHub organization
- Non-members receive an error

### Making Authenticated Requests

Include the session token in the Authorization header:

```
Authorization: Bearer <session_token>
```

Example with curl:
```bash
curl -X POST http://localhost:3000/ \
  -H "Authorization: Bearer abc123...xyz789" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "{ member(memberId: 1) { name email } }"
  }'
```

### Logout

```graphql
mutation {
  logout(sessionToken: "your_session_token_here")
}
```

This invalidates the specified session for the current user.

**Returns:** `true` if successful, `false` if not authenticated

## Bot Management
### Creating Bots (Admin Only)

Admins can create bot accounts with API keys:

```graphql
mutation {
  createBot(name: "Presence Bot")
}
```

**Response:**
```json
{
  "data": {
    "createBot": {
      "apiKey": "root_wnTK5uRq8FECFSvSC8OVZ8h0SSJefTMlvGWJmsS4"
    }
  }
}
```

**⚠️ Important:** The API key is only returned ONCE. Store it securely!

### Using API Keys (Bots)

Bots use API keys instead of session tokens. Include the API key in the `Authorization` header in the same format as before.

### Deleting Bots (Admin Only)

```graphql
mutation {
  deleteBot(apiKeyId: 1)
}
```

**Returns:** `true` if successful


## Permission Checking in Code

### In GraphQL Resolvers

Protected mutations use guards:

```rust
use crate::auth::guards::{AdminGuard, AdminOrBotGuard, AuthGuard};

#[Object]
impl SomeMutations {
    #[graphql(name = "protectedMutation", guard = "AdminOrBotGuard")]
    async fn protected_mutation(&self, ctx: &Context<'_>) -> Result<String> {
        // Only admins and bots can reach here
        Ok("Success".to_string())
    }

    #[graphql(name = "memberOnlyMutation", guard = "AuthGuard")]
    async fn member_only_mutation(&self, ctx: &Context<'_>) -> Result<String> {
        // Any authenticated user can reach here
        Ok("Success".to_string())
    }

    #[graphql(name = "adminOnlyMutation", guard = "AdminGuard")]
    async fn admin_only_mutation(&self, ctx: &Context<'_>) -> Result<String> {
        // Only admins can reach here
        Ok("Success".to_string())
    }
}
```

### Available Guards

- `AuthGuard` - Requires any authenticated user (Member, Admin, or Bot)
- `AdminGuard` - Requires Admin role
- `AdminOrBotGuard` - Requires Admin or Bot role

## Troubleshooting

### "Authentication context not found" Error

The auth middleware is not properly configured. Ensure:

- That the `Authorization:` header is present in the HTTP Request.

### "User is not a member of the amfoss organization"

The GitHub account is not part of the specified organization. Either:
1. Add the user to the organization
2. Change `GITHUB_ORG_NAME` in .env (for testing)

## API Reference

### GraphQL Mutations

#### `githubOAuthCallback(code: String!): AuthResponse!`

Complete OAuth flow and create session.

**Input:**
- `code`: OAuth authorization code from GitHub

**Returns:**
- `member`: Member information including memberId, name, email, role, githubUser
- `sessionToken`: Session token for subsequent requests

---

#### `logout(sessionToken: String!): Boolean!`

Invalidate the specified session for current user.

**Input:**
- `sessionToken`: The session token to invalidate

**Returns:** `true` if successful, `false` if not authenticated

---

#### `createBot(name: String!): String!` 🔒 Admin only

Create a new bot with API key.

**Input:**
- `name`: Bot name/description

**Returns:** The API key string (only shown once!)

---

#### `deleteBot(apiKeyId: Int!): Boolean!` 🔒 Admin only

Delete a bot and revoke its API key.

**Input:**
- `apiKeyId`: ID of the API key to delete

**Returns:** `true` if successful

## Example

### Complete Member Authentication Flow

```javascript
// 1. Redirect to GitHub OAuth
window.location.href = 'http://localhost:3000/auth/github';

// 2. After callback, extract code from URL
const urlParams = new URLSearchParams(window.location.search);
const code = urlParams.get('code');

// 3. Call GraphQL mutation
const response = await fetch('http://localhost:3000/', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    query: `
      mutation($code: String!) {
        githubOAuthCallback(code: $code) {
          member { memberId name email role }
          sessionToken
        }
      }
    `,
    variables: { code }
  })
});

const { data } = await response.json();
const sessionToken = data.githubOAuthCallback.sessionToken;

// 4. Store token
localStorage.setItem('sessionToken', sessionToken);

// 5. Use token in subsequent requests
fetch('http://localhost:3000/', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
    'Authorization': `Bearer ${sessionToken}`
  },
  body: JSON.stringify({
    query: '{ member(memberId: 1) { name } }'
  })
});
```
## Architecture Notes

### Authentication Flow

1. **Request arrives** → `auth_middleware` extracts Authorization header
2. **Token validation** → Tries session token first, then API key
3. **Member lookup** → Returns associated Member or None
4. **Context injection** → AuthContext with Member is added to request extensions
5. **GraphQL execution** → Guards check AuthContext for permissions
6. **Response** → Returns data or permission error

### Bot Members

Bots are represented as synthetic Member objects with:
- Negative member_id (to distinguish from real members)
- Role set to Bot
- Email format: `bot-{api_key_id}@internal.amfoss.in`
- Name from API key name


### API Key Management

- API keys are prefixed with `root_`
- Keys are hashed with bcrypt (cost factor 12) before storage
- Last used timestamp is updated on each validation
- Keys can only be deleted by admins
