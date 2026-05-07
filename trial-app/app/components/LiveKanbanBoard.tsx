"use client";

import { useEffect, useRef, useState, type DragEvent } from "react";

import type { SimIssue, SimIssueState } from "@/db/sim";

const COLUMNS: {
  key: SimIssueState;
  label: string;
  headerClass: string;
  countClass: string;
}[] = [
  {
    key: "todo",
    label: "Todo",
    headerClass:
      "bg-neutral-100 text-neutral-700 dark:bg-neutral-800 dark:text-neutral-200",
    countClass:
      "bg-neutral-200 text-neutral-700 dark:bg-neutral-700 dark:text-neutral-200",
  },
  {
    key: "inprogress",
    label: "In Progress",
    headerClass:
      "bg-blue-100 text-blue-800 dark:bg-blue-950 dark:text-blue-200",
    countClass:
      "bg-blue-200 text-blue-800 dark:bg-blue-900 dark:text-blue-200",
  },
  {
    key: "inreview",
    label: "In Review",
    headerClass:
      "bg-amber-100 text-amber-800 dark:bg-amber-950 dark:text-amber-200",
    countClass:
      "bg-amber-200 text-amber-800 dark:bg-amber-900 dark:text-amber-200",
  },
  {
    key: "done",
    label: "Done",
    headerClass:
      "bg-green-100 text-green-800 dark:bg-green-950 dark:text-green-200",
    countClass:
      "bg-green-200 text-green-800 dark:bg-green-900 dark:text-green-200",
  },
];

export type LiveKanbanBoardProps = {
  initialIssues: SimIssue[];
  projectSlug: string;
};

export function LiveKanbanBoard({
  initialIssues,
  projectSlug,
}: LiveKanbanBoardProps) {
  const [issues, setIssues] = useState<SimIssue[]>(initialIssues);
  const [draggingId, setDraggingId] = useState<number | null>(null);
  const [hoverColumn, setHoverColumn] = useState<SimIssueState | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const es = new EventSource("/api/events/stream");

    const onCreated = (e: MessageEvent<string>) => {
      const issue = JSON.parse(e.data) as SimIssue;
      if (issue.project_slug !== projectSlug) return;
      setIssues((prev) =>
        prev.some((i) => i.id === issue.id) ? prev : [...prev, issue],
      );
    };
    const onUpdated = (e: MessageEvent<string>) => {
      const issue = JSON.parse(e.data) as SimIssue;
      if (issue.project_slug !== projectSlug) return;
      setIssues((prev) =>
        prev.map((i) => (i.id === issue.id ? issue : i)),
      );
    };

    es.addEventListener("issue.created", onCreated as EventListener);
    es.addEventListener("issue.updated", onUpdated as EventListener);

    return () => {
      es.close();
    };
  }, [projectSlug]);

  const onDragStart = (id: number) => (e: DragEvent<HTMLLIElement>) => {
    setDraggingId(id);
    e.dataTransfer.effectAllowed = "move";
    e.dataTransfer.setData("text/plain", String(id));
  };

  const onDragEnd = () => {
    setDraggingId(null);
    setHoverColumn(null);
  };

  const onDragOver = (column: SimIssueState) => (e: DragEvent<HTMLElement>) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = "move";
    if (hoverColumn !== column) setHoverColumn(column);
  };

  const onDragLeave = () => {
    setHoverColumn(null);
  };

  const onDrop = (column: SimIssueState) => async (
    e: DragEvent<HTMLElement>,
  ) => {
    e.preventDefault();
    setHoverColumn(null);
    const id = Number(e.dataTransfer.getData("text/plain"));
    if (!Number.isInteger(id)) return;
    const before = issues;
    const target = issues.find((i) => i.id === id);
    if (!target || target.state === column) return;

    setIssues((prev) =>
      prev.map((i) => (i.id === id ? { ...i, state: column } : i)),
    );
    setError(null);
    try {
      const res = await fetch(`/api/plane/issues/${id}`, {
        method: "PATCH",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ state: column }),
      });
      if (!res.ok) {
        throw new Error(`patch failed: ${res.status}`);
      }
    } catch (err) {
      setIssues(before);
      setError(
        err instanceof Error
          ? `상태 변경 실패: ${err.message}`
          : "상태 변경 실패",
      );
    }
  };

  return (
    <div className="space-y-2" data-testid="live-kanban">
      {error ? (
        <p
          data-testid="live-kanban-error"
          role="alert"
          className="rounded-md border border-red-200 bg-red-50 px-3 py-1.5 text-xs text-red-700 dark:border-red-900 dark:bg-red-950/40 dark:text-red-300"
        >
          {error}
        </p>
      ) : null}
      <div
        role="list"
        aria-label="Live kanban board"
        className="grid h-[420px] grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-4"
      >
        {COLUMNS.map(({ key, label, headerClass, countClass }) => {
          const columnIssues = issues.filter((i) => i.state === key);
          const isHover = hoverColumn === key;
          return (
            <section
              key={key}
              role="listitem"
              data-column={key}
              aria-label={`${label} column`}
              onDragOver={onDragOver(key)}
              onDragLeave={onDragLeave}
              onDrop={onDrop(key)}
              className={`flex h-full flex-col overflow-hidden rounded-lg border bg-white transition-colors dark:bg-neutral-950 ${
                isHover
                  ? "border-blue-400 ring-2 ring-blue-200 dark:border-blue-500 dark:ring-blue-900"
                  : "border-neutral-200 dark:border-neutral-800"
              }`}
            >
              <header
                className={`flex items-center justify-between px-3 py-2 text-sm font-semibold ${headerClass}`}
              >
                <span>{label}</span>
                <span
                  className={`rounded-full px-2 py-0.5 text-xs font-medium ${countClass}`}
                  aria-label={`${columnIssues.length} cards`}
                >
                  {columnIssues.length}
                </span>
              </header>
              <ol className="flex-1 space-y-2 overflow-y-auto p-2">
                {columnIssues.map((issue) => (
                  <li
                    key={issue.id}
                    data-card-id={issue.id}
                    draggable
                    tabIndex={0}
                    onDragStart={onDragStart(issue.id)}
                    onDragEnd={onDragEnd}
                    className={`animate-card-enter cursor-grab rounded-md border border-neutral-200 bg-white px-3 py-2 text-sm shadow-sm transition-opacity active:cursor-grabbing dark:border-neutral-700 dark:bg-neutral-900 ${
                      draggingId === issue.id ? "opacity-50" : ""
                    }`}
                  >
                    <div className="flex items-baseline gap-2">
                      <span className="font-mono text-xs text-neutral-500 dark:text-neutral-400">
                        #{issue.sequence_id}
                      </span>
                      <span className="text-neutral-900 dark:text-neutral-100">
                        {issue.title}
                      </span>
                    </div>
                    {issue.assignee ? (
                      <div className="mt-1 text-xs text-neutral-500 dark:text-neutral-400">
                        @{issue.assignee}
                      </div>
                    ) : null}
                  </li>
                ))}
                {columnIssues.length === 0 ? (
                  <li className="rounded-md border border-dashed border-neutral-200 px-3 py-2 text-center text-xs text-neutral-400 dark:border-neutral-800 dark:text-neutral-600">
                    Drop a card here
                  </li>
                ) : null}
              </ol>
            </section>
          );
        })}
      </div>
    </div>
  );
}
