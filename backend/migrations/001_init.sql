CREATE TABLE IF NOT EXISTS repo_config (
  id TEXT PRIMARY KEY,
  repo_name TEXT NOT NULL,
  repo_url TEXT NOT NULL,
  auth_type TEXT NOT NULL DEFAULT 'SSH',
  username TEXT,
  access_token TEXT,
  branch_pattern TEXT DEFAULT '*',
  scan_interval_seconds INTEGER DEFAULT 60,
  enabled INTEGER DEFAULT 1,
  owner_email TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS repo_branch_state (
  id TEXT PRIMARY KEY,
  repo_id TEXT NOT NULL,
  branch_name TEXT NOT NULL,
  last_commit_id TEXT,
  last_scan_time TEXT,
  updated_at TEXT,
  UNIQUE(repo_id, branch_name)
);

CREATE TABLE IF NOT EXISTS review_task (
  id TEXT PRIMARY KEY,
  repo_id TEXT NOT NULL,
  repo_name TEXT NOT NULL,
  branch_name TEXT NOT NULL,
  old_commit_id TEXT,
  new_commit_id TEXT NOT NULL,
  status TEXT NOT NULL,
  result TEXT,
  risk_level TEXT,
  commit_count INTEGER DEFAULT 0,
  file_count INTEGER DEFAULT 0,
  issue_count INTEGER DEFAULT 0,
  high_count INTEGER DEFAULT 0,
  critical_count INTEGER DEFAULT 0,
  email_sent INTEGER DEFAULT 0,
  error_msg TEXT,
  created_at TEXT NOT NULL,
  started_at TEXT,
  finished_at TEXT
);

CREATE TABLE IF NOT EXISTS review_commit (
  id TEXT PRIMARY KEY,
  task_id TEXT NOT NULL,
  commit_id TEXT NOT NULL,
  author_name TEXT,
  author_email TEXT,
  commit_msg TEXT,
  commit_time TEXT
);

CREATE TABLE IF NOT EXISTS review_file (
  id TEXT PRIMARY KEY,
  task_id TEXT NOT NULL,
  file_path TEXT NOT NULL,
  change_type TEXT,
  additions INTEGER DEFAULT 0,
  deletions INTEGER DEFAULT 0,
  diff_content TEXT,
  skipped INTEGER DEFAULT 0,
  skip_reason TEXT
);

CREATE TABLE IF NOT EXISTS review_issue (
  id TEXT PRIMARY KEY,
  task_id TEXT NOT NULL,
  file_id TEXT,
  file_path TEXT NOT NULL,
  line_no INTEGER,
  issue_level TEXT NOT NULL,
  issue_type TEXT,
  title TEXT,
  description TEXT,
  suggestion TEXT,
  status TEXT DEFAULT 'TODO',
  need_email INTEGER DEFAULT 0,
  created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_review_task_created_at ON review_task(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_review_issue_task_id ON review_issue(task_id);
CREATE INDEX IF NOT EXISTS idx_review_issue_level ON review_issue(issue_level);
