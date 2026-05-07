import type { SimChannel, SimIssue, SimPost, SimProject } from "@/db/sim";

export type SimEvent =
  | { kind: "project.created"; payload: SimProject }
  | { kind: "issue.created"; payload: SimIssue }
  | { kind: "issue.updated"; payload: SimIssue }
  | { kind: "channel.created"; payload: SimChannel }
  | { kind: "post.created"; payload: SimPost };

export type SimSubscriber = (event: SimEvent) => void;

type GlobalWithBus = typeof globalThis & {
  __genasisSimSubscribers?: Set<SimSubscriber>;
};

const g = globalThis as GlobalWithBus;
const subscribers: Set<SimSubscriber> = (g.__genasisSimSubscribers ??=
  new Set<SimSubscriber>());

export function subscribe(fn: SimSubscriber): () => void {
  subscribers.add(fn);
  return () => {
    subscribers.delete(fn);
  };
}

export function emit(event: SimEvent): void {
  for (const fn of [...subscribers]) {
    try {
      fn(event);
    } catch {
      subscribers.delete(fn);
    }
  }
}

export function subscriberCount(): number {
  return subscribers.size;
}
