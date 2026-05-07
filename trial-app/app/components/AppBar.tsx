import Link from "next/link";

export type TrialTab = "demo" | "live" | "signup";

const TABS: { key: TrialTab; label: string }[] = [
  { key: "demo", label: "체험하기" },
  { key: "live", label: "라이브 트라이얼" },
  { key: "signup", label: "신청하기" },
];

export function AppBar({ activeTab }: { activeTab: TrialTab }) {
  return (
    <header className="sticky top-0 z-10 flex items-center justify-between border-b border-neutral-200 bg-white/80 px-6 py-3 backdrop-blur dark:border-neutral-800 dark:bg-neutral-950/80">
      <Link
        href={{ pathname: "/", query: { tab: "demo" } }}
        className="text-lg font-semibold tracking-tight"
      >
        Genasis Trial
      </Link>
      <nav aria-label="Trial sections" className="flex items-center gap-1">
        {TABS.map(({ key, label }) => {
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
              {label}
            </Link>
          );
        })}
      </nav>
    </header>
  );
}
