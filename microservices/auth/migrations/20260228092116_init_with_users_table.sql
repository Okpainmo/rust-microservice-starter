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
    is_admin BOOLEAN NOT NULL DEFAULT FALSE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    is_logged_out BOOLEAN NOT NULL DEFAULT FALSE,
    phone_number VARCHAR(20) UNIQUE,
    country VARCHAR(100),
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Index for users.email
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);