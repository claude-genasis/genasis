"use client";

import { ChatSidebar } from "@/app/components/ChatSidebar";
import { useLang } from "@/app/components/LangProvider";
import { LiveChatThread } from "@/app/components/LiveChatThread";
import { LiveKanbanBoard } from "@/app/components/LiveKanbanBoard";
import type { SimChannel, SimIssue, SimPost, SimProject } from "@/db/sim";

export type LiveBoardProps = {
  project: SimProject;
  channel: SimChannel;
  initialIssues: SimIssue[];
  initialPosts: SimPost[];
};

export function LiveBoard({
  project,
  channel,
  initialIssues,
  initialPosts,
}: LiveBoardProps) {
  const { t } = useLang();
  return (
    <div className="space-y-4" data-testid="live-board">
      <div
        className="rounded-md border border-neutral-200 bg-neutral-50 px-4 py-2 text-xs text-neutral-600 dark:border-neutral-800 dark:bg-neutral-900 dark:text-neutral-300"
        data-testid="live-banner"
      >
        <span>
          {t("live.banner", { project: project.slug, channel: channel.name })}
        </span>
      </div>
      <div className="relative" data-testid="live-board-stage">
        <LiveKanbanBoard
          initialIssues={initialIssues}
          projectSlug={project.slug}
        />
        <ChatSidebar channelName={channel.name}>
          <LiveChatThread
            initialPosts={initialPosts}
            channelId={channel.id}
            channelName={channel.name}
          />
        </ChatSidebar>
      </div>
    </div>
  );
}
