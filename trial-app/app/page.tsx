import { AppBar, type TrialTab } from "@/app/components/AppBar";
import { KanbanBoard, type KanbanCard } from "@/app/components/KanbanBoard";

const DEMO_INITIAL_CARDS: KanbanCard[] = [
  { id: 1, title: "Add login page", column: "todo" },
  { id: 2, title: "Wire up auth API", column: "todo" },
  { id: 3, title: "Draft README", column: "todo" },
];

function resolveTab(raw: string | string[] | undefined): TrialTab {
  const value = Array.isArray(raw) ? raw[0] : raw;
  return value === "signup" ? "signup" : "demo";
}

export default async function HomePage({
  searchParams,
}: {
  searchParams: Promise<{ tab?: string | string[] }>;
}) {
  const params = await searchParams;
  const activeTab = resolveTab(params.tab);

  return (
    <div className="flex min-h-screen flex-col">
      <AppBar activeTab={activeTab} />
      <main className="flex-1 px-6 py-10">
        {activeTab === "demo" ? <DemoSection /> : <SignupSection />}
      </main>
    </div>
  );
}

function DemoSection() {
  return (
    <section
      aria-labelledby="demo-heading"
      className="mx-auto max-w-5xl space-y-3"
    >
      <h1 id="demo-heading" className="text-2xl font-semibold">
        체험하기
      </h1>
      <p className="text-sm text-neutral-500">
        에이전트 팀의 칸반과 채팅 흐름을 미리 보여드립니다. 인터랙티브 데모는 곧 이어집니다.
      </p>
      <KanbanBoard cards={DEMO_INITIAL_CARDS} />
    </section>
  );
}

function SignupSection() {
  return (
    <section
      aria-labelledby="signup-heading"
      className="mx-auto max-w-5xl space-y-3"
    >
      <h1 id="signup-heading" className="text-2xl font-semibold">
        신청하기
      </h1>
      <p className="text-sm text-neutral-500">
        호스팅된 Plane + Mattermost 체험 환경을 신청할 수 있는 폼이 곧 제공됩니다.
      </p>
    </section>
  );
}
