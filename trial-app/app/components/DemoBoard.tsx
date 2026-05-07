"use client";

import { ChatThread } from "@/app/components/ChatThread";
import { KanbanBoard } from "@/app/components/KanbanBoard";
import { DEMO_STEPS } from "@/lib/demo-script";
import { useDemoSprint } from "@/lib/use-demo-sprint";

export function DemoBoard() {
  const {
    cards,
    messages,
    typingActor,
    status,
    completedSteps,
    run,
    reset,
  } = useDemoSprint();

  const runLabel =
    status === "running"
      ? "재생 중…"
      : status === "complete"
        ? "▶ 다시 재생"
        : "▶ Run Demo Sprint";

  const statusLabel =
    status === "idle"
      ? "대기 중 — Run 버튼을 누르면 8단계 데모가 재생됩니다."
      : status === "running"
        ? `재생 중 — ${completedSteps} / ${DEMO_STEPS.length} 단계`
        : `완료 — ${DEMO_STEPS.length} / ${DEMO_STEPS.length} 단계`;

  return (
    <div className="space-y-4" data-testid="demo-board" data-status={status}>
      <div className="flex flex-wrap items-center gap-3">
        <button
          type="button"
          onClick={run}
          disabled={status === "running"}
          data-testid="demo-run-button"
          className="rounded-md bg-neutral-900 px-4 py-2 text-sm font-medium text-white shadow-sm transition-colors hover:bg-neutral-700 disabled:cursor-not-allowed disabled:bg-neutral-300 disabled:text-neutral-600 dark:bg-white dark:text-neutral-900 dark:hover:bg-neutral-200 dark:disabled:bg-neutral-700 dark:disabled:text-neutral-400"
        >
          {runLabel}
        </button>
        <button
          type="button"
          onClick={reset}
          disabled={status === "idle"}
          data-testid="demo-reset-button"
          className="rounded-md border border-neutral-300 bg-white px-4 py-2 text-sm font-medium text-neutral-800 shadow-sm transition-colors hover:bg-neutral-50 disabled:cursor-not-allowed disabled:opacity-50 dark:border-neutral-700 dark:bg-neutral-950 dark:text-neutral-100 dark:hover:bg-neutral-900"
        >
          Reset
        </button>
        <span
          data-testid="demo-status"
          className="text-xs text-neutral-500 dark:text-neutral-400"
          aria-live="polite"
        >
          {statusLabel}
        </span>
      </div>
      <div className="grid grid-cols-1 gap-4 lg:grid-cols-[1fr_minmax(280px,360px)]">
        <KanbanBoard cards={cards} />
        <ChatThread messages={messages} typingActor={typingActor} />
      </div>
    </div>
  );
}
