import { NextResponse } from "next/server";
import { z } from "zod";

import { updateSubmissionCredentials } from "@/db";

export const runtime = "nodejs";
export const dynamic = "force-dynamic";

const WebhookSchema = z.object({
  token: z.string().min(1),
  plane: z.object({
    url: z.url(),
    login: z.string(),
    password: z.string(),
    api_key: z.string(),
    workspace_slug: z.string(),
  }),
  mattermost: z.object({
    url: z.url(),
    login: z.string(),
    password: z.string(),
    bot_tokens: z.record(z.string(), z.string()),
  }),
});

export async function POST(req: Request) {
  const expected = process.env.WEBHOOK_SHARED_SECRET;
  if (!expected) {
    return NextResponse.json(
      { error: "webhook_not_configured" },
      { status: 503 },
    );
  }
  const provided = req.headers.get("x-genasis-webhook-secret");
  if (!provided || provided !== expected) {
    return NextResponse.json({ error: "unauthorized" }, { status: 401 });
  }

  let raw: unknown;
  try {
    raw = await req.json();
  } catch {
    return NextResponse.json({ error: "invalid_json" }, { status: 400 });
  }

  const parsed = WebhookSchema.safeParse(raw);
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

  const { token, plane, mattermost } = parsed.data;
  const updated = updateSubmissionCredentials(token, { plane, mattermost });
  if (!updated) {
    return NextResponse.json({ error: "not_found" }, { status: 404 });
  }
  return NextResponse.json(
    { ok: true, status: updated.status, token },
    { status: 200 },
  );
}
