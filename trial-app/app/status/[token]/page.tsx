import { notFound } from "next/navigation";

import { CredentialsView } from "@/app/components/CredentialsView";
import { getSubmissionByToken, type Credentials, type SubmissionRow } from "@/db";
import { generateGenasisToml } from "@/lib/genasis-toml";

export const runtime = "nodejs";
export const dynamic = "force-dynamic";

function parseCredentials(json: string | null): Credentials | null {
  if (!json) return null;
  try {
    return JSON.parse(json) as Credentials;
  } catch {
    return null;
  }
}

function parseTechStack(json: string | null): string[] {
  if (!json) return [];
  try {
    const v = JSON.parse(json);
    return Array.isArray(v) ? (v as string[]) : [];
  } catch {
    return [];
  }
}

export default async function StatusPage({
  params,
}: {
  params: Promise<{ token: string }>;
}) {
  const { token } = await params;
  const submission = getSubmissionByToken(token);
  if (!submission) notFound();

  const techStack = parseTechStack(submission.tech_stack);

  return (
    <main
      className="mx-auto max-w-3xl space-y-6 px-6 py-10"
      data-testid="status-page"
      data-status={submission.status}
    >
      <header className="space-y-2">
        <h1 className="text-2xl font-semibold">신청 상태</h1>
        <p className="text-xs text-neutral-500">
          Token: <code className="font-mono">{token}</code>
        </p>
      </header>

      {submission.status === "pending" ? (
        <PendingCard submission={submission} techStack={techStack} />
      ) : null}

      {submission.status === "provisioned" ? (
        <ProvisionedView submission={submission} techStack={techStack} />
      ) : null}

      {submission.status === "revoked" ? (
        <RevokedCard submission={submission} />
      ) : null}
    </main>
  );
}

function SubmissionSummary({
  submission,
  techStack,
}: {
  submission: SubmissionRow;
  techStack: string[];
}) {
  return (
    <dl
      data-testid="submission-summary"
      className="grid grid-cols-[max-content_1fr] gap-x-6 gap-y-2 text-sm"
    >
      <dt className="text-neutral-600 dark:text-neutral-400">Name</dt>
      <dd>{submission.name}</dd>
      <dt className="text-neutral-600 dark:text-neutral-400">Email</dt>
      <dd>{submission.email}</dd>
      {submission.phone ? (
        <>
          <dt className="text-neutral-600 dark:text-neutral-400">Phone</dt>
          <dd>{submission.phone}</dd>
        </>
      ) : null}
      <dt className="text-neutral-600 dark:text-neutral-400">Project</dt>
      <dd>{submission.project_name}</dd>
      <dt className="text-neutral-600 dark:text-neutral-400">Team size</dt>
      <dd>{submission.team_size}</dd>
      {techStack.length > 0 ? (
        <>
          <dt className="text-neutral-600 dark:text-neutral-400">Stack</dt>
          <dd>{techStack.join(", ")}</dd>
        </>
      ) : null}
      {submission.message ? (
        <>
          <dt className="text-neutral-600 dark:text-neutral-400">Message</dt>
          <dd className="whitespace-pre-wrap">{submission.message}</dd>
        </>
      ) : null}
      <dt className="text-neutral-600 dark:text-neutral-400">Submitted</dt>
      <dd>
        <time dateTime={submission.created_at}>{submission.created_at}</time>
      </dd>
    </dl>
  );
}

function PendingCard({
  submission,
  techStack,
}: {
  submission: SubmissionRow;
  techStack: string[];
}) {
  return (
    <section
      data-testid="status-pending"
      className="space-y-4 rounded-lg border border-yellow-200 bg-yellow-50 p-5 dark:border-yellow-900 dark:bg-yellow-950/40"
    >
      <div className="space-y-1">
        <p className="font-semibold text-yellow-900 dark:text-yellow-200">
          ⏳ Pending — 관리자 검토 중입니다
        </p>
        <p className="text-sm text-yellow-800 dark:text-yellow-300">
          관리자가 Plane + Mattermost 환경을 준비하면 이 페이지에 자격증명이
          표시됩니다. 이 URL을 북마크해두세요.
        </p>
      </div>
      <SubmissionSummary submission={submission} techStack={techStack} />
    </section>
  );
}

function ProvisionedView({
  submission,
  techStack,
}: {
  submission: SubmissionRow;
  techStack: string[];
}) {
  const credentials = parseCredentials(submission.credentials_json);
  if (!credentials) {
    return (
      <section
        data-testid="status-provisioned-error"
        className="rounded-lg border border-red-200 bg-red-50 p-5 text-sm text-red-800 dark:border-red-900 dark:bg-red-950/40 dark:text-red-300"
      >
        자격증명 페이로드를 읽을 수 없습니다. 관리자에게 문의해주세요.
      </section>
    );
  }
  const tomlSnippet = generateGenasisToml(submission.project_name, credentials);

  return (
    <div data-testid="status-provisioned" className="space-y-6">
      <section className="space-y-1 rounded-lg border border-green-200 bg-green-50 p-5 dark:border-green-900 dark:bg-green-950/40">
        <p className="font-semibold text-green-900 dark:text-green-200">
          ✅ Provisioned — 자격증명이 발급됐습니다
        </p>
        <p className="text-sm text-green-800 dark:text-green-300">
          아래 자격증명을 안전하게 보관해주세요. 비밀 항목은 기본적으로
          가려져있고 Show 버튼으로 확인할 수 있습니다.
        </p>
      </section>
      <SubmissionSummary submission={submission} techStack={techStack} />
      <CredentialsView credentials={credentials} tomlSnippet={tomlSnippet} />
    </div>
  );
}

function RevokedCard({ submission }: { submission: SubmissionRow }) {
  return (
    <section
      data-testid="status-revoked"
      className="rounded-lg border border-neutral-300 bg-neutral-100 p-5 text-sm text-neutral-800 dark:border-neutral-700 dark:bg-neutral-900 dark:text-neutral-200"
    >
      <p className="font-semibold">🚫 Revoked — 체험 환경이 회수됐습니다</p>
      <p className="mt-1">
        프로젝트 <code>{submission.project_name}</code> 의 체험 환경이 관리자에
        의해 회수됐습니다. 다시 신청하시거나 관리자에게 문의해주세요.
      </p>
    </section>
  );
}
