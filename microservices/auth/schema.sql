-- consolidated schema for the Rust Auth implementation
-- this file contains the create statements for user table on the database

-- Users Table
CREATE TABLE IF NOT EXISTS users (
    id BIGSERIAL PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    password VARCHAR(255) NOT NULL, -- Stores Argon2 hashed password
    full_name VARCHAR(511) NOT NULL, -- Computed from first_name + last_name
    profile_image VARCHAR(512),
    access_token VARCHAR(1024),
    refresh_token VARCHAR(1024),
    one_time_password_token VARCHAR(1024),
    status VARCHAR(10) NOT NULL DEFAULT 'offline',
    last_seen VARCHAR(20),
    user_type VARCHAR(16) NOT NULL DEFAULT 'user' CHECK (user_type IN ('user', 'admin')),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    is_logged_out BOOLEAN NOT NULL DEFAULT FALSE,
    phone_number VARCHAR(20) UNIQUE,
    country VARCHAR(100),
    country_code VARCHAR(10),
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Index for users.email
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_user_type ON users(user_type);

CREATE TABLE IF NOT EXISTS roles (
    id UUID PRIMARY KEY,
    name VARCHAR(64) NOT NULL UNIQUE,
    description TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS permissions (
    id UUID PRIMARY KEY,
    name VARCHAR(128) NOT NULL UNIQUE,
    description TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS user_roles (
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, role_id)
);

CREATE TABLE IF NOT EXISTS role_permissions (
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_id UUID NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    PRIMARY KEY (role_id, permission_id)
);

CREATE INDEX IF NOT EXISTS idx_user_roles_user_id ON user_roles(user_id);
CREATE INDEX IF NOT EXISTS idx_user_roles_role_id ON user_roles(role_id);
CREATE INDEX IF NOT EXISTS idx_role_permissions_role_id ON role_permissions(role_id);
CREATE INDEX IF NOT EXISTS idx_role_permissions_permission_id ON role_permissions(permission_id);

INSERT INTO roles (id, name, description)
VALUES
    ('00000000-0000-0000-0000-000000000001', 'user', 'Default application user'),
    ('00000000-0000-0000-0000-000000000002', 'admin', 'Application administrator')
ON CONFLICT (name) DO NOTHING;

CREATE TABLE IF NOT EXISTS sessions (
    id UUID PRIMARY KEY,
    creation_order BIGSERIAL, -- sort entries in db client(e.g. pgAdmin) with this
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    refresh_token_hash TEXT NOT NULL UNIQUE,
    expires_at TIMESTAMP NOT NULL,
    status VARCHAR(16) NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'revoked')),
    revoked_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_refresh_token_hash ON sessions(refresh_token_hash);
CREATE INDEX IF NOT EXISTS idx_sessions_status ON sessions(status);
CREATE INDEX IF NOT EXISTS idx_sessions_active ON sessions(user_id, status, expires_at);
CREATE INDEX IF NOT EXISTS idx_sessions_creation_order ON sessions(creation_order);

CREATE TABLE IF NOT EXISTS sub_sessions (
    id UUID PRIMARY KEY,
    creation_order BIGSERIAL, -- sort entries in db client(e.g. pgAdmin) with this
    session_id UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    activity_type VARCHAR(64) NOT NULL,
    activity_description TEXT,
    ip_address VARCHAR(64),
    user_agent TEXT,
    request_method VARCHAR(16) NOT NULL,
    request_path TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_sub_sessions_session_id ON sub_sessions(session_id);
CREATE INDEX IF NOT EXISTS idx_sub_sessions_user_id ON sub_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sub_sessions_created_at ON sub_sessions(created_at);
CREATE INDEX IF NOT EXISTS idx_sub_sessions_activity_type ON sub_sessions(activity_type);
CREATE INDEX IF NOT EXISTS idx_sub_sessions_creation_order ON sub_sessions(creation_order);
