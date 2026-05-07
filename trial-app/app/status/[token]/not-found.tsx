import Link from "next/link";

import { t } from "@/lib/i18n";
import { getLang } from "@/lib/lang-server";

export default async function StatusNotFound() {
  const lang = await getLang();
  return (
    <main
      className="mx-auto max-w-2xl space-y-4 px-6 py-16 text-center"
      data-testid="status-not-found"
    >
      <h1 className="text-2xl font-semibold">
        {t(lang, "status.notFound.title")}
      </h1>
      <p className="text-sm text-neutral-500">
        {t(lang, "status.notFound.body")}
      </p>
      <Link
        href="/?tab=signup"
        className="inline-block rounded-md bg-neutral-900 px-4 py-2 text-sm font-medium text-white hover:bg-neutral-700 dark:bg-white dark:text-neutral-900 dark:hover:bg-neutral-200"
      >
        {t(lang, "status.notFound.cta")}
      </Link>
    </main>
  );
}
