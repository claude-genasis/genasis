import { AppBar, type TrialTab } from "@/app/components/AppBar";
import { DemoBoard } from "@/app/components/DemoBoard";
import { LiveBoard } from "@/app/components/LiveBoard";
import { SignupForm } from "@/app/components/SignupForm";
import { ensureChannel, ensureProject, listIssues, listPosts } from "@/db/sim";

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

  return (
    <div className="flex min-h-screen flex-col">
      <AppBar activeTab={activeTab} />
      <main className="flex-1 px-6 py-10">
        {activeTab === "demo" ? <DemoSection /> : null}
        {activeTab === "live" ? <LiveSection /> : null}
        {activeTab === "signup" ? <SignupSection /> : null}
      </main>
    </div>
  );
}

function DemoSection() {
  return (
    <section
      aria-labelledby="demo-heading"
      className="mx-auto max-w-6xl space-y-4"
    >
      <h1 id="demo-heading" className="text-2xl font-semibold">
        체험하기
      </h1>
      <p className="text-sm text-neutral-500">
        에이전트 팀이 한 스프린트를 진행하는 모습을 미리 보여드립니다. 아래 Run 버튼을 누르면 PM·Frontend·Code-reviewer·QA가 #1 이슈를 함께 처리하는 흐름이 칸반과 채팅에서 동시에 재생됩니다.
      </p>
      <DemoBoard />
    </section>
  );
}

function LiveSection() {
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
        라이브 트라이얼
      </h1>
      <p className="text-sm text-neutral-500">
        에이전트 팀이 실제로 호출하는 Plane / Mattermost 시뮬레이터입니다.{" "}
        <code className="font-mono text-xs">genasis dev</code> 가 트라이얼 모드로
        실행되면 카드 생성·상태 변경·메시지가 이 화면에 라이브로 흘러들어옵니다.
        직접 카드를 끌어 옮기거나 메시지를 보내면 에이전트가 다음 폴링에서 그
        변화를 보게 됩니다.
      </p>
      <LiveBoard
        project={project}
        channel={channel}
        initialIssues={initialIssues}
        initialPosts={initialPosts}
      />
    </section>
  );
}

function SignupSection() {
  return (
    <section
      aria-labelledby="signup-heading"
      className="mx-auto max-w-3xl space-y-4"
    >
      <h1 id="signup-heading" className="text-2xl font-semibold">
        신청하기
      </h1>
      <p className="text-sm text-neutral-500">
        호스팅된 Plane + Mattermost 체험 환경을 신청해주세요. 관리자가 검토 후
        자격증명을 보내드립니다.
      </p>
      <SignupForm />
    </section>
  );
}
