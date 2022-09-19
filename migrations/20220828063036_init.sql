CREATE TYPE lesson_status AS ENUM ('rejected', 'new', 'approved', 'best');

CREATE TABLE lesson (
    id serial PRIMARY KEY,
    text TEXT NOT NULL,
    spam_token TEXT NOT NULL,
    status lesson_status NOT NULL DEFAULT 'new',
    created_at timestamptz NOT NULL DEFAULT now()
);
