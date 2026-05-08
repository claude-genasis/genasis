"use client";

import Link from "next/link";

import { LangSwitcher } from "@/app/components/LangSwitcher";
import { useLang } from "@/app/components/LangProvider";

export type TrialTab = "demo" | "live" | "signup";

const TABS: { key: TrialTab; labelKey: string }[] = [
  { key: "demo", labelKey: "nav.tab.demo" },
  { key: "live", labelKey: "nav.tab.live" },
  { key: "signup", labelKey: "nav.tab.signup" },
];

export function AppBar({ activeTab }: { activeTab: TrialTab }) {
  const { t } = useLang();
  return (
    <header className="sticky top-0 z-50 flex items-center justify-between border-b border-neutral-200 bg-white/80 px-6 py-3 backdrop-blur dark:border-neutral-800 dark:bg-neutral-950/80">
      <Link
        href={{ pathname: "/", query: { tab: "demo" } }}
        className="text-lg font-semibold tracking-tight"
      >
        {t("nav.brand")}
      </Link>
      <div className="flex items-center gap-3">
        <nav aria-label="Trial sections" className="flex items-center gap-1">
          {TABS.map(({ key, labelKey }) => {
            const isActive = key === activeTab;
            return (
              <Link
                key={key}
                href={{ pathname: "/", query: { tab: key } }}
                aria-current={isActive ? "page" : undefined}
                className={
                  "rounded-md px-3 py-1.5 text-sm font-medium transition-colors " +
                  (isActive
                    ? "bg-neutral-900 text-white dark:bg-white dark:text-neutral-900"
                    : "text-neutral-600 hover:bg-neutral-100 dark:text-neutral-300 dark:hover:bg-neutral-800")
                }
              >
                {t(labelKey)}
              </Link>
            );
          })}
        </nav>
        <LangSwitcher />
      </div>
    </header>
  );
}
