"use client";

import { useCallback, useEffect, useRef, useState } from "react";

import type { ChatMessage } from "@/app/components/ChatThread";
import type { KanbanCard } from "@/app/components/KanbanBoard";
import {
  DEMO_STEPS,
  INITIAL_CARDS,
  INITIAL_MESSAGES,
  TYPING_LEAD_MS,
} from "@/lib/demo-script";

export type SprintStatus = "idle" | "running" | "complete";

export type SprintState = {
  cards: KanbanCard[];
  messages: ChatMessage[];
  typingActor: string | null;
  status: SprintStatus;
  completedSteps: number;
};

const INITIAL_STATE: SprintState = {
  cards: INITIAL_CARDS,
  messages: INITIAL_MESSAGES,
  typingActor: null,
  status: "idle",
  completedSteps: 0,
};

export function useDemoSprint() {
  const [state, setState] = useState<SprintState>(INITIAL_STATE);
  const timersRef = useRef<ReturnType<typeof setTimeout>[]>([]);

  const clearTimers = useCallback(() => {
    timersRef.current.forEach(clearTimeout);
    timersRef.current = [];
  }, []);

  const reset = useCallback(() => {
    clearTimers();
    setState(INITIAL_STATE);
  }, [clearTimers]);

  const run = useCallback(() => {
    clearTimers();
    setState({
      cards: INITIAL_CARDS,
      messages: INITIAL_MESSAGES,
      typingActor: null,
      status: "running",
      completedSteps: 0,
    });
    DEMO_STEPS.forEach((step, idx) => {
      const isLast = idx === DEMO_STEPS.length - 1;
      if (step.message) {
        const typingAt = Math.max(0, step.offsetMs - TYPING_LEAD_MS);
        const typingActor = step.message.actor;
        const t1 = setTimeout(() => {
          setState((s) => ({ ...s, typingActor }));
        }, typingAt);
        timersRef.current.push(t1);
      }
      const t2 = setTimeout(() => {
        setState((s) => {
          let cards = s.cards;
          if (step.kanbanOp) {
            const op = step.kanbanOp;
            if (op.kind === "create") {
              cards = [
                ...cards,
                { id: op.id, title: op.title, column: op.column },
              ];
            } else {
              cards = cards.map((c) =>
                c.id === op.id ? { ...c, column: op.to } : c,
              );
            }
          }
          const messages = step.message
            ? [...s.messages, step.message]
            : s.messages;
          return {
            cards,
            messages,
            typingActor: null,
            status: isLast ? "complete" : "running",
            completedSteps: idx + 1,
          };
        });
      }, step.offsetMs);
      timersRef.current.push(t2);
    });
  }, [clearTimers]);

  useEffect(() => {
    return () => {
      clearTimers();
    };
  }, [clearTimers]);

  return { ...state, run, reset };
}
