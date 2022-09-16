-- Your SQL goes here
CREATE TABLE "tasks" (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    name TEXT NOT NULL,
    started_at BIGINT NOT NULL,
    finished_at BIGINT
);

CREATE INDEX tasks_name_idx on tasks (name);
CREATE INDEX tasks_name_started_at_idx on tasks (name, started_at);
CREATE INDEX tasks_name_finished_at_idx on tasks (name, finished_at);
CREATE INDEX tasks_name_started_at_finished_at_idx on tasks (name, started_at, finished_at);
