# ADR-019 — 실 Plane + Mattermost 자동 provisioning (`genasis provision`)

> English: [docs/ADR/ADR-019-real-provisioning.md](../../ADR/ADR-019-real-provisioning.md)

상태: **제안 (alpha.26 scaffold 까지 ship; REST adapter 작업 중)**
일자: 2026-05-15

## 배경

Genasis 의 trial path (ADR-017) 는 가짜 Plane + Mattermost 환경을 몇 초만에
띄워서 "한번 봐볼게" 단계엔 좋지만 실제 협업은 안 됨. 팀이 진짜로 agent
와 협업하기로 결정한 순간엔 진짜 Plane 프로젝트 + 진짜 Mattermost 채널 +
agent 별 진짜 서비스 계정이 필요하고, 그것이 `genasis listen` 데몬과 함께
동작해야 한다.

이전 답은 별도의 Python `provision.py` 스크립트였다. trial 은 binary,
real 은 script — tooling 이 분열되고 Python + `requests` 설치가 추가로
필요했으며, 자격증명 다루는 부분이 Rust 의 타입 시스템 밖으로 빠진다.

ADR-019 는 그 전체 흐름을 기존 `genasis` 바이너리 안의 first-class
서브커맨드 두 개로 통합한다: 초기 셋업은 **`genasis provision`**,
이후 멤버 변동은 **`genasis team add | remove | list`**.

## 결정

### 1. 단일 binary `genasis provision`

```
genasis provision
  [--team "Marketing Squad"]
  [--app "Quiz Demo"]
  [--humans "Bravo Kim <gnoopy@gmail.com>,Alice <alice@x.com>"]
  [--humans-file ./humans.json]
  [--agents pm,frontend,backend,devops,designer,qa,planner,architect,code-reviewer,security]
  [--output ./]
  [--non-interactive]
  [--dry-run]
```

필수 환경 변수:

```
PLANE_URL          # https://plane.realstory.blog 또는 http://localhost:8080
PLANE_ADMIN_TOKEN  # Plane admin UI 에서 발급한 admin API token
MM_URL             # https://mm.realstory.blog 또는 http://localhost:8065
MM_ADMIN_TOKEN     # Mattermost system-admin PAT
```

실행 모드 2개:

- **Interactive (default — 플래그 누락 시)** — stdin 으로 team 이름,
  app 이름, 인간 멤버 (한 명씩 `name?`/`email?`) 를 차례로 묻고
  확인 화면을 보여준다. CI / 무인 설치는 `--non-interactive` 로
  prompt 를 건너뛰며 그 경우 필수 값 누락은 즉시 에러.

- **완전 스크립트** — 모든 input 을 플래그 또는 `--humans-file` 로.

### 2. Slug 약어 규칙

team / app 이름은 **5자 이내 소문자 slug** 로 약어화 — `genasis_core::slug::slugify_abbrev`:

| 입력                          | Slug    | 이유                          |
|------------------------------|---------|-------------------------------|
| `Marketing Squad`             | `ms`    | 2 단어 → 각 첫 글자           |
| `Marketing Communications`    | `mc`    | 2 단어 → 각 첫 글자           |
| `Quiz Demo`                   | `qd`    | 2 단어 → 각 첫 글자           |
| `Quiz`                        | `quiz`  | 1 단어 → 첫 5자               |
| `Pomodoro`                    | `pomod` | 1 단어 → 첫 5자               |
| `팀협업` (한글)                | `tc` (번역 → "team collab" → `tc`) — `claude` 없으면 deunicode 음역 `the` |

한글 input 은 먼저 사용자 머신의 `claude -p` CLI 에 "short English
phrase (1-3 words)" 로 번역 요청 후 약어화. `claude` 호출은 30초
timeout; 실패 시 `deunicode` 음역으로 fallback.

### 3. 식별자 패턴

| 리소스                         | 패턴                                | 예시                              |
|--------------------------------|-------------------------------------|-----------------------------------|
| Plane workspace (선호)         | `<team-slug>`                       | `ms`                              |
| Plane workspace (fallback)     | `agentic`                           | 공용 (per-team 권한 막힐 때)      |
| Plane project name             | `<app-name>` 또는 `<team>-<app>`    | `Quiz Demo`                       |
| Plane project identifier       | `<APP-SLUG>` (대문자)               | `QUIZ`                            |
| Mattermost team                | `team-<team-slug>`                  | `team-ms`                         |
| Mattermost scrum channel       | `scrum-<app-slug>`                  | `scrum-quiz`                      |
| Agent user (Plane + MM)        | `<role>-<team-slug>@genasis.bot`    | `pm-ms@genasis.bot`               |
| Human username (제안)          | 이메일 local-part                   | `gnoopy@gmail.com` → `gnoopy`     |

### 4. 출력 파일

`--output` (기본: cwd) 에 두 파일 작성:

**`genasis.toml`** — 식별자 + URL 만 (private repo 이면 commit 안전):

```toml
[provision]
provisioned_at = "2026-05-15T22:35:00+09:00"
team_slug = "ms"
app_slug = "quiz"

[plane]
flavor = "real"
url = "https://plane.realstory.blog"
workspace_slug = "ms"
project_id = "01HABC..."
project_identifier = "QUIZ"

[mattermost]
flavor = "real"
url = "https://mm.realstory.blog"
team_id = "z4t5..."
scrum_channel_id = "ch-..."
scrum_channel_name = "scrum-quiz"

[[humans]]
name = "Bravo Kim"
email = "gnoopy@gmail.com"
username = "gnoopy"
plane_user_id = "..."
mm_user_id = "..."

[[agents]]
role = "pm"
email = "pm-ms@genasis.bot"
plane_user_id = "..."
mm_user_id = "..."
```

**`.env.local`** — 자격증명. chmod 600, gitignore. JSON blob 안 쓰는
flat KEY=VALUE 형식 (사용자 명시 선호):

```env
PLANE_URL=https://plane.realstory.blog
PLANE_WORKSPACE_SLUG=ms
PLANE_PROJECT_ID=01HABC...

PLANE_AGENT_TOKEN_PM=plk_...
PLANE_AGENT_TOKEN_FRONTEND=plk_...
PLANE_AGENT_USERID_PM=...
PLANE_AGENT_USERID_FRONTEND=...
# ... agent 별 pair, humans 도 같은 패턴

MM_URL=https://mm.realstory.blog
MM_TEAM_ID=z4t5...
MM_SCRUM_CHANNEL_ID=ch-...

MM_AGENT_PAT_PM=mm_pat_...
MM_AGENT_USERID_PM=...

HUMAN_PLANE_USERID_GNOOPY=...
HUMAN_MM_USERID_GNOOPY=...
```

데몬의 D-098 env passthrough 가 이미 이 환경 변수들을 MCP server +
orchestrator claude 에 전달한다.

### 5. Plane workspace 전략

2-path:

1. **신청자별 workspace** (선호). admin token 이 권한 있으면
   `POST /api/v1/workspaces/` 로 team slug 워크스페이스 생성.
   같은 Plane 인스턴스의 다른 Genasis 팀들과 완전 격리됨.

2. **공용 `agentic` workspace + naming 컨벤션** (fallback). workspace
   생성 권한이 막혀 403 이면 미리 만들어진 `agentic` 워크스페이스 안에
   프로젝트 이름을 `<team>-<app>` (예: `ms-quiz`) 로 둔다. 어느 경로
   타졌는지 최종 요약에서 사용자에게 알림.

### 6. Post-provision: `genasis team`

`provision` 은 1회용. 운영 중 변경:

```
genasis team add human "Charlie <charlie@x.com>"   # 인간 추가
genasis team add agent designer                     # agent 추가
genasis team add agent custom-role                  # 새 role 추가
genasis team remove human alice@x.com               # 인간 deactivate
genasis team remove agent designer                  # agent 은퇴
genasis team list                                   # 현재 roster + health
```

모두 idempotent: 이미 있는 멤버 재추가는 informational message 와 함께
no-op, 없는 멤버 제거도 no-op. Plane / MM 은 deactivated 계정의 history
(작성한 issue, post) 를 보존한다.

### 7. 실패 처리

자동 rollback 없음. 모든 REST 호출은 GET-before-create (멱등) 또는
target state 로 수렴하는 PATCH 로 구조화. 부분 실패 시 사용자가 같은
명령 재실행 → 이미 생성된 리소스 감지하고 skip, 다음 step 부터 이어서
진행.

## 결과

- **단일 Rust binary 가 팀 lifecycle 전체 소유**: trial, real
  provisioning, 멤버 변경. Python / shell 추가 설치 없음. 운영자 호스팅
  vs 자체 호스트 경로 사이 언어 분열 없음.

- **`.env.local` 이 자격증명 경계**. `genasis.toml` 안의 모든 것
  (식별자, URL, role 이름) 은 공유 가능. `.env.local` 안의 모든 것은
  사용자 머신을 벗어나면 안 되는 per-agent secret. 데몬이 startup 시
  두 파일 모두 load 하고 필요한 env 만 forward.

- **Slug 충돌은 존재하지만 제한적**. "Marketing Squad" 두 팀이 다
  `ms` 로 slug 됨 — per-team workspace path 에서 한쪽이 409 로 막힘,
  fallback path 에서도 두 번째 project 가 409 로 막힘. 사용자에게 다른
  team 이름 선택 요청. auto-disambiguation (`ms2`, `ms3`) 안 함 — 그러면
  식별자가 의미 없어짐.

- **한글 번역은 사용자의 `claude` CLI 의존**. `genasis listen` 과 동일한
  의존성이라 새 설치 요건 추가 아님. `claude` 가 없으면 transliteration
  fallback 이 uniqueness 는 보장하지만 의미는 손실 — 한글 team 이름의
  slug 가 사람이 읽을 수 있길 원하는 사용자는 `provision` 전에 `claude`
  설치 확인 권장.

- **trial 경로는 그대로**. `genasis init --trial` / `genasis listen
  --trial` 은 `mmplane-trial.realstory.blog` 대상으로 변동 없이 작동.
  ADR-017 과 ADR-019 는 보완 관계, 경쟁 관계 아님.

## 구현 상태 (alpha.26)

- `genasis-core::slug` — landed (unit test 8개).
- `genasis-cli::cmd_provision` — clap surface, plan resolution,
  interactive prompts, dry-run preview landed (unit test 6개).
- `genasis-cli::cmd_team` — clap surface scaffolded; 본문 구현은
  명확한 "not yet implemented" 에러 반환 + 다음 alpha 가리킴.
- **Pending**: `genasis-providers::plane::real_provisioner` (REST),
  `genasis-providers::mattermost::real_provisioner` (REST), output
  파일 writers, `genasis team` 본문 구현.
