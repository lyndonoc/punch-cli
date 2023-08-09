-- Add up migration script here
CREATE TABLE "tasks" (
    id BIGSERIAL PRIMARY KEY,
    user_github_id VARCHAR NOT NULL,
    name TEXT NOT NULL,
    started_at BIGINT NOT NULL,
    finished_at BIGINT
);

CREATE INDEX user_tasks_started_at_idx ON tasks (user_github_id, started_at);
CREATE INDEX user_tasks_finished_at_idx ON tasks (user_github_id, finished_at);
CREATE INDEX user_tasks_started_at_finished_at_idx ON tasks (user_github_id, started_at, finished_at);
