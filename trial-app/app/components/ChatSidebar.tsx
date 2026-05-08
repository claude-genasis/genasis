"use client";

import { useState, type ReactNode } from "react";

import { useLang } from "@/app/components/LangProvider";

// 1.2× of the previous w-80 (320px) / w-96 (384px) sizes.
const SIDEBAR_WIDTH_CLASS = "w-96 sm:w-[460px]";
const HANDLE_OPEN_LEFT_CLASS = "left-96 sm:left-[460px]";

// Matches LiveKanbanBoard's grid height so the panel and the kanban
// share the same vertical extent and top edge.
const SIDEBAR_HEIGHT_CLASS = "h-[630px]";

export function ChatSidebar({
  channelName,
  defaultOpen = true,
  children,
}: {
  channelName: string;
  defaultOpen?: boolean;
  children: ReactNode;
}) {
  const { t } = useLang();
  const [isOpen, setOpen] = useState(defaultOpen);

  const close = () => setOpen(false);
  const toggle = () => setOpen((v) => !v);

  const closeLabel = t("sidebar.close");
  const openLabel = t("sidebar.open");

  return (
    <>
      <aside
        data-testid="chat-sidebar"
        data-open={isOpen}
        aria-label={`${channelName} chat sidebar`}
        aria-hidden={!isOpen}
        className={`absolute left-0 top-0 z-30 flex flex-col rounded-lg border border-neutral-200 bg-neutral-50 shadow-xl transition-transform duration-300 ease-out dark:border-neutral-800 dark:bg-neutral-900 ${SIDEBAR_WIDTH_CLASS} ${SIDEBAR_HEIGHT_CLASS} ${
          isOpen ? "translate-x-0" : "-translate-x-full"
        }`}
      >
        <header className="flex shrink-0 items-center justify-between rounded-t-lg border-b border-neutral-200 bg-neutral-100 px-4 py-2.5 dark:border-neutral-800 dark:bg-neutral-800">
          <span className="font-mono text-sm font-semibold text-neutral-700 dark:text-neutral-200">
            #{channelName}
          </span>
          <button
            type="button"
            data-testid="chat-sidebar-close"
            onClick={close}
            aria-label={closeLabel}
            title={closeLabel}
            className="rounded-md p-1 text-neutral-500 transition-colors hover:bg-neutral-200 hover:text-neutral-900 dark:text-neutral-400 dark:hover:bg-neutral-700 dark:hover:text-neutral-100"
          >
            <svg
              width="16"
              height="16"
              viewBox="0 0 16 16"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
              aria-hidden="true"
            >
              <path d="M3 3 L13 13 M13 3 L3 13" />
            </svg>
          </button>
        </header>
        <div className="min-h-0 flex-1 overflow-hidden rounded-b-lg">
          {children}
        </div>
      </aside>
      <button
        type="button"
        data-testid="chat-sidebar-handle"
        onClick={toggle}
        aria-label={isOpen ? closeLabel : openLabel}
        aria-expanded={isOpen}
        title={isOpen ? closeLabel : openLabel}
        className={`absolute top-1/2 z-30 flex -translate-y-1/2 items-center justify-center rounded-r-md border border-l-0 border-neutral-300 bg-neutral-800 px-1.5 py-5 text-xs font-bold text-white shadow-md transition-[left] duration-300 ease-out hover:bg-neutral-700 dark:border-neutral-700 dark:bg-neutral-700 dark:hover:bg-neutral-600 ${
          isOpen ? HANDLE_OPEN_LEFT_CLASS : "left-0"
        }`}
      >
        <span aria-hidden="true">{isOpen ? "◀" : "▶"}</span>
      </button>
    </>
  );
}
