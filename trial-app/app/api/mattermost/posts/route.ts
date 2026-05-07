import { NextResponse } from "next/server";
import { z } from "zod";

import { listPosts, postRoot, postThread } from "@/db/sim";
import { checkTrialSecret } from "@/lib/trial-auth";

export const runtime = "nodejs";
export const dynamic = "force-dynamic";

const PostSchema = z.object({
  channel_id: z.number().int().positive(),
  root_id: z.number().int().positive().optional(),
  actor: z.string().min(1),
  message: z.string().min(1),
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
  const parsed = PostSchema.safeParse(raw);
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
  const data = parsed.data;
  const post = data.root_id
    ? postThread({
        channel_id: data.channel_id,
        root_id: data.root_id,
        actor: data.actor,
        message: data.message,
      })
    : postRoot({
        channel_id: data.channel_id,
        actor: data.actor,
        message: data.message,
      });
  if (!post) {
    return NextResponse.json({ error: "not_found" }, { status: 404 });
  }
  return NextResponse.json(post, { status: 200 });
}

export async function GET(req: Request) {
  const auth = checkTrialSecret(req);
  if (auth) return auth;
  const url = new URL(req.url);
  const idParam = url.searchParams.get("channel_id");
  const nameParam = url.searchParams.get("channel_name");
  if (!idParam && !nameParam) {
    return NextResponse.json(
      { error: "missing_channel_id_or_name" },
      { status: 400 },
    );
  }
  const channel_id = idParam ? Number(idParam) : undefined;
  if (idParam && (channel_id === undefined || !Number.isInteger(channel_id))) {
    return NextResponse.json({ error: "invalid_channel_id" }, { status: 400 });
  }
  const posts = listPosts({
    channel_id,
    channel_name: nameParam ?? undefined,
  });
  return NextResponse.json({ posts }, { status: 200 });
}
