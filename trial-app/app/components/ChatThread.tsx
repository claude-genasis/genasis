"use client";

import { useEffect, useRef } from "react";

import { useLang } from "@/app/components/LangProvider";

export type ChatMessage = {
  time: string;
  actor: string;
  text: string;
};

const ACTOR_BADGE: Record<string, string> = {
  pm: "bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200",
  frontend: "bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200",
  backend: "bg-emerald-100 text-emerald-800 dark:bg-emerald-900 dark:text-emerald-200",
  "code-reviewer": "bg-amber-100 text-amber-800 dark:bg-amber-900 dark:text-amber-200",
  qa: "bg-rose-100 text-rose-800 dark:bg-rose-900 dark:text-rose-200",
  designer: "bg-pink-100 text-pink-800 dark:bg-pink-900 dark:text-pink-200",
  architect: "bg-cyan-100 text-cyan-800 dark:bg-cyan-900 dark:text-cyan-200",
  devops: "bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-200",
  ux: "bg-fuchsia-100 text-fuchsia-800 dark:bg-fuchsia-900 dark:text-fuchsia-200",
  human: "bg-neutral-200 text-neutral-800 dark:bg-neutral-700 dark:text-neutral-100",
};

const FALLBACK_BADGE =
  "bg-neutral-100 text-neutral-800 dark:bg-neutral-800 dark:text-neutral-200";

export function ChatThread({
  messages,
  typingActor = null,
  channel = "scrum-demo",
}: {
  messages: ChatMessage[];
  typingActor?: string | null;
  channel?: string;
}) {
  const { t } = useLang();
  const scrollRef = useRef<HTMLOListElement>(null);

  useEffect(() => {
    const el = scrollRef.current;
    if (!el) return;
    el.scrollTo({ top: el.scrollHeight, behavior: "smooth" });
  }, [messages.length, typingActor]);

  return (
    <div
      data-testid="chat-thread"
      className="flex h-[420px] flex-col overflow-hidden rounded-lg border border-neutral-200 bg-white dark:border-neutral-800 dark:bg-neutral-950"
    >
      <header className="flex items-center justify-between border-b border-neutral-200 bg-neutral-50 px-4 py-2 text-sm font-semibold dark:border-neutral-800 dark:bg-neutral-900">
        <span className="font-mono text-neutral-700 dark:text-neutral-200">
          #{channel}
        </span>
        <span className="text-xs font-normal text-neutral-500 dark:text-neutral-400">
          {messages.length}개 메시지
        </span>
      </header>
      <ol
        ref={scrollRef}
        aria-live="polite"
        aria-label="Chat thread"
        data-testid="chat-message-list"
        className="flex-1 space-y-2 overflow-y-auto p-3"
      >
        {messages.map((msg, i) => {
          const badge = ACTOR_BADGE[msg.actor] ?? FALLBACK_BADGE;
          return (
            <li
              key={i}
              data-message-index={i}
              data-actor={msg.actor}
              className="flex items-start gap-2 text-sm leading-snug"
            >
              <span className="shrink-0 pt-0.5 font-mono text-xs text-neutral-500 dark:text-neutral-400">
                {msg.time}
              </span>
              <span
                className={`shrink-0 rounded-full px-2 py-0.5 text-xs font-medium ${badge}`}
              >
                [{msg.actor}]
              </span>
              <span className="text-neutral-900 dark:text-neutral-100">
                {msg.text}
              </span>
            </li>
          );
        })}
        {typingActor ? (
          <li
            data-testid="chat-typing-indicator"
            data-actor={typingActor}
            aria-live="polite"
            aria-label={`${typingActor} is typing`}
            className="flex items-center gap-2 px-1 py-2 text-sm"
          >
            <span
              className={`shrink-0 rounded-full px-2 py-0.5 text-xs font-medium ${ACTOR_BADGE[typingActor] ?? FALLBACK_BADGE}`}
            >
              [{typingActor}]
            </span>
            <span className="flex items-center gap-1.5 text-neutral-400 dark:text-neutral-500">
              <span className="h-1.5 w-1.5 animate-bounce rounded-full bg-current [animation-delay:-0.3s]" />
              <span className="h-1.5 w-1.5 animate-bounce rounded-full bg-current [animation-delay:-0.15s]" />
              <span className="h-1.5 w-1.5 animate-bounce rounded-full bg-current" />
            </span>
            <span className="text-xs text-neutral-500 dark:text-neutral-400">
              {t("demo.chat.typingSuffix")}
            </span>
          </li>
        ) : null}
      </ol>
    </div>
  );
}
