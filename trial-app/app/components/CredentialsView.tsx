"use client";

import { useState } from "react";

import type { Credentials } from "@/db";

export type CredentialsViewProps = {
  credentials: Credentials;
  tomlSnippet: string;
};

type Pair = { label: string; value: string; mask?: boolean };

export function CredentialsView({
  credentials,
  tomlSnippet,
}: CredentialsViewProps) {
  return (
    <div className="space-y-6" data-testid="credentials-view">
      <CredentialBlock
        title="Plane"
        testId="creds-plane"
        pairs={[
          { label: "URL", value: credentials.plane.url },
          { label: "Login", value: credentials.plane.login },
          {
            label: "Password",
            value: credentials.plane.password,
            mask: true,
          },
          { label: "API Key", value: credentials.plane.api_key, mask: true },
          { label: "Workspace slug", value: credentials.plane.workspace_slug },
        ]}
      />
      <CredentialBlock
        title="Mattermost"
        testId="creds-mattermost"
        pairs={[
          { label: "URL", value: credentials.mattermost.url },
          { label: "Login", value: credentials.mattermost.login },
          {
            label: "Password",
            value: credentials.mattermost.password,
            mask: true,
          },
        ]}
      />
      <BotTokens tokens={credentials.mattermost.bot_tokens} />
      <TomlSnippet snippet={tomlSnippet} />
    </div>
  );
}

function CredentialBlock({
  title,
  testId,
  pairs,
}: {
  title: string;
  testId: string;
  pairs: Pair[];
}) {
  return (
    <section
      data-testid={testId}
      className="space-y-2 rounded-lg border border-neutral-200 bg-white p-4 dark:border-neutral-800 dark:bg-neutral-950"
    >
      <h3 className="text-sm font-semibold">{title}</h3>
      <dl className="grid grid-cols-[max-content_1fr] gap-x-4 gap-y-1.5 text-sm">
        {pairs.map((p) => (
          <SecretRow key={p.label} pair={p} />
        ))}
      </dl>
    </section>
  );
}

function SecretRow({ pair }: { pair: Pair }) {
  const [shown, setShown] = useState(!pair.mask);
  const [copied, setCopied] = useState(false);
  const display = shown ? pair.value : "•".repeat(Math.min(pair.value.length, 12));

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(pair.value);
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    } catch {
      // ignore — older browsers
    }
  };

  return (
    <>
      <dt className="text-neutral-600 dark:text-neutral-400">{pair.label}</dt>
      <dd className="flex items-center gap-2 font-mono text-neutral-900 dark:text-neutral-100">
        <span data-testid={`creds-value-${pair.label.toLowerCase().replace(/\s+/g, "-")}`}>
          {display}
        </span>
        {pair.mask ? (
          <button
            type="button"
            onClick={() => setShown((s) => !s)}
            className="rounded border border-neutral-300 px-1.5 py-0.5 text-xs font-sans text-neutral-700 hover:bg-neutral-50 dark:border-neutral-700 dark:text-neutral-200 dark:hover:bg-neutral-900"
            aria-label={shown ? "Hide value" : "Show value"}
          >
            {shown ? "Hide" : "Show"}
          </button>
        ) : null}
        <button
          type="button"
          onClick={copy}
          className="rounded border border-neutral-300 px-1.5 py-0.5 text-xs font-sans text-neutral-700 hover:bg-neutral-50 dark:border-neutral-700 dark:text-neutral-200 dark:hover:bg-neutral-900"
        >
          {copied ? "Copied" : "Copy"}
        </button>
      </dd>
    </>
  );
}

function BotTokens({ tokens }: { tokens: Record<string, string> }) {
  return (
    <section
      data-testid="creds-bot-tokens"
      className="space-y-2 rounded-lg border border-neutral-200 bg-white p-4 dark:border-neutral-800 dark:bg-neutral-950"
    >
      <h3 className="text-sm font-semibold">Mattermost Bot Tokens</h3>
      <dl className="grid grid-cols-[max-content_1fr] gap-x-4 gap-y-1.5 text-sm">
        {Object.entries(tokens)
          .sort(([a], [b]) => a.localeCompare(b))
          .map(([role, token]) => (
            <SecretRow key={role} pair={{ label: role, value: token, mask: true }} />
          ))}
      </dl>
    </section>
  );
}

function TomlSnippet({ snippet }: { snippet: string }) {
  const [copied, setCopied] = useState(false);
  const copy = async () => {
    try {
      await navigator.clipboard.writeText(snippet);
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    } catch {
      // ignore
    }
  };
  return (
    <section
      data-testid="creds-toml-snippet"
      className="space-y-2 rounded-lg border border-neutral-200 bg-neutral-50 p-4 dark:border-neutral-800 dark:bg-neutral-900"
    >
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-semibold">genasis.toml</h3>
        <button
          type="button"
          onClick={copy}
          data-testid="creds-toml-copy"
          className="rounded-md border border-neutral-300 bg-white px-3 py-1 text-xs font-medium text-neutral-700 shadow-sm hover:bg-neutral-50 dark:border-neutral-700 dark:bg-neutral-950 dark:text-neutral-200 dark:hover:bg-neutral-900"
        >
          {copied ? "Copied" : "Copy"}
        </button>
      </div>
      <pre className="max-h-96 overflow-auto whitespace-pre rounded bg-white p-3 text-xs leading-relaxed dark:bg-neutral-950">
        <code>{snippet}</code>
      </pre>
    </section>
  );
}
