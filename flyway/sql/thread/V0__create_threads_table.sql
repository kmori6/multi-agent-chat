CREATE TABLE threads (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  title text,
  status text NOT NULL DEFAULT 'idle'
    CHECK (status IN ('idle', 'running', 'archived', 'error')),
  agent_a_name text NOT NULL,
  agent_a_persona text NOT NULL,
  agent_b_name text NOT NULL,
  agent_b_persona text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);
