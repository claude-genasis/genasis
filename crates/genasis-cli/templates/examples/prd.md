# PRD: Example Feature — Task Status

> Status: Draft
> Owner: TBD

## 1. Overview

Add the ability to track task status (pending / in_progress / done)
on every issue, with a visible badge on the kanban card and a filter
on the list view.

## 2. User Story

> As a developer, I want to mark tasks with their current state so I
> can see at a glance what's blocking the team.

## 3. Requirements

### 3.1 Schema
- Add `status` column to the issues table.
- Allowed values: `pending`, `in_progress`, `done`.
- Default `pending`.

### 3.2 UI
- Status badge on every kanban card (gray/blue/green).
- Status dropdown on the issue detail page.
- Filter dropdown on the list view: All | Pending | In Progress | Done.

### 3.3 API
- `PATCH /issues/:id` accepts `{ status }`.
- Validation: must be one of the three values.

## 4. Acceptance Criteria

- New issues default to `pending`.
- Changing status persists immediately and is visible to teammates
  within 1 second.
- Filter dropdown narrows the list deterministically.
- All existing tests still pass.

## 5. Non-goals (v1)

- Custom statuses defined per project.
- Status history / audit log.
- Email notifications on status change.
