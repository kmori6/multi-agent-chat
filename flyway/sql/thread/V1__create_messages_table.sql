CREATE TABLE messages (
    id uuid PRIMARY KEY,
    thread_id uuid NOT NULL REFERENCES threads(id) ON DELETE CASCADE,
    speaker text NOT NULL
        CHECK (speaker IN ('user', 'agent_a', 'agent_b', 'system')),
    content text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);
