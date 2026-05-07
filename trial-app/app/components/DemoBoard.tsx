"use client";

import { ChatThread } from "@/app/components/ChatThread";
import { KanbanBoard } from "@/app/components/KanbanBoard";
import { useLang } from "@/app/components/LangProvider";
import { DEMO_STEPS } from "@/lib/demo-script";
import { useDemoSprint } from "@/lib/use-demo-sprint";

export function DemoBoard() {
  const { t } = useLang();
  const {
    cards,
    messages,
    typingActor,
    status,
    completedSteps,
    run,
    reset,
  } = useDemoSprint();

  const total = DEMO_STEPS.length;
  const runLabel =
    status === "running"
      ? t("demo.run.running")
      : status === "complete"
        ? t("demo.run.complete")
        : t("demo.run.idle");

  const statusLabel =
    status === "idle"
      ? t("demo.status.idle")
      : status === "running"
        ? t("demo.status.running", { completed: completedSteps, total })
        : t("demo.status.complete", { total });

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
          {t("demo.reset")}
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
