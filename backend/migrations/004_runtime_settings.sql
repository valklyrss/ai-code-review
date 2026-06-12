CREATE TABLE IF NOT EXISTS scanner_setting (
  id TEXT PRIMARY KEY,
  interval_seconds INTEGER NOT NULL DEFAULT 60,
  max_concurrent_tasks INTEGER NOT NULL DEFAULT 1,
  max_diff_lines INTEGER NOT NULL DEFAULT 3000,
  max_file_diff_lines INTEGER NOT NULL DEFAULT 800,
  git_command_timeout_seconds INTEGER NOT NULL DEFAULT 120,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS mail_setting (
  id TEXT PRIMARY KEY,
  enabled INTEGER NOT NULL DEFAULT 0,
  smtp_host TEXT,
  smtp_port INTEGER NOT NULL DEFAULT 465,
  username TEXT,
  password TEXT,
  from_addr TEXT NOT NULL DEFAULT 'AI代码审核 <ai-review@example.com>',
  use_tls INTEGER NOT NULL DEFAULT 1,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS review_setting (
  id TEXT PRIMARY KEY,
  default_prompt_name TEXT NOT NULL DEFAULT 'java_legacy',
  serious_levels TEXT NOT NULL,
  allowed_extensions TEXT NOT NULL,
  ignore_paths TEXT NOT NULL,
  ignore_extensions TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

