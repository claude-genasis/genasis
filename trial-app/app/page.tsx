import { AppBar, type TrialTab } from "@/app/components/AppBar";
import { DemoBoard } from "@/app/components/DemoBoard";
import { SignupForm } from "@/app/components/SignupForm";

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
