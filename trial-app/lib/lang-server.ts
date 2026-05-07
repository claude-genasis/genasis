import "server-only";

import { cookies } from "next/headers";

import { LANG_COOKIE, isLang, type Lang } from "@/lib/i18n";

export async function getLang(): Promise<Lang> {
  const c = await cookies();
  const v = c.get(LANG_COOKIE)?.value;
  return isLang(v) ? v : "ko";
}
