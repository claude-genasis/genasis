"use client";

import { useEffect, useRef, useState, type FormEvent, type KeyboardEvent } from "react";

import type { SimPost } from "@/db/sim";

const ACTOR_BADGE: Record<string, string> = {
  pm: "bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200",
  frontend:
    "bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200",
  backend:
    "bg-emerald-100 text-emerald-800 dark:bg-emerald-900 dark:text-emerald-200",
  "code-reviewer":
    "bg-amber-100 text-amber-800 dark:bg-amber-900 dark:text-amber-200",
  qa: "bg-rose-100 text-rose-800 dark:bg-rose-900 dark:text-rose-200",
  designer:
    "bg-pink-100 text-pink-800 dark:bg-pink-900 dark:text-pink-200",
  architect:
    "bg-cyan-100 text-cyan-800 dark:bg-cyan-900 dark:text-cyan-200",
  devops:
    "bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-200",
  ux: "bg-fuchsia-100 text-fuchsia-800 dark:bg-fuchsia-900 dark:text-fuchsia-200",
  human:
    "bg-neutral-200 text-neutral-800 dark:bg-neutral-700 dark:text-neutral-100",
};

const FALLBACK_BADGE =
  "bg-neutral-100 text-neutral-800 dark:bg-neutral-800 dark:text-neutral-200";

export type LiveChatThreadProps = {
  initialPosts: SimPost[];
  channelId: number;
  channelName: string;
};

function formatTime(iso: string): string {
  // SQLite produces "YYYY-MM-DD HH:MM:SS" without timezone. Render hour:minute.
  const m = /(\d{2}):(\d{2}):(\d{2})/.exec(iso);
  if (m) return `${m[1]}:${m[2]}`;
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return iso;
  return d.toLocaleTimeString("en-GB", { hour: "2-digit", minute: "2-digit" });
}

export function LiveChatThread({
  initialPosts,
  channelId,
  channelName,
}: LiveChatThreadProps) {
  const [posts, setPosts] = useState<SimPost[]>(initialPosts);
  const [draft, setDraft] = useState("");
  const [sending, setSending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const scrollRef = useRef<HTMLOListElement>(null);

  useEffect(() => {
    const es = new EventSource("/api/events/stream");
    const onPost = (e: MessageEvent<string>) => {
      const post = JSON.parse(e.data) as SimPost;
      if (post.channel_id !== channelId) return;
      setPosts((prev) =>
        prev.some((p) => p.id === post.id) ? prev : [...prev, post],
      );
    };
    es.addEventListener("post.created", onPost as EventListener);
    return () => {
      es.close();
    };
  }, [channelId]);

  useEffect(() => {
    const el = scrollRef.current;
    if (!el) return;
    el.scrollTo({ top: el.scrollHeight, behavior: "smooth" });
  }, [posts.length]);

  const send = async () => {
    const text = draft.trim();
    if (!text || sending) return;
    setSending(true);
    setError(null);
    try {
      const res = await fetch("/api/mattermost/posts", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          channel_id: channelId,
          actor: "human",
          message: text,
        }),
      });
      if (!res.ok) {
        const data = (await res.json().catch(() => ({}))) as {
          error?: string;
        };
        throw new Error(data.error ?? `HTTP ${res.status}`);
      }
      setDraft("");
    } catch (err) {
      setError(
        err instanceof Error
          ? `메시지 전송 실패: ${err.message}`
          : "메시지 전송 실패",
      );
    } finally {
      setSending(false);
    }
  };

  const onSubmit = (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    void send();
  };

  const onKeyDown = (e: KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      void send();
    }
  };

  return (
    <div
      data-testid="live-chat-thread"
      className="flex h-[420px] flex-col overflow-hidden rounded-lg border border-neutral-200 bg-white dark:border-neutral-800 dark:bg-neutral-950"
    >
      <header className="flex items-center justify-between border-b border-neutral-200 bg-neutral-50 px-4 py-2 text-sm font-semibold dark:border-neutral-800 dark:bg-neutral-900">
        <span className="font-mono text-neutral-700 dark:text-neutral-200">
          #{channelName}
        </span>
        <span className="text-xs font-normal text-neutral-500 dark:text-neutral-400">
          {posts.length}개 메시지 · 라이브
        </span>
      </header>
      <ol
        ref={scrollRef}
        aria-live="polite"
        aria-label="Live chat thread"
        data-testid="live-chat-message-list"
        className="flex-1 space-y-2 overflow-y-auto p-3"
      >
        {posts.map((post, i) => (
          <li
            key={post.id}
            data-message-index={i}
            data-actor={post.actor}
            className="flex items-start gap-2 text-sm leading-snug"
          >
            <span className="shrink-0 pt-0.5 font-mono text-xs text-neutral-500 dark:text-neutral-400">
              {formatTime(post.created_at)}
            </span>
            <span
              className={`shrink-0 rounded-full px-2 py-0.5 text-xs font-medium ${ACTOR_BADGE[post.actor] ?? FALLBACK_BADGE}`}
            >
              [{post.actor}]
            </span>
            <span className="text-neutral-900 dark:text-neutral-100">
              {post.message}
            </span>
          </li>
        ))}
        {posts.length === 0 ? (
          <li className="rounded-md border border-dashed border-neutral-200 p-4 text-center text-xs text-neutral-400 dark:border-neutral-800 dark:text-neutral-600">
            에이전트 호출을 기다리는 중… 직접 아래 입력창에 메시지를 보내볼 수도 있습니다.
          </li>
        ) : null}
      </ol>
      {error ? (
        <p
          data-testid="live-chat-error"
          role="alert"
          className="border-t border-red-200 bg-red-50 px-3 py-1.5 text-xs text-red-700 dark:border-red-900 dark:bg-red-950/40 dark:text-red-300"
        >
          {error}
        </p>
      ) : null}
      <form
        onSubmit={onSubmit}
        className="border-t border-neutral-200 bg-white p-2 dark:border-neutral-800 dark:bg-neutral-950"
      >
        <div className="flex items-end gap-2">
          <textarea
            data-testid="live-chat-composer"
            rows={1}
            value={draft}
            onChange={(e) => setDraft(e.target.value)}
            onKeyDown={onKeyDown}
            placeholder="메시지를 입력하고 Enter (Shift+Enter 줄바꿈)"
            className="flex-1 resize-none rounded-md border border-neutral-300 bg-white px-3 py-2 text-sm shadow-sm focus:border-neutral-500 focus:outline-none focus:ring-2 focus:ring-neutral-200 dark:border-neutral-700 dark:bg-neutral-950 dark:text-neutral-100"
            disabled={sending}
            aria-label="Compose message"
          />
          <button
            type="submit"
            data-testid="live-chat-send"
            disabled={sending || !draft.trim()}
            className="rounded-md bg-neutral-900 px-3 py-2 text-sm font-medium text-white shadow-sm hover:bg-neutral-700 disabled:cursor-not-allowed disabled:bg-neutral-300 disabled:text-neutral-600 dark:bg-white dark:text-neutral-900 dark:hover:bg-neutral-200 dark:disabled:bg-neutral-700 dark:disabled:text-neutral-400"
          >
            {sending ? "전송 중…" : "Send"}
          </button>
        </div>
      </form>
    </div>
  );
}
