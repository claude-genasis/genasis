import { NextResponse } from "next/server";
import { z } from "zod";

import { ensureChannel } from "@/db/sim";
import { checkTrialSecret } from "@/lib/trial-auth";

export const runtime = "nodejs";
export const dynamic = "force-dynamic";

const ChannelSchema = z.object({
  name: z.string().min(1),
  display_name: z.string().min(1),
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
  const parsed = ChannelSchema.safeParse(raw);
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
  const channel = ensureChannel(parsed.data);
  return NextResponse.json(channel, { status: 200 });
}
