# AuthGate Examples

This directory contains examples of how to use AuthGate in different environments.

## Traefik Integration

The `traefik-docker-compose.yml` file demonstrates how to integrate AuthGate with Traefik as a forward authentication middleware.

### Usage

1. Build the AuthGate Docker image:
   ```bash
   cd ..
   docker build -t authgate:latest .
   ```

2. Start the example stack:
   ```bash
   cd examples
   docker-compose -f traefik-docker-compose.yml up -d
   ```

3. Access the protected application at http://app.example.com (you'll need to add this to your hosts file)

### Configuration

The example demonstrates two configuration backends:

#### JSON File Configuration

The `authgate-json` service uses the `authgate.json` file in this directory, which defines:

- Authentication settings (session URL and login redirect)
- Protected routes with different authorization requirements:
  - Admin routes requiring the "admin" role
  - API routes requiring the "users:read" permission
  - Client subdomains requiring team-based access

#### PostgreSQL Configuration

The `authgate-postgres` service uses a PostgreSQL database for configuration:

- The database is initialized with the SQL migrations in the `migrations` directory
- It contains the same configuration as the JSON file, but stored in database tables
- The `auth_config` table contains authentication settings
- The `routes` table contains route definitions with JSONB for authorization requirements

#### Admin API

The PostgreSQL-based service also enables the Admin API:

- The Admin API is available at `http://auth-postgres.example.com/admin`
- It supports two authentication methods:
  1. Bearer token authentication via the `AUTHGATE_ADMIN_TOKEN` environment variable
  2. Session cookie authentication for users with specific roles

**Bearer Token Authentication:**
```bash
curl -H "Authorization: Bearer your-secure-admin-token" http://auth-postgres.example.com/admin/health
```

**Session Cookie Authentication:**
- Users with a valid session cookie and the `admin` or `superuser` role can access the Admin API
- This is configured via the `AUTHGATE_SESSION_COOKIE` and `AUTHGATE_ADMIN_SESSION_ROLES` environment variables

**Routes Management API:**

List all routes:
```bash
curl -H "Authorization: Bearer your-secure-admin-token" http://auth-postgres.example.com/admin/routes
```

Get a specific route:
```bash
curl -H "Authorization: Bearer your-secure-admin-token" http://auth-postgres.example.com/admin/routes/route-id
```

Create a new route:
```bash
curl -X POST -H "Authorization: Bearer your-secure-admin-token" \
  -H "Content-Type: application/json" \
  -d '{"host":"api.example.com","path":"/api","require":{"roles":["admin"]}}' \
  http://auth-postgres.example.com/admin/routes
```

Update a route:
```bash
curl -X PUT -H "Authorization: Bearer your-secure-admin-token" \
  -H "Content-Type: application/json" \
  -d '{"host":"api.example.com","path":"/api/v2","require":{"roles":["admin"]}}' \
  http://auth-postgres.example.com/admin/routes/route-id
```

Delete a route:
```bash
curl -X DELETE -H "Authorization: Bearer your-secure-admin-token" \
  http://auth-postgres.example.com/admin/routes/route-id
```

### Session Caching

The example includes Redis for session caching, which reduces load on your authentication service. The configuration includes:

- A Redis container for storing session data
- Environment variables to enable and configure caching:
  - `AUTHGATE_CACHE_ENABLED=true` - Enables session caching
  - `AUTHGATE_CACHE_BACKEND=redis` - Uses Redis as the cache backend
  - `AUTHGATE_REDIS_URL=redis://redis:6379` - Connects to the Redis container

You can switch to in-memory caching by changing `AUTHGATE_CACHE_BACKEND=memory`, or disable caching entirely with `AUTHGATE_CACHE_ENABLED=false`.

## Custom Integration

You can adapt these examples to your specific needs by:

1. Modifying the `authgate.json` configuration
2. Adjusting the Traefik labels to match your routing requirements
3. Replacing the example protected application with your actual services