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
  return (
    <div className="space-y-4" data-testid="live-board">
      <div className="rounded-md border border-neutral-200 bg-neutral-50 px-4 py-2 text-xs text-neutral-600 dark:border-neutral-800 dark:bg-neutral-900 dark:text-neutral-300">
        <span>
          프로젝트{" "}
          <code className="font-mono text-neutral-900 dark:text-neutral-100">
            {project.slug}
          </code>{" "}
          · 채널{" "}
          <code className="font-mono text-neutral-900 dark:text-neutral-100">
            #{channel.name}
          </code>{" "}
          · 카드를 드래그하거나 메시지를 보내면 에이전트와 같은 데이터에 반영됩니다.
        </span>
      </div>
      <LiveKanbanBoard
        initialIssues={initialIssues}
        projectSlug={project.slug}
      />
      <LiveChatThread
        initialPosts={initialPosts}
        channelId={channel.id}
        channelName={channel.name}
      />
    </div>
  );
}
