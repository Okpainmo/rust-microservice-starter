ALTER TABLE sessions
    ADD COLUMN IF NOT EXISTS creation_order BIGSERIAL;

ALTER TABLE sub_sessions
    ADD COLUMN IF NOT EXISTS creation_order BIGSERIAL;

CREATE INDEX IF NOT EXISTS idx_sessions_creation_order ON sessions(creation_order);
CREATE INDEX IF NOT EXISTS idx_sub_sessions_creation_order ON sub_sessions(creation_order);
