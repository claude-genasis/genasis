import { NextResponse } from "next/server";
import { z } from "zod";

import { insertSubmission } from "@/db";
import { generateToken } from "@/lib/token";

export const runtime = "nodejs";
export const dynamic = "force-dynamic";

const SubmissionSchema = z.object({
  name: z.string().trim().min(1, "name required"),
  email: z.email("invalid email").trim(),
  phone: z.string().optional(),
  projectName: z.string().trim().min(1, "projectName required"),
  teamSize: z.enum(["solo", "small", "medium"]),
  techStack: z.array(z.string()).optional(),
  message: z.string().optional(),
});

export type SubmissionInput = z.infer<typeof SubmissionSchema>;

const MM_BASE_URL = process.env.MM_BASE_URL ?? "https://mm.realstory.blog";

function formatMattermostMessage(
  input: SubmissionInput,
  techStack: string[],
  submittedAt: string,
): string {
  return [
    "🆕 **Genasis Trial Request**",
    `• Name: ${input.name}`,
    `• Email: ${input.email}`,
    `• Phone: ${input.phone?.trim() || "(none)"}`,
    `• Project: ${input.projectName}`,
    `• Team size: ${input.teamSize}`,
    `• Stack: ${techStack.length > 0 ? techStack.join(", ") : "(none)"}`,
    `• Message: ${input.message?.trim() || "(none)"}`,
    `• Submitted: ${submittedAt}`,
  ].join("\n");
}

type NotifyResult =
  | { kind: "sent" }
  | { kind: "skipped" }
  | { kind: "failed"; reason: string };

async function notifyMattermost(message: string): Promise<NotifyResult> {
  const token = process.env.MM_BOT_TOKEN;
  const channelId = process.env.MM_TRIAL_CHANNEL_ID;
  if (!token || !channelId) return { kind: "skipped" };
  try {
    const res = await fetch(`${MM_BASE_URL}/api/v4/posts`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${token}`,
      },
      body: JSON.stringify({ channel_id: channelId, message }),
    });
    if (!res.ok) {
      const detail = await res.text().catch(() => "");
      return {
        kind: "failed",
        reason: `${res.status} ${detail || res.statusText}`.trim(),
      };
    }
    return { kind: "sent" };
  } catch (err) {
    return {
      kind: "failed",
      reason: err instanceof Error ? err.message : String(err),
    };
  }
}

export async function POST(req: Request) {
  let raw: unknown;
  try {
    raw = await req.json();
  } catch {
    return NextResponse.json({ error: "invalid_json" }, { status: 400 });
  }

  const parsed = SubmissionSchema.safeParse(raw);
  if (!parsed.success) {
    return NextResponse.json(
      {
        error: "validation_failed",
        issues: parsed.error.issues.map((i) => ({
          path: i.path,
          message: i.message,
        })),
      },
      { status: 400 },
    );
  }

  const input = parsed.data;
  const techStack = input.techStack ?? [];
  const phone = input.phone?.trim() || null;
  const message = input.message?.trim() || null;
  const token = generateToken();
  const submittedAt = new Date().toISOString();

  insertSubmission({
    token,
    name: input.name,
    email: input.email,
    phone,
    projectName: input.projectName,
    teamSize: input.teamSize,
    techStack,
    message,
  });

  const notify = await notifyMattermost(
    formatMattermostMessage(input, techStack, submittedAt),
  );

  const statusUrl = `/status/${token}`;

  if (notify.kind === "failed") {
    return NextResponse.json(
      {
        error: "mattermost_notify_failed",
        reason: notify.reason,
        token,
        statusUrl,
      },
      { status: 500 },
    );
  }

  return NextResponse.json(
    { token, statusUrl, notification: notify.kind },
    { status: 200 },
  );
}
