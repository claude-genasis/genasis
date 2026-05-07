import type { ChatMessage } from "@/app/components/ChatThread";
import type { KanbanCard, KanbanColumn } from "@/app/components/KanbanBoard";

export type KanbanOp =
  | { kind: "create"; id: number; title: string; column: KanbanColumn }
  | { kind: "move"; id: number; to: KanbanColumn };

export type DemoStep = {
  offsetMs: number;
  message?: ChatMessage;
  kanbanOp?: KanbanOp;
};

export const TYPING_LEAD_MS = 600;

export const INITIAL_CARDS: KanbanCard[] = [];
export const INITIAL_MESSAGES: ChatMessage[] = [];

export const DEMO_STEPS: DemoStep[] = [
  {
    offsetMs: 0,
    kanbanOp: { kind: "create", id: 1, title: "Add login page", column: "todo" },
    message: {
      time: "14:00",
      actor: "pm",
      text: "Created issue #1 — Add login page",
    },
  },
  {
    offsetMs: 2_000,
    message: { time: "14:01", actor: "pm", text: "#1 assigned to frontend" },
  },
  {
    offsetMs: 3_000,
    kanbanOp: { kind: "move", id: 1, to: "inprogress" },
    message: { time: "14:02", actor: "frontend", text: "Starting work on #1" },
  },
  {
    offsetMs: 6_000,
    message: {
      time: "14:05",
      actor: "frontend",
      text: "PR #1 ready for review",
    },
  },
  {
    offsetMs: 7_000,
    message: {
      time: "14:06",
      actor: "code-reviewer",
      text: "LGTM, minor nit on L42",
    },
  },
  {
    offsetMs: 9_000,
    message: { time: "14:07", actor: "frontend", text: "Fixed — PR updated" },
  },
  {
    offsetMs: 10_000,
    message: { time: "14:08", actor: "qa", text: "Running test suite..." },
  },
  {
    offsetMs: 12_000,
    kanbanOp: { kind: "move", id: 1, to: "done" },
    message: { time: "14:09", actor: "qa", text: "✅ All tests passed" },
  },
];

export const DEMO_DURATION_MS =
  DEMO_STEPS[DEMO_STEPS.length - 1]!.offsetMs + 500;
