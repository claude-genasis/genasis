import type { Credentials } from "@/db";

function safeIdent(name: string): string {
  return name.replace(/[^a-zA-Z0-9_-]/g, "_");
}

function tomlString(value: string): string {
  return `"${value.replace(/\\/g, "\\\\").replace(/"/g, '\\"')}"`;
}

export function generateGenasisToml(
  projectName: string,
  credentials: Credentials,
): string {
  const sections: string[] = [];
  sections.push(`[project]
name = ${tomlString(safeIdent(projectName))}
`);
  sections.push(`[plane]
url = ${tomlString(credentials.plane.url)}
workspace_slug = ${tomlString(credentials.plane.workspace_slug)}
flavor = "auto"
# Set PLANE_API_KEY in your environment from the Plane API key
# below or via your secret manager.
`);
  sections.push(`[mattermost]
url = ${tomlString(credentials.mattermost.url)}
flavor = "auto"
# Set MM_ADMIN_TOKEN in your environment from the bot tokens
# section below.
`);
  const botEntries = Object.entries(credentials.mattermost.bot_tokens)
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([role, token]) => `${safeIdent(role)} = ${tomlString(token)}`)
    .join("\n");
  if (botEntries.length > 0) {
    sections.push(`[mattermost.bot_tokens]\n${botEntries}\n`);
  }
  return sections.join("\n");
}
