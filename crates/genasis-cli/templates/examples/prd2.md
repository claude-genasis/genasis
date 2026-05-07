# PRD2: Example Feature Expansion — Task Comments + Mentions

> Status: Draft
> Owner: TBD
> Depends on: PRD.md (task status)

## 1. Overview

Extend the task tracker with threaded comments and `@user` mentions
that surface as Mattermost notifications.

## 2. User Story

> As a reviewer, I want to leave threaded comments on a task and
> mention specific teammates so the right person gets notified
> without me having to switch tools.

## 3. Requirements

### 3.1 Comments
- Add a `comments` table: `id, task_id (FK), author, body, created_at`.
- Comment composer on the task detail page.
- Comments render in chronological order with author + timestamp.

### 3.2 Mentions
- Detect `@username` patterns in the body.
- Resolve to a known user; if no match, render as plain text.
- On save, post a Mattermost message to the mentioned user's DM
  with a deep link back to the task.

### 3.3 API
- `POST /tasks/:id/comments` — create.
- `GET /tasks/:id/comments` — list.
- `DELETE /comments/:id` — author-only.

## 4. Acceptance Criteria

- A reviewer can post a comment that includes `@frontend`.
- @frontend receives a DM in Mattermost within 5 seconds.
- Comment list reflects new posts without a full page refresh.
- Deletion is restricted to the original author or an admin.

## 5. Non-goals (v1)

- Rich text / Markdown rendering beyond plain text and mentions.
- File attachments.
- Reactions / emoji.
