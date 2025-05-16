-- Create auth_config table
CREATE TABLE IF NOT EXISTS auth_config (
    id SERIAL PRIMARY KEY,
    session_url TEXT NOT NULL,
    login_redirect TEXT NOT NULL,
    cookie_name TEXT
);

-- Create routes table
CREATE TABLE IF NOT EXISTS routes (
    id SERIAL PRIMARY KEY,
    host TEXT NOT NULL,
    path TEXT NOT NULL,
    require JSONB NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Insert example routes
INSERT INTO routes (host, path, require)
VALUES
    ('app.example.com', '/admin/*', '{"roles": ["admin"]}'),
    ('app.example.com', '/api/users/*', '{"permissions": ["users:read"]}'),
    ('*.client.example.com', '/', '{"teams": [{"id": "client-team-id", "scopes": [{"resource_type": "client", "action": "access"}]}]}'),
    ('dashboard.example.com', '/reports/*', '{"scopes": [{"resource_type": "report", "action": "view"}]}')
ON CONFLICT DO NOTHING;