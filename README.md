# AuthGate

AuthGate is a standalone Traefik forwardAuth middleware that authenticates incoming requests using a configurable external session endpoint and authorizes access to protected routes based on roles, permissions, and team-based scopes.

## Features

- **Authentication**: Validates user sessions via an external session endpoint
- **Authorization**: Controls access based on roles, permissions, scopes, and team membership
- **Route Matching**: Supports both hostname (including wildcard subdomains) and path prefix matching
- **Simple Configuration**: Single JSON file for all settings
- **Traefik Integration**: Works as a forwardAuth middleware with Traefik

## Configuration

AuthGate is configured through a single JSON file (`authgate.json`). The configuration includes:

- **Authentication settings**: Session endpoint URL and login redirect URL
- **Route definitions**: Host and path patterns with authorization requirements
- **Cookie configuration**: Name of the session cookie (defaults to "session")

Example configuration:

```json
{
  "auth": {
    "session_url": "https://auth.example.com/session",
    "login_redirect": "https://auth.example.com/login"
  },
  "cookie_name": "session",
  "routes": [
    {
      "host": "app.example.com",
      "path": "/admin/*",
      "require": {
        "roles": ["admin"]
      }
    },
    {
      "host": "*.client.example.com",
      "path": "/",
      "require": {
        "teams": [
          {
            "id": "client-team-id",
            "scopes": [
              {
                "resource_type": "client",
                "action": "access"
              }
            ]
          }
        ]
      }
    }
  ]
}
```

## Route Matching

- **Host matching**: Supports exact matches and wildcard subdomains (e.g., `*.client.example.com`)
- **Path matching**: Supports exact matches and prefix matching with wildcards (e.g., `/api/*`)

## Authorization Rules

Each route can specify one or more of the following authorization requirements:

- **Roles**: User must have at least one of the specified roles
- **Permissions**: User must have at least one of the specified permissions
- **Scopes**: User must have all the specified scopes
- **Teams**: User must be a member of at least one of the specified teams, and if scopes are specified for a team, the user must have those scopes within that team

## Session Endpoint

The session endpoint should return a JSON response with the following structure:

```json
{
  "user": {
    "id": "user-id",
    "email": "user@example.com",
    "roles": ["user", "admin"],
    "permissions": ["users:read", "users:write"],
    "teams": [
      {
        "id": "team-id",
        "name": "Team Name",
        "is_owner": true,
        "scopes": [
          {
            "resource_type": "client",
            "resource_id": "client-id",
            "action": "access"
          }
        ]
      }
    ]
  },
  "tenant_id": "tenant-id",
  "authority": "example.com"
}
```

## Running with Docker

```bash
docker run -p 4181:4181 -v /path/to/authgate.json:/app/authgate.json authgate
```

## Environment Variables

### Core Configuration
- `PORT`: Port to listen on (default: `4181`)
- `RUST_LOG`: Logging level (default: `info`)
- `AUTHGATE_ENABLE_ADMIN_API`: Enable the Admin API (default: `false`)
- `AUTHGATE_ADMIN_TOKEN`: Bearer token for Admin API authentication
- `AUTHGATE_SESSION_COOKIE`: Name of the session cookie for session-based authentication (default: same as cookie_name in config)
- `AUTHGATE_ADMIN_SESSION_ROLES`: Comma-separated list of roles allowed to access the Admin API via session authentication

### Configuration Providers
AuthGate supports multiple configuration backends:

- `AUTHGATE_CONFIG_BACKEND`: Configuration backend to use, either `json` or `postgres` (default: `json`)

#### JSON File Provider
When using the JSON file provider (`AUTHGATE_CONFIG_BACKEND=json`):

- `AUTHGATE_CONFIG`: Path to the configuration file (default: `authgate.json`)

#### PostgreSQL Provider
When using the PostgreSQL provider (`AUTHGATE_CONFIG_BACKEND=postgres`):

- `DATABASE_URL`: PostgreSQL connection URL (default: `postgres://postgres:postgres@localhost/authgate`)

The PostgreSQL schema requires two tables:

1. `auth_config` - Contains authentication settings:
   - `session_url` - URL for session validation
   - `login_redirect` - URL for login redirection
   - `cookie_name` - Optional name of the session cookie

2. `routes` - Contains route definitions:
   - `host` - Hostname pattern (e.g., `app.example.com` or `*.client.example.com`)
   - `path` - Path pattern (e.g., `/admin/*`)
   - `require` - JSONB column containing authorization requirements

Example SQL schema:

```sql
CREATE TABLE auth_config (
    id SERIAL PRIMARY KEY,
    session_url TEXT NOT NULL,
    login_redirect TEXT NOT NULL,
    cookie_name TEXT
);

CREATE TABLE routes (
    id SERIAL PRIMARY KEY,
    host TEXT NOT NULL,
    path TEXT NOT NULL,
    require JSONB NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
```

Example route entries:

```sql
INSERT INTO routes (host, path, require)
VALUES
    ('app.example.com', '/admin/*', '{"roles": ["admin"]}'),
    ('app.example.com', '/api/users/*', '{"permissions": ["users:read"]}'),
    ('*.client.example.com', '/', '{"teams": [{"id": "client-team-id", "scopes": [{"resource_type": "client", "action": "access"}]}]}');
```

A setup script is provided in `scripts/setup_postgres.sh` to initialize the database.

### Admin API

AuthGate includes an Admin API that allows you to manage routes and authentication settings programmatically. The Admin API is disabled by default and can only be enabled when using the PostgreSQL configuration backend.

To enable the Admin API:

1. Set `AUTHGATE_ENABLE_ADMIN_API=true` in your environment
2. Use the PostgreSQL configuration backend (`AUTHGATE_CONFIG_BACKEND=postgres`)

When enabled, the Admin API is available at the `/admin` endpoint. It provides:

- `/admin/health` - Health check endpoint
- `/admin/routes` - Routes management API:
  - `GET /admin/routes` - List all routes
  - `GET /admin/routes/:id` - Get a specific route by ID
  - `POST /admin/routes` - Create a new route
  - `PUT /admin/routes/:id` - Update an existing route
  - `DELETE /admin/routes/:id` - Delete a route

If the Admin API is disabled or you're using the JSON file configuration backend, all Admin API endpoints will return a 403 Forbidden response.

#### Admin API Authentication

The Admin API supports two authentication methods:

##### 1. Bearer Token Authentication

For programmatic access (CLI tools, scripts, etc.):

1. Set the `AUTHGATE_ADMIN_TOKEN` environment variable to a secure random string
2. Include the token in the `Authorization` header of your requests:

```
Authorization: Bearer your-admin-token
```

If the token is missing or invalid, the API will respond with a 401 Unauthorized status and include a `WWW-Authenticate: Bearer` header in the response.

Example request:

```bash
curl -H "Authorization: Bearer your-admin-token" http://localhost:4181/admin/health
```

##### 2. Session Cookie Authentication

For browser-based access by authenticated users:

1. Set the `AUTHGATE_SESSION_COOKIE` environment variable to the name of your session cookie (defaults to the same as cookie_name in config)
2. Set the `AUTHGATE_ADMIN_SESSION_ROLES` environment variable to a comma-separated list of roles that are allowed to access the Admin API
3. Users with a valid session cookie and at least one of the allowed roles can access the Admin API

Example configuration:

```
AUTHGATE_SESSION_COOKIE=session
AUTHGATE_ADMIN_SESSION_ROLES=admin,superuser
```

If the session is invalid or the user doesn't have any of the allowed roles, the API will respond with a 403 Forbidden status.

### Session Caching
AuthGate supports caching session data to reduce load on the authentication service:

- `AUTHGATE_CACHE_ENABLED`: Enable or disable session caching (default: `true`)
- `AUTHGATE_CACHE_BACKEND`: Cache backend to use, either `memory` or `redis` (default: `memory`)
- `AUTHGATE_REDIS_URL`: Redis connection URL when using the Redis backend (default: `redis://127.0.0.1:6379`)

#### Caching Behavior

When caching is enabled, AuthGate will:

1. Extract the expiration time (`exp` claim) from the JWT session token
2. Cache the session data with a TTL matching the JWT expiration
3. Use the cached session for subsequent requests until it expires
4. Fall back to a 5-minute TTL if the JWT expiration cannot be extracted

This ensures that cached sessions are automatically invalidated when the JWT expires, maintaining security while reducing load on your authentication service.

## Traefik Configuration

Example Traefik configuration to use AuthGate as a forwardAuth middleware:

```yaml
http:
  middlewares:
    authgate:
      forwardAuth:
        address: "http://authgate:4181/"
        authResponseHeaders:
          - "X-Auth-User-Id"
          - "X-Auth-User-Email"
          - "X-Auth-User-Roles"
          - "X-Auth-User-Permissions"

  routers:
    my-app:
      rule: "Host(`app.example.com`)"
      service: my-app-service
      middlewares:
        - authgate
```

### User Information Headers

When a request is authorized, AuthGate forwards the following headers to the upstream service:

- `X-Auth-User-Id`: The authenticated user's ID
- `X-Auth-User-Email`: The authenticated user's email address
- `X-Auth-User-Roles`: Comma-separated list of the user's roles
- `X-Auth-User-Permissions`: Comma-separated list of the user's permissions

## Building from Source

```bash
cargo build --release
```

## License

MIT