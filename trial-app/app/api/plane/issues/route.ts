import { NextResponse } from "next/server";
import { z } from "zod";

import { createIssue, listIssues } from "@/db/sim";
import { checkTrialSecret } from "@/lib/trial-auth";

export const runtime = "nodejs";
export const dynamic = "force-dynamic";

const CreateIssueSchema = z.object({
  project_slug: z.string().min(1),
  title: z.string().min(1),
  assignee: z.string().optional(),
});

export async function POST(req: Request) {
  const auth = checkTrialSecret(req);
  if (auth) return auth;
  let raw: unknown;
  try {
    raw = await req.json();
  } catch {
    return NextResponse.json({ error: "invalid_json" }, { status: 400 });
  }
  const parsed = CreateIssueSchema.safeParse(raw);
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
  const issue = createIssue({
    project_slug: parsed.data.project_slug,
    title: parsed.data.title,
    assignee: parsed.data.assignee ?? null,
  });
  return NextResponse.json(issue, { status: 200 });
}

export async function GET(req: Request) {
  const auth = checkTrialSecret(req);
  if (auth) return auth;
  const url = new URL(req.url);
  const slug = url.searchParams.get("project_slug");
  if (!slug) {
    return NextResponse.json(
      { error: "missing_project_slug" },
      { status: 400 },
    );
  }
  const issues = listIssues({ project_slug: slug });
  return NextResponse.json({ issues }, { status: 200 });
}
