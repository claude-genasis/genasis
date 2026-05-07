"use client";

import { useLang } from "@/app/components/LangProvider";
import { LANG_LABELS, LANGS } from "@/lib/i18n";

export function LangSwitcher() {
  const { lang, setLang, t } = useLang();
  return (
    <div
      role="group"
      aria-label={t("lang.aria")}
      data-testid="lang-switcher"
      className="flex items-center gap-0.5 rounded-md border border-neutral-200 bg-white p-0.5 text-xs dark:border-neutral-700 dark:bg-neutral-950"
    >
      {LANGS.map((l) => {
        const active = l === lang;
        return (
          <button
            key={l}
            type="button"
            data-testid={`lang-${l}`}
            onClick={() => setLang(l)}
            aria-pressed={active}
            className={
              "rounded px-2 py-0.5 font-medium transition-colors " +
              (active
                ? "bg-neutral-900 text-white dark:bg-white dark:text-neutral-900"
                : "text-neutral-600 hover:bg-neutral-100 dark:text-neutral-300 dark:hover:bg-neutral-800")
            }
          >
            {LANG_LABELS[l]}
          </button>
        );
      })}
    </div>
  );
}
