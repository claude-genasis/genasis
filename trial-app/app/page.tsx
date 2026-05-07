import { AppBar, type TrialTab } from "@/app/components/AppBar";
import { DemoBoard } from "@/app/components/DemoBoard";
import { LiveBoard } from "@/app/components/LiveBoard";
import { SignupForm } from "@/app/components/SignupForm";
import { ensureChannel, ensureProject, listIssues, listPosts } from "@/db/sim";
import { t, type Lang } from "@/lib/i18n";
import { getLang } from "@/lib/lang-server";

const LIVE_PROJECT_SLUG = "trial-demo";
const LIVE_PROJECT_NAME = "Genasis Trial Demo";
const LIVE_CHANNEL_NAME = "scrum-trial-demo";
const LIVE_CHANNEL_DISPLAY = "Trial Demo Scrum";

function resolveTab(raw: string | string[] | undefined): TrialTab {
  const value = Array.isArray(raw) ? raw[0] : raw;
  if (value === "signup") return "signup";
  if (value === "live") return "live";
  return "demo";
}

export default async function HomePage({
  searchParams,
}: {
  searchParams: Promise<{ tab?: string | string[] }>;
}) {
  const params = await searchParams;
  const activeTab = resolveTab(params.tab);
  const lang = await getLang();

  return (
    <div className="flex min-h-screen flex-col">
      <AppBar activeTab={activeTab} />
      <main className="flex-1 px-6 py-10">
        {activeTab === "demo" ? <DemoSection lang={lang} /> : null}
        {activeTab === "live" ? <LiveSection lang={lang} /> : null}
        {activeTab === "signup" ? <SignupSection lang={lang} /> : null}
      </main>
    </div>
  );
}

function DemoSection({ lang }: { lang: Lang }) {
  return (
    <section
      aria-labelledby="demo-heading"
      className="mx-auto max-w-6xl space-y-4"
    >
      <h1 id="demo-heading" className="text-2xl font-semibold">
        {t(lang, "demo.heading")}
      </h1>
      <p className="text-sm text-neutral-500">{t(lang, "demo.intro")}</p>
      <DemoBoard />
    </section>
  );
}

function LiveSection({ lang }: { lang: Lang }) {
  const project = ensureProject({
    slug: LIVE_PROJECT_SLUG,
    name: LIVE_PROJECT_NAME,
  });
  const channel = ensureChannel({
    name: LIVE_CHANNEL_NAME,
    display_name: LIVE_CHANNEL_DISPLAY,
  });
  const initialIssues = listIssues({ project_slug: project.slug });
  const initialPosts = listPosts({ channel_id: channel.id });

  return (
    <section
      aria-labelledby="live-heading"
      className="mx-auto max-w-6xl space-y-4"
      data-testid="live-section"
    >
      <h1 id="live-heading" className="text-2xl font-semibold">
        {t(lang, "live.heading")}
      </h1>
      <p className="text-sm text-neutral-500">{t(lang, "live.intro")}</p>
      <LiveBoard
        project={project}
        channel={channel}
        initialIssues={initialIssues}
        initialPosts={initialPosts}
      />
    </section>
  );
}

function SignupSection({ lang }: { lang: Lang }) {
  return (
    <section
      aria-labelledby="signup-heading"
      className="mx-auto max-w-3xl space-y-4"
    >
      <h1 id="signup-heading" className="text-2xl font-semibold">
        {t(lang, "signup.heading")}
      </h1>
      <p className="text-sm text-neutral-500">{t(lang, "signup.intro")}</p>
      <SignupForm />
    </section>
  );
}
