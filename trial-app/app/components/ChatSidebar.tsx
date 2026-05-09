"use client";

import { useState, type ReactNode } from "react";

import { useLang } from "@/app/components/LangProvider";

// 1.2× of the original w-80 (320px) / w-96 (384px).
const SIDEBAR_WIDTH_CLASS = "w-96 sm:w-[460px]";

// h-full = 100% of the positioned parent (<section relative>),
// so the sidebar naturally spans heading → kanban bottom without
// exceeding the section and creating a second scrollbar.

// When closed, expose 64px peek from the right edge.
const SIDEBAR_PEEK = 64;
const SIDEBAR_CLOSED_TRANSLATE_CLASS = "translate-x-[calc(100%-64px)]";

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
    <aside
      data-testid="chat-sidebar"
      data-open={isOpen}
      data-peek-px={SIDEBAR_PEEK}
      aria-label={`${channelName} chat sidebar`}
      className={`absolute right-0 top-0 z-30 flex flex-col rounded-lg border border-slate-300 bg-slate-100 shadow-xl transition-transform duration-300 ease-out dark:border-slate-700 dark:bg-slate-900 ${SIDEBAR_WIDTH_CLASS} h-full ${
        isOpen ? "translate-x-0" : SIDEBAR_CLOSED_TRANSLATE_CLASS
      }`}
    >
      <header className="flex shrink-0 items-center justify-between rounded-t-lg border-b border-slate-300 bg-slate-200 px-4 py-2.5 dark:border-slate-700 dark:bg-slate-800">
        {/* Channel name left-aligned */}
        <span className="font-mono text-sm font-semibold text-slate-700 dark:text-slate-200">
          #{channelName}
        </span>
        {/* X close button right-aligned */}
        <button
          type="button"
          data-testid="chat-sidebar-close"
          onClick={close}
          aria-label={closeLabel}
          title={closeLabel}
          className="rounded-md p-1 text-slate-500 transition-colors hover:bg-slate-300 hover:text-slate-900 dark:text-slate-400 dark:hover:bg-slate-700 dark:hover:text-slate-100"
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

      {/* Handle: protrudes outside the aside's left edge.
          right-full places its right edge at the aside's left edge,
          so it hangs entirely to the left. Moves with the aside
          automatically — no separate open/closed position needed. */}
      <button
        type="button"
        data-testid="chat-sidebar-handle"
        onClick={toggle}
        aria-label={isOpen ? closeLabel : openLabel}
        aria-expanded={isOpen}
        title={isOpen ? closeLabel : openLabel}
        className="absolute right-full top-1/2 z-30 flex -translate-y-1/2 items-center justify-center rounded-l-md border border-r-0 border-slate-400 bg-slate-800 px-1.5 py-5 text-xs font-bold text-white shadow-md hover:bg-slate-700 dark:border-slate-600 dark:bg-slate-700 dark:hover:bg-slate-600"
      >
        <span aria-hidden="true">{isOpen ? "▶" : "◀"}</span>
      </button>
    </aside>
  );
}
