ALTER TABLE sessions
    ADD COLUMN IF NOT EXISTS status VARCHAR(16) NOT NULL DEFAULT 'active';

ALTER TABLE sessions
    DROP CONSTRAINT IF EXISTS sessions_status_check;

ALTER TABLE sessions
    ADD CONSTRAINT sessions_status_check CHECK (status IN ('active', 'revoked'));

UPDATE sessions
SET status = 'revoked'
WHERE revoked_at IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_sessions_status ON sessions(status);

DROP INDEX IF EXISTS idx_sessions_active;
CREATE INDEX IF NOT EXISTS idx_sessions_active ON sessions(user_id, status, expires_at);
