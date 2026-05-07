import { NextResponse } from "next/server";
import { z } from "zod";

import { getIssueById, transitionIssue } from "@/db/sim";
import { checkTrialSecret } from "@/lib/trial-auth";

export const runtime = "nodejs";
export const dynamic = "force-dynamic";

const PatchSchema = z.object({
  state: z.enum(["todo", "inprogress", "inreview", "done"]).optional(),
  assignee: z.union([z.string(), z.null()]).optional(),
});

function parseId(raw: string): number | null {
  const n = Number(raw);
  return Number.isInteger(n) && n > 0 ? n : null;
}

export async function PATCH(
  req: Request,
  ctx: { params: Promise<{ id: string }> },
) {
  const auth = checkTrialSecret(req);
  if (auth) return auth;
  const { id: rawId } = await ctx.params;
  const id = parseId(rawId);
  if (id === null) {
    return NextResponse.json({ error: "invalid_id" }, { status: 400 });
  }
  let raw: unknown;
  try {
    raw = await req.json();
  } catch {
    return NextResponse.json({ error: "invalid_json" }, { status: 400 });
  }
  const parsed = PatchSchema.safeParse(raw);
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
  const updated = transitionIssue(id, parsed.data);
  if (!updated) {
    return NextResponse.json({ error: "not_found" }, { status: 404 });
  }
  return NextResponse.json(updated, { status: 200 });
}

export async function GET(
  req: Request,
  ctx: { params: Promise<{ id: string }> },
) {
  const auth = checkTrialSecret(req);
  if (auth) return auth;
  const { id: rawId } = await ctx.params;
  const id = parseId(rawId);
  if (id === null) {
    return NextResponse.json({ error: "invalid_id" }, { status: 400 });
  }
  const issue = getIssueById(id);
  if (!issue) {
    return NextResponse.json({ error: "not_found" }, { status: 404 });
  }
  return NextResponse.json(issue, { status: 200 });
}
