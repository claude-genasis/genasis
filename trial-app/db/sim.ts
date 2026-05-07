import { emit } from "@/lib/events";

import { getDb } from "./index";

export type SimIssueState = "todo" | "inprogress" | "inreview" | "done";

export interface SimProject {
  id: number;
  slug: string;
  name: string;
  created_at: string;
}

export interface SimIssueRow {
  id: number;
  project_id: number;
  sequence_id: number;
  title: string;
  state: SimIssueState;
  assignee: string | null;
  created_at: string;
  updated_at: string;
}

export interface SimIssue extends SimIssueRow {
  project_slug: string;
}

export interface SimChannel {
  id: number;
  name: string;
  display_name: string;
  created_at: string;
}

export interface SimPost {
  id: number;
  channel_id: number;
  channel_name: string;
  root_id: number | null;
  actor: string;
  message: string;
  created_at: string;
}

let migrated = false;

function ensureMigrated() {
  if (migrated) return;
  const db = getDb();
  db.exec(`
    CREATE TABLE IF NOT EXISTS sim_projects (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      slug TEXT NOT NULL UNIQUE,
      name TEXT NOT NULL,
      created_at TEXT NOT NULL DEFAULT (datetime('now'))
    );
    CREATE TABLE IF NOT EXISTS sim_issues (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      project_id INTEGER NOT NULL REFERENCES sim_projects(id) ON DELETE CASCADE,
      sequence_id INTEGER NOT NULL,
      title TEXT NOT NULL,
      state TEXT NOT NULL DEFAULT 'todo'
        CHECK (state IN ('todo','inprogress','inreview','done')),
      assignee TEXT,
      created_at TEXT NOT NULL DEFAULT (datetime('now')),
      updated_at TEXT NOT NULL DEFAULT (datetime('now'))
    );
    CREATE INDEX IF NOT EXISTS idx_sim_issues_project_state
      ON sim_issues(project_id, state);
    CREATE TABLE IF NOT EXISTS sim_channels (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      name TEXT NOT NULL UNIQUE,
      display_name TEXT NOT NULL,
      created_at TEXT NOT NULL DEFAULT (datetime('now'))
    );
    CREATE TABLE IF NOT EXISTS sim_posts (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      channel_id INTEGER NOT NULL REFERENCES sim_channels(id) ON DELETE CASCADE,
      root_id INTEGER REFERENCES sim_posts(id) ON DELETE CASCADE,
      actor TEXT NOT NULL,
      message TEXT NOT NULL,
      created_at TEXT NOT NULL DEFAULT (datetime('now'))
    );
    CREATE INDEX IF NOT EXISTS idx_sim_posts_channel_created
      ON sim_posts(channel_id, created_at);
  `);
  migrated = true;
}

function projectBySlug(slug: string): SimProject | undefined {
  ensureMigrated();
  return getDb()
    .prepare("SELECT * FROM sim_projects WHERE slug = ?")
    .get(slug) as SimProject | undefined;
}

function projectById(id: number): SimProject | undefined {
  return getDb()
    .prepare("SELECT * FROM sim_projects WHERE id = ?")
    .get(id) as SimProject | undefined;
}

function issueRowToIssue(row: SimIssueRow, slug: string): SimIssue {
  return { ...row, project_slug: slug };
}

export function ensureProject(input: {
  slug: string;
  name: string;
}): SimProject {
  ensureMigrated();
  const existing = projectBySlug(input.slug);
  if (existing) return existing;
  const stmt = getDb().prepare(
    "INSERT INTO sim_projects (slug, name) VALUES (?, ?) RETURNING *",
  );
  const project = stmt.get(input.slug, input.name) as SimProject;
  emit({ kind: "project.created", payload: project });
  return project;
}

export function createIssue(input: {
  project_slug: string;
  title: string;
  assignee?: string | null;
}): SimIssue {
  ensureMigrated();
  const db = getDb();
  const project =
    projectBySlug(input.project_slug) ??
    ensureProject({ slug: input.project_slug, name: input.project_slug });
  const seqRow = db
    .prepare(
      "SELECT COALESCE(MAX(sequence_id), 0) AS m FROM sim_issues WHERE project_id = ?",
    )
    .get(project.id) as { m: number };
  const seq = seqRow.m + 1;
  const stmt = db.prepare(
    `INSERT INTO sim_issues (project_id, sequence_id, title, assignee)
     VALUES (?, ?, ?, ?)
     RETURNING *`,
  );
  const row = stmt.get(
    project.id,
    seq,
    input.title,
    input.assignee ?? null,
  ) as SimIssueRow;
  const issue = issueRowToIssue(row, project.slug);
  emit({ kind: "issue.created", payload: issue });
  return issue;
}

export function transitionIssue(
  id: number,
  patch: { state?: SimIssueState; assignee?: string | null },
): SimIssue | undefined {
  ensureMigrated();
  const db = getDb();
  const sets: string[] = [];
  const params: unknown[] = [];
  if (patch.state !== undefined) {
    sets.push("state = ?");
    params.push(patch.state);
  }
  if (patch.assignee !== undefined) {
    sets.push("assignee = ?");
    params.push(patch.assignee);
  }
  sets.push("updated_at = datetime('now')");
  params.push(id);
  const stmt = db.prepare(
    `UPDATE sim_issues SET ${sets.join(", ")} WHERE id = ? RETURNING *`,
  );
  const row = stmt.get(...params) as SimIssueRow | undefined;
  if (!row) return undefined;
  const project = projectById(row.project_id);
  if (!project) return undefined;
  const issue = issueRowToIssue(row, project.slug);
  emit({ kind: "issue.updated", payload: issue });
  return issue;
}

export function getIssueById(id: number): SimIssue | undefined {
  ensureMigrated();
  const row = getDb()
    .prepare("SELECT * FROM sim_issues WHERE id = ?")
    .get(id) as SimIssueRow | undefined;
  if (!row) return undefined;
  const project = projectById(row.project_id);
  if (!project) return undefined;
  return issueRowToIssue(row, project.slug);
}

export function listIssues(input: { project_slug: string }): SimIssue[] {
  ensureMigrated();
  const project = projectBySlug(input.project_slug);
  if (!project) return [];
  const rows = getDb()
    .prepare(
      "SELECT * FROM sim_issues WHERE project_id = ? ORDER BY sequence_id",
    )
    .all(project.id) as SimIssueRow[];
  return rows.map((r) => issueRowToIssue(r, project.slug));
}

export function ensureChannel(input: {
  name: string;
  display_name: string;
}): SimChannel {
  ensureMigrated();
  const db = getDb();
  const existing = db
    .prepare("SELECT * FROM sim_channels WHERE name = ?")
    .get(input.name) as SimChannel | undefined;
  if (existing) return existing;
  const channel = db
    .prepare(
      "INSERT INTO sim_channels (name, display_name) VALUES (?, ?) RETURNING *",
    )
    .get(input.name, input.display_name) as SimChannel;
  emit({ kind: "channel.created", payload: channel });
  return channel;
}

function postWithChannelName(
  row: { id: number; channel_id: number; root_id: number | null; actor: string; message: string; created_at: string },
  channelName: string,
): SimPost {
  return {
    id: row.id,
    channel_id: row.channel_id,
    channel_name: channelName,
    root_id: row.root_id,
    actor: row.actor,
    message: row.message,
    created_at: row.created_at,
  };
}

function channelById(id: number): SimChannel | undefined {
  return getDb()
    .prepare("SELECT * FROM sim_channels WHERE id = ?")
    .get(id) as SimChannel | undefined;
}

export function postRoot(input: {
  channel_id: number;
  actor: string;
  message: string;
}): SimPost | undefined {
  ensureMigrated();
  const channel = channelById(input.channel_id);
  if (!channel) return undefined;
  const row = getDb()
    .prepare(
      `INSERT INTO sim_posts (channel_id, root_id, actor, message)
       VALUES (?, NULL, ?, ?)
       RETURNING *`,
    )
    .get(input.channel_id, input.actor, input.message) as {
    id: number;
    channel_id: number;
    root_id: number | null;
    actor: string;
    message: string;
    created_at: string;
  };
  const post = postWithChannelName(row, channel.name);
  emit({ kind: "post.created", payload: post });
  return post;
}

export function postThread(input: {
  channel_id: number;
  root_id: number;
  actor: string;
  message: string;
}): SimPost | undefined {
  ensureMigrated();
  const channel = channelById(input.channel_id);
  if (!channel) return undefined;
  const row = getDb()
    .prepare(
      `INSERT INTO sim_posts (channel_id, root_id, actor, message)
       VALUES (?, ?, ?, ?)
       RETURNING *`,
    )
    .get(input.channel_id, input.root_id, input.actor, input.message) as {
    id: number;
    channel_id: number;
    root_id: number | null;
    actor: string;
    message: string;
    created_at: string;
  };
  const post = postWithChannelName(row, channel.name);
  emit({ kind: "post.created", payload: post });
  return post;
}

export function listPosts(input: {
  channel_id?: number;
  channel_name?: string;
}): SimPost[] {
  ensureMigrated();
  let channel: SimChannel | undefined;
  if (input.channel_id !== undefined) {
    channel = channelById(input.channel_id);
  } else if (input.channel_name) {
    channel = getDb()
      .prepare("SELECT * FROM sim_channels WHERE name = ?")
      .get(input.channel_name) as SimChannel | undefined;
  }
  if (!channel) return [];
  const rows = getDb()
    .prepare(
      `SELECT * FROM sim_posts
       WHERE channel_id = ?
       ORDER BY created_at, id`,
    )
    .all(channel.id) as Array<{
    id: number;
    channel_id: number;
    root_id: number | null;
    actor: string;
    message: string;
    created_at: string;
  }>;
  return rows.map((r) => postWithChannelName(r, channel!.name));
}

export function getChannelByName(name: string): SimChannel | undefined {
  ensureMigrated();
  return getDb()
    .prepare("SELECT * FROM sim_channels WHERE name = ?")
    .get(name) as SimChannel | undefined;
}
