import Database from "better-sqlite3";
import { mkdirSync } from "node:fs";
import { dirname, resolve } from "node:path";

export type SubmissionStatus = "pending" | "provisioned" | "revoked";

export interface SubmissionRow {
  id: number;
  token: string;
  name: string;
  email: string;
  phone: string | null;
  project_name: string;
  team_size: string;
  tech_stack: string | null;
  message: string | null;
  status: SubmissionStatus;
  credentials_json: string | null;
  created_at: string;
  updated_at: string;
}

const DEFAULT_DB_PATH = "./data/trial.db";

let dbInstance: Database.Database | null = null;

export function getDatabasePath(): string {
  return process.env.DATABASE_PATH ?? DEFAULT_DB_PATH;
}

export function getDb(): Database.Database {
  if (dbInstance) return dbInstance;

  const path = resolve(getDatabasePath());
  mkdirSync(dirname(path), { recursive: true });

  const db = new Database(path);
  db.pragma("journal_mode = WAL");
  db.pragma("foreign_keys = ON");

  migrate(db);

  dbInstance = db;
  return db;
}

function migrate(db: Database.Database): void {
  db.exec(`
    CREATE TABLE IF NOT EXISTS submissions (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      token TEXT NOT NULL UNIQUE,
      name TEXT NOT NULL,
      email TEXT NOT NULL,
      phone TEXT,
      project_name TEXT NOT NULL,
      team_size TEXT NOT NULL,
      tech_stack TEXT,
      message TEXT,
      status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'provisioned', 'revoked')),
      credentials_json TEXT,
      created_at TEXT NOT NULL DEFAULT (datetime('now')),
      updated_at TEXT NOT NULL DEFAULT (datetime('now'))
    );
    CREATE INDEX IF NOT EXISTS idx_submissions_token ON submissions(token);
    CREATE INDEX IF NOT EXISTS idx_submissions_status ON submissions(status);
  `);
}

export function closeDb(): void {
  if (dbInstance) {
    dbInstance.close();
    dbInstance = null;
  }
}
