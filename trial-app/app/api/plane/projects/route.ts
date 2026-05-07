import { NextResponse } from "next/server";
import { z } from "zod";

import { ensureProject } from "@/db/sim";
import { checkTrialSecret } from "@/lib/trial-auth";

export const runtime = "nodejs";
export const dynamic = "force-dynamic";

const ProjectSchema = z.object({
  slug: z
    .string()
    .min(1)
    .regex(/^[a-z0-9][a-z0-9-]*$/i, "slug must be alphanumeric (with optional hyphens)"),
  name: z.string().min(1),
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
  const parsed = ProjectSchema.safeParse(raw);
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
  const project = ensureProject(parsed.data);
  return NextResponse.json(project, { status: 200 });
}
