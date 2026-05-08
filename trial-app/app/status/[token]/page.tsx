import { notFound } from "next/navigation";

import { CredentialsView } from "@/app/components/CredentialsView";
import { getSubmissionByToken, type Credentials, type SubmissionRow } from "@/db";
import { generateGenasisToml } from "@/lib/genasis-toml";
import { t, type Lang } from "@/lib/i18n";
import { getLang } from "@/lib/lang-server";

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

  const lang = await getLang();
  const techStack = parseTechStack(submission.tech_stack);

  return (
    <main
      className="mx-auto max-w-3xl space-y-6 px-6 py-10"
      data-testid="status-page"
      data-status={submission.status}
    >
      <header className="space-y-2">
        <h1 className="text-2xl font-semibold">{t(lang, "status.heading")}</h1>
        <p className="text-xs text-neutral-500">
          {t(lang, "status.token")}{" "}
          <code className="font-mono">{token}</code>
        </p>
      </header>

      {submission.status === "pending" ? (
        <PendingCard
          submission={submission}
          techStack={techStack}
          lang={lang}
        />
      ) : null}

      {submission.status === "provisioned" ? (
        <ProvisionedView
          submission={submission}
          techStack={techStack}
          lang={lang}
        />
      ) : null}

      {submission.status === "revoked" ? (
        <RevokedCard submission={submission} lang={lang} />
      ) : null}
    </main>
  );
}

function SubmissionSummary({
  submission,
  techStack,
  lang,
}: {
  submission: SubmissionRow;
  techStack: string[];
  lang: Lang;
}) {
  return (
    <dl
      data-testid="submission-summary"
      className="grid grid-cols-[max-content_1fr] gap-x-6 gap-y-2 text-sm"
    >
      <dt className="text-neutral-600 dark:text-neutral-400">
        {t(lang, "status.summary.name")}
      </dt>
      <dd>{submission.name}</dd>
      <dt className="text-neutral-600 dark:text-neutral-400">
        {t(lang, "status.summary.email")}
      </dt>
      <dd>{submission.email}</dd>
      {submission.phone ? (
        <>
          <dt className="text-neutral-600 dark:text-neutral-400">
            {t(lang, "status.summary.phone")}
          </dt>
          <dd>{submission.phone}</dd>
        </>
      ) : null}
      <dt className="text-neutral-600 dark:text-neutral-400">
        {t(lang, "status.summary.project")}
      </dt>
      <dd>{submission.project_name}</dd>
      <dt className="text-neutral-600 dark:text-neutral-400">
        {t(lang, "status.summary.teamSize")}
      </dt>
      <dd>{submission.team_size}</dd>
      {techStack.length > 0 ? (
        <>
          <dt className="text-neutral-600 dark:text-neutral-400">
            {t(lang, "status.summary.stack")}
          </dt>
          <dd>{techStack.join(", ")}</dd>
        </>
      ) : null}
      {submission.message ? (
        <>
          <dt className="text-neutral-600 dark:text-neutral-400">
            {t(lang, "status.summary.message")}
          </dt>
          <dd className="whitespace-pre-wrap">{submission.message}</dd>
        </>
      ) : null}
      <dt className="text-neutral-600 dark:text-neutral-400">
        {t(lang, "status.summary.submitted")}
      </dt>
      <dd>
        <time dateTime={submission.created_at}>{submission.created_at}</time>
      </dd>
    </dl>
  );
}

function PendingCard({
  submission,
  techStack,
  lang,
}: {
  submission: SubmissionRow;
  techStack: string[];
  lang: Lang;
}) {
  return (
    <section
      data-testid="status-pending"
      className="space-y-4 rounded-lg border border-yellow-200 bg-yellow-50 p-5 dark:border-yellow-900 dark:bg-yellow-950/40"
    >
      <div className="space-y-1">
        <h2 className="font-semibold text-yellow-900 dark:text-yellow-200">
          {t(lang, "status.pending.title")}
        </h2>
        <p className="text-sm text-yellow-800 dark:text-yellow-300">
          {t(lang, "status.pending.body")}
        </p>
      </div>
      <SubmissionSummary
        submission={submission}
        techStack={techStack}
        lang={lang}
      />
    </section>
  );
}

function ProvisionedView({
  submission,
  techStack,
  lang,
}: {
  submission: SubmissionRow;
  techStack: string[];
  lang: Lang;
}) {
  const credentials = parseCredentials(submission.credentials_json);
  if (!credentials) {
    return (
      <section
        data-testid="status-provisioned-error"
        className="rounded-lg border border-red-200 bg-red-50 p-5 text-sm text-red-800 dark:border-red-900 dark:bg-red-950/40 dark:text-red-300"
      >
        {t(lang, "status.provisioned.errorParse")}
      </section>
    );
  }
  const tomlSnippet = generateGenasisToml(submission.project_name, credentials);

  return (
    <div data-testid="status-provisioned" className="space-y-6">
      <section className="space-y-1 rounded-lg border border-green-200 bg-green-50 p-5 dark:border-green-900 dark:bg-green-950/40">
        <h2 className="font-semibold text-green-900 dark:text-green-200">
          {t(lang, "status.provisioned.title")}
        </h2>
        <p className="text-sm text-green-800 dark:text-green-300">
          {t(lang, "status.provisioned.body")}
        </p>
      </section>
      <SubmissionSummary
        submission={submission}
        techStack={techStack}
        lang={lang}
      />
      <CredentialsView credentials={credentials} tomlSnippet={tomlSnippet} />
    </div>
  );
}

function RevokedCard({
  submission,
  lang,
}: {
  submission: SubmissionRow;
  lang: Lang;
}) {
  return (
    <section
      data-testid="status-revoked"
      className="rounded-lg border border-neutral-300 bg-neutral-100 p-5 text-sm text-neutral-800 dark:border-neutral-700 dark:bg-neutral-900 dark:text-neutral-200"
    >
      <h2 className="font-semibold">{t(lang, "status.revoked.title")}</h2>
      <p className="mt-1">
        {t(lang, "status.revoked.body", {
          project: submission.project_name,
        })}
      </p>
    </section>
  );
}
