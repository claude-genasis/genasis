export type Lang = "ko" | "en";

export const LANGS: readonly Lang[] = ["ko", "en"] as const;

export const LANG_LABELS: Record<Lang, string> = {
  ko: "한국어",
  en: "English",
};

export const LANG_COOKIE = "genasis_trial_lang";

type Dict = Record<string, string>;

const KO: Dict = {
  // App bar / global
  "nav.brand": "Genasis Trial",
  "nav.tab.demo": "체험하기",
  "nav.tab.live": "라이브 트라이얼",
  "nav.tab.signup": "신청하기",

  // Demo (scripted) section
  "demo.heading": "체험하기",
  "demo.intro":
    "에이전트 팀이 한 스프린트를 진행하는 모습을 미리 보여드립니다. 아래 Run 버튼을 누르면 PM·Frontend·Code-reviewer·QA가 #1 이슈를 함께 처리하는 흐름이 칸반과 채팅에서 동시에 재생됩니다.",
  "demo.run.idle": "▶ Run Demo Sprint",
  "demo.run.running": "재생 중…",
  "demo.run.complete": "▶ 다시 재생",
  "demo.reset": "Reset",
  "demo.status.idle":
    "대기 중 — Run 버튼을 누르면 8단계 데모가 재생됩니다.",
  "demo.status.running": "재생 중 — {completed} / {total} 단계",
  "demo.status.complete": "완료 — {total} / {total} 단계",
  "demo.chat.typingSuffix": "입력 중…",

  // Live (trial bridge) section
  "live.heading": "라이브 트라이얼",
  "live.intro":
    "에이전트 팀이 실제로 호출하는 Plane / Mattermost 시뮬레이터입니다. genasis dev 가 트라이얼 모드로 실행되면 카드 생성·상태 변경·메시지가 이 화면에 라이브로 흘러들어옵니다. 직접 카드를 끌어 옮기거나 메시지를 보내면 에이전트가 다음 폴링에서 그 변화를 보게 됩니다.",
  "live.banner":
    "프로젝트 {project} · 채널 #{channel} · 카드를 드래그하거나 메시지를 보내면 에이전트와 같은 데이터에 반영됩니다. 채팅은 좌측 사이드바 핸들로 열고 닫을 수 있습니다.",
  "live.empty.posts":
    "에이전트 호출을 기다리는 중… 직접 아래 입력창에 메시지를 보내볼 수도 있습니다.",
  "live.empty.cards": "Drop a card here",
  "live.kanban.col.todo": "Todo",
  "live.kanban.col.inprogress": "In Progress",
  "live.kanban.col.inreview": "In Review",
  "live.kanban.col.done": "Done",
  "live.kanban.error": "상태 변경 실패: {reason}",
  "live.chat.count": "{count}개 메시지 · 라이브",
  "live.chat.error": "메시지 전송 실패: {reason}",
  "live.chat.composer.placeholder":
    "메시지를 입력하고 Enter (Shift+Enter 줄바꿈)",
  "live.chat.send.idle": "Send",
  "live.chat.send.sending": "전송 중…",
  "sidebar.open": "사이드바 열기",
  "sidebar.close": "사이드바 닫기",

  // Signup form
  "signup.heading": "신청하기",
  "signup.intro":
    "호스팅된 Plane + Mattermost 체험 환경을 신청해주세요. 관리자가 검토 후 자격증명을 보내드립니다.",
  "signup.field.name": "이름",
  "signup.field.email": "이메일",
  "signup.field.phone": "전화번호",
  "signup.field.projectName": "프로젝트명",
  "signup.field.teamSize": "팀 규모",
  "signup.field.techStack": "기술 스택",
  "signup.field.message": "메시지",
  "signup.required.name": "이름을 입력해주세요.",
  "signup.required.email": "이메일을 입력해주세요.",
  "signup.required.email.format": "올바른 이메일 형식이 아닙니다.",
  "signup.required.projectName": "프로젝트명을 입력해주세요.",
  "signup.required.teamSize": "팀 규모를 선택해주세요.",
  "signup.optional": "(선택)",
  "signup.placeholder.email": "you@example.com",
  "signup.placeholder.phone": "010-0000-0000",
  "signup.placeholder.projectName": "my-cool-app",
  "signup.placeholder.message":
    "추가 컨텍스트가 있다면 자유롭게 적어주세요.",
  "signup.teamSize.placeholder": "선택해주세요…",
  "signup.teamSize.solo": "solo (1명)",
  "signup.teamSize.small": "small (2–5명)",
  "signup.teamSize.medium": "medium (6–10명)",
  "signup.submit.idle": "Submit Request",
  "signup.submit.sending": "전송 중…",
  "signup.submit.error.validation": "입력값을 확인해주세요.",
  "signup.submit.error.generic":
    "신청 중 오류가 발생했습니다. 잠시 후 다시 시도해주세요.",
  "signup.submit.error.network": "네트워크 오류가 발생했습니다. 다시 시도해주세요.",
  "signup.banner":
    "ℹ trial.realstory.blog에서 제공하는 공유 환경입니다. 관리자 협의 후 기간 제한 없이 이용 가능합니다.",

  // Status page
  "status.heading": "신청 상태",
  "status.token": "Token:",
  "status.summary.name": "Name",
  "status.summary.email": "Email",
  "status.summary.phone": "Phone",
  "status.summary.project": "Project",
  "status.summary.teamSize": "Team size",
  "status.summary.stack": "Stack",
  "status.summary.message": "Message",
  "status.summary.submitted": "Submitted",
  "status.pending.title": "⏳ Pending — 관리자 검토 중입니다",
  "status.pending.body":
    "관리자가 Plane + Mattermost 환경을 준비하면 이 페이지에 자격증명이 표시됩니다. 이 URL을 북마크해두세요.",
  "status.provisioned.title": "✅ Provisioned — 자격증명이 발급됐습니다",
  "status.provisioned.body":
    "아래 자격증명을 안전하게 보관해주세요. 비밀 항목은 기본적으로 가려져있고 Show 버튼으로 확인할 수 있습니다.",
  "status.provisioned.errorParse":
    "자격증명 페이로드를 읽을 수 없습니다. 관리자에게 문의해주세요.",
  "status.revoked.title": "🚫 Revoked — 체험 환경이 회수됐습니다",
  "status.revoked.body":
    "프로젝트 {project} 의 체험 환경이 관리자에 의해 회수됐습니다. 다시 신청하시거나 관리자에게 문의해주세요.",
  "status.notFound.title": "신청 정보를 찾을 수 없습니다",
  "status.notFound.body":
    "토큰이 잘못됐거나 만료됐을 수 있습니다. URL을 다시 확인해주세요.",
  "status.notFound.cta": "신청 페이지로 이동",

  // Credentials view
  "creds.show": "Show",
  "creds.hide": "Hide",
  "creds.copy": "Copy",
  "creds.copied": "Copied",
  "creds.plane": "Plane",
  "creds.mattermost": "Mattermost",
  "creds.botTokens": "Mattermost Bot Tokens",

  // Lang switcher
  "lang.aria": "언어",
};

const EN: Dict = {
  // App bar / global
  "nav.brand": "Genasis Trial",
  "nav.tab.demo": "Try it",
  "nav.tab.live": "Live trial",
  "nav.tab.signup": "Apply",

  // Demo
  "demo.heading": "Try it",
  "demo.intro":
    "A scripted preview of an agentic team working a sprint together. Press Run below to watch PM, Frontend, Code-reviewer, and QA take issue #1 from todo to done — kanban and chat update in lockstep.",
  "demo.run.idle": "▶ Run Demo Sprint",
  "demo.run.running": "Running…",
  "demo.run.complete": "▶ Replay",
  "demo.reset": "Reset",
  "demo.status.idle":
    "Idle — press Run to play the 8-step demo.",
  "demo.status.running": "Running — {completed} / {total} steps",
  "demo.status.complete": "Complete — {total} / {total} steps",
  "demo.chat.typingSuffix": "is typing…",

  // Live
  "live.heading": "Live trial",
  "live.intro":
    "A lightweight Plane / Mattermost simulator the agentic team actually calls into. When `genasis dev` runs in trial mode, card creates, state transitions, and chat messages flow into this screen live. Drag cards or send messages yourself and the agents will see those changes on their next poll.",
  "live.banner":
    "Project {project} · Channel #{channel} · Drag a card or send a message and you operate on the same data the agents see. Use the left handle to open or close the chat sidebar.",
  "live.empty.posts":
    "Waiting for agent activity… You can also send a message yourself from the composer below.",
  "live.empty.cards": "Drop a card here",
  "live.kanban.col.todo": "Todo",
  "live.kanban.col.inprogress": "In Progress",
  "live.kanban.col.inreview": "In Review",
  "live.kanban.col.done": "Done",
  "live.kanban.error": "Transition failed: {reason}",
  "live.chat.count": "{count} messages · live",
  "live.chat.error": "Send failed: {reason}",
  "live.chat.composer.placeholder":
    "Type a message and press Enter (Shift+Enter for newline)",
  "live.chat.send.idle": "Send",
  "live.chat.send.sending": "Sending…",
  "sidebar.open": "Open sidebar",
  "sidebar.close": "Close sidebar",

  // Signup
  "signup.heading": "Apply",
  "signup.intro":
    "Apply for a hosted Plane + Mattermost trial environment. After admin review you'll receive credentials by email.",
  "signup.field.name": "Name",
  "signup.field.email": "Email",
  "signup.field.phone": "Phone",
  "signup.field.projectName": "Project name",
  "signup.field.teamSize": "Team size",
  "signup.field.techStack": "Tech stack",
  "signup.field.message": "Message",
  "signup.required.name": "Please enter your name.",
  "signup.required.email": "Please enter your email.",
  "signup.required.email.format": "That doesn't look like a valid email.",
  "signup.required.projectName": "Please enter a project name.",
  "signup.required.teamSize": "Please pick a team size.",
  "signup.optional": "(optional)",
  "signup.placeholder.email": "you@example.com",
  "signup.placeholder.phone": "+1 555-555-5555",
  "signup.placeholder.projectName": "my-cool-app",
  "signup.placeholder.message":
    "Anything else you'd like the admin to know.",
  "signup.teamSize.placeholder": "Pick a size…",
  "signup.teamSize.solo": "solo (1)",
  "signup.teamSize.small": "small (2–5)",
  "signup.teamSize.medium": "medium (6–10)",
  "signup.submit.idle": "Submit Request",
  "signup.submit.sending": "Sending…",
  "signup.submit.error.validation": "Please double-check the fields above.",
  "signup.submit.error.generic":
    "Something went wrong. Please try again in a moment.",
  "signup.submit.error.network": "Network error. Please try again.",
  "signup.banner":
    "ℹ Shared environment hosted at trial.realstory.blog. Available indefinitely once the admin approves your request.",

  // Status
  "status.heading": "Application status",
  "status.token": "Token:",
  "status.summary.name": "Name",
  "status.summary.email": "Email",
  "status.summary.phone": "Phone",
  "status.summary.project": "Project",
  "status.summary.teamSize": "Team size",
  "status.summary.stack": "Stack",
  "status.summary.message": "Message",
  "status.summary.submitted": "Submitted",
  "status.pending.title": "⏳ Pending — under admin review",
  "status.pending.body":
    "Once the admin provisions your Plane + Mattermost environment, credentials will appear on this page. Bookmark this URL.",
  "status.provisioned.title": "✅ Provisioned — credentials issued",
  "status.provisioned.body":
    "Keep these credentials safe. Sensitive values are masked by default; use the Show button to reveal them.",
  "status.provisioned.errorParse":
    "Could not read the credential payload. Please contact the admin.",
  "status.revoked.title": "🚫 Revoked — trial environment recalled",
  "status.revoked.body":
    "The trial environment for project {project} has been revoked by the admin. Reapply or contact the admin.",
  "status.notFound.title": "Application not found",
  "status.notFound.body":
    "The token may be wrong or expired. Please double-check the URL.",
  "status.notFound.cta": "Back to Apply",

  // Credentials view
  "creds.show": "Show",
  "creds.hide": "Hide",
  "creds.copy": "Copy",
  "creds.copied": "Copied",
  "creds.plane": "Plane",
  "creds.mattermost": "Mattermost",
  "creds.botTokens": "Mattermost Bot Tokens",

  // Lang switcher
  "lang.aria": "Language",
};

const DICTS: Record<Lang, Dict> = { ko: KO, en: EN };

export function t(
  lang: Lang,
  key: string,
  params?: Record<string, string | number>,
): string {
  const raw = DICTS[lang][key] ?? DICTS.ko[key] ?? key;
  if (!params) return raw;
  let out = raw;
  for (const [k, v] of Object.entries(params)) {
    out = out.replaceAll(`{${k}}`, String(v));
  }
  return out;
}

export function isLang(value: unknown): value is Lang {
  return value === "ko" || value === "en";
}
