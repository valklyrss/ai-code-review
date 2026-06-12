ALTER TABLE repo_config ADD COLUMN sync_status TEXT NOT NULL DEFAULT 'NOT_SYNCED';
ALTER TABLE repo_config ADD COLUMN sync_progress INTEGER NOT NULL DEFAULT 0;
ALTER TABLE repo_config ADD COLUMN sync_message TEXT;
ALTER TABLE repo_config ADD COLUMN sync_started_at TEXT;
ALTER TABLE repo_config ADD COLUMN sync_finished_at TEXT;
