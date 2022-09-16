-- This file should undo anything in `up.sql`
DROP INDEX tasks_name_started_at_finished_at_idx;
DROP INDEX tasks_name_finished_at_idx;
DROP INDEX tasks_name_started_at_idx;
DROP INDEX tasks_name_idx;

DROP TABLE "tasks";
