> English: [`../../ADR/ADR-016-trial-app-name-alignment-and-multitenancy.md`](../../ADR/ADR-016-trial-app-name-alignment-and-multitenancy.md)

# ADR-016: trial-app 식별자 정렬 + 멀티테넌트 샌드박스

## Status

Proposed (2026-05-11). ADR-013에서 명시되지 않았던 공백을 메우는
결정 — ADR-013은 genasis ↔ trial-app 사이의 *라우팅*을 정의했지만,
그 라우트로 흘러가는 *식별자*(팀명, 채널명, 프로젝트명)는 다루지
않았다.

## Context

현재 `genasis init --trial`은 사용자가 어떤 팀을 만들고 싶어하든
`genasis.toml`에 Plane/Mattermost 식별자를 리터럴 `"trial"` /
`"trial-demo"`로 박아 쓴다:

```toml
[project]    name = "trial-demo"
[plane]      workspace_slug = "trial"
[mattermost] team_name = "trial"
```

trial-app 시뮬레이터(`agents-pool/trial-app/db/sim.ts`)는 reactive하다
— `ensureProject(slug)`은 처음 보는 slug가 들어오면 lazy 생성한다.
그러나 **팀별 격리가 없다.** 모든 `genasis init --trial` 사용자가
`https://mmplane-trial.realstory.blog`의 한 SQLite 네임스페이스를
공유하므로, 동시 데모가 서로의 칸반 카드와 채팅 스레드를 덮어쓴다.

대칭으로, 실모드 템플릿(`agents/genasis.toml.tera`)은
`[mattermost].team_name`과 `[plane].workspace_slug`은 선언하지만
**채널명이나 사람이 읽을 프로젝트명은 선언하지 않는다.** ADR-014의
"#scrum-{project_name}" 관습은 에이전트 프롬프트 안의 보일러플레이트로만
존재할 뿐, CLI나 trial-app이 읽을 수 있는 데이터로는 없다.

결과 — 통합 갭 세 가지:

1. **식별자 소실** — 사용자의 팀/프로젝트명이 `genasis init`과
   에이전트가 보는 값 사이에서 사라진다. 트라이얼 모드에서는
   "trial"로 보이고, 실모드에서는 채널 리스트 인지 없이 단일
   필드로부터 렌더된 "#scrum-{project_name}"으로 보인다.
2. **bootstrap 부재** — `genasis init`이 trial-app에 "이 팀이 존재하고
   초기 채널 구성은 이렇다"라고 알릴 방법이 없다.
3. **테넌시 부재** — (2)가 있어도, trial-app은 한 사용자의 `marketing`
   프로젝트와 다른 사용자의 그것을 구분할 수 없다.

세 갭이 모두 닫히기 전까지, trial 모드 워크플로우는 North Star
(CLAUDE.md §Core Philosophy §3)를 위반한다 — `genasis init --trial`을
실행한 새 팀은 데모에서 *자기 팀*을 보지 못하고 리터럴 "trial"이
붙은 공유 스크래치패드만 본다.

## Decision

세 가지 변경을 함께 출하한다 — 각각이 나머지 둘 없이는 무용하다.

### 1. 실모드 schema에 명시적 채널·프로젝트명 필드 추가

`crates/genasis-core/src/config.rs`의 `PlaneConfig`/`MattermostConfig`
확장:

```rust
pub struct PlaneConfig {
    pub url: String,
    pub flavor: String,
    pub workspace_slug: String,
    pub project_id: Option<String>,
    /// Plane UI와 trial-app 칸반 헤더에 표시되는 사람-읽기용 프로젝트명.
    /// 비어 있으면 `project.name`으로 대체된다.
    pub project_name: Option<String>,
}

pub struct MattermostConfig {
    pub url: String,
    pub flavor: String,
    pub team_name: String,
    /// 이 팀에 프로비저닝되는 채널들. `key = "scrum"`인 첫 번째
    /// 채널이 ADR-014의 binding-stakeholder 채널.
    pub channels: Vec<MattermostChannel>,
}

pub struct MattermostChannel {
    /// 에이전트와 템플릿에서 쓰는 안정적 키("scrum", "design",
    /// "design-incoming"). 팀 안에서 유일해야 한다.
    pub key: String,
    /// Mattermost 채널 슬러그. 기본값 "{key}-{project_slug}".
    pub name: String,
    /// Mattermost UI와 trial-app 사이드바에 표시되는 사람-읽기용 이름.
    pub display_name: String,
}
```

두 신규 필드는 `#[serde(default)]`이므로 기존 `genasis.toml`은 그대로
로드된다. 누락된 값은 `Config::derive_defaults()`가 채운다 —
`project_name = project.name`, `team_name + project.name`으로부터
단일 `scrum` 채널을 합성.

TOML 표현은 inline-table 배열을 쓴다 — serde-toml round-trip이
섹션 순서에 흔들리지 않게:

```toml
[mattermost]
url = "https://mm.example.com"
team_name = "marketing"
flavor = "auto"
channels = [
  { key = "scrum",          name = "scrum-marketing-squad",          display_name = "Marketing Squad — Scrum" },
  { key = "design",         name = "design-marketing-squad",         display_name = "Marketing Squad — Design" },
  { key = "design-incoming", name = "design-incoming-marketing-squad", display_name = "Marketing Squad — Design Incoming" },
]
```

### 2. `[trial].team_token` — 멀티테넌트 키

`TrialConfig`에 추가:

```rust
pub struct TrialConfig {
    pub enabled: bool,
    pub url: String,
    pub shared_secret: String,
    /// 팀별 격리 키. `genasis init --trial`이 16바이트 hex로
    /// 무작위 생성. trial-app은 모든 sim_* 행을 이 토큰으로
    /// 스코핑하므로, 호스팅 인스턴스의 동시 사용자가 서로의
    /// 샌드박스를 덮어쓰지 못한다.
    pub team_token: Option<String>,
}
```

빈 값/누락이면 trial-app 쪽에서 리터럴 `"default"`로 해석된다 —
이 ADR 이전의 config는 단일테넌트 동작 그대로.

`genasis init --trial`은 `getrandom`으로 신선한 토큰(16바이트 hex)을
생성, 사용자가 고른 프로젝트명·채널 리스트와 함께 config에 적는다.

### 3. trial-app sim의 팀 스코프화 + bootstrap 라우트

trial-app sim schema는 `user_version = 1`에서 `2`로 마이그레이션:

```sql
-- sim_projects, sim_channels, sim_issues, sim_posts 각각에:
team_token TEXT NOT NULL DEFAULT 'default'

-- 그리고 단일 컬럼 UNIQUE(slug/name)는 다음으로 교체:
UNIQUE(team_token, slug)   -- sim_projects
UNIQUE(team_token, name)   -- sim_channels

-- 신규 테이블
CREATE TABLE sim_teams (
  token        TEXT PRIMARY KEY,
  project_slug TEXT NOT NULL,
  project_name TEXT NOT NULL,
  created_at   TEXT NOT NULL DEFAULT (datetime('now'))
);
```

마이그레이션은 `PRAGMA user_version`으로 게이팅된 작은
`migrate(db)` 함수가 수행한다. 기존 행은 `team_token = 'default'`로
재할당되어, 토큰 헤더를 보내지 않는 클라이언트(=레거시 호스팅 데모)는
계속 동작한다.

신규 라우트 — `POST /api/trial/bootstrap` — 입력:

```json
{
  "team_token": "a3f9b2c1e8d4...",
  "project":  { "slug": "marketing-squad", "name": "Marketing Squad" },
  "channels": [
    { "key": "scrum", "name": "scrum-marketing-squad", "display_name": "Marketing Squad — Scrum" }
  ]
}
```

…그 토큰 아래서 `ensureTeam`, `ensureProject`, `ensureChannel`을
멱등적으로 호출. 같은 payload로 재실행 = no-op. 충돌 payload는
display name만 갱신, 기존 issue/post는 절대 삭제하지 않음.

모든 Plane / Mattermost 브리지 라우트는 활성 팀을 다음 순서로
해석한다 — `X-Genasis-Team-Token` 헤더 → `?team=` 쿼리 → fallback
`"default"`. fallback이 있는 이유: 토큰 없이 익명 브라우저 탭으로
열린 레거시 "click and play" 데모도 계속 동작해야 함.

## Consequences

**쉬워지는 것**:
- `genasis init --trial --name "Marketing Squad"`이 실제로 칸반에
  "Marketing Squad"로 표시되고, 채팅에 "#scrum-marketing-squad"가
  뜨는 샌드박스를 만든다 — 에이전트가 부르는 이름과 일치.
- 실모드(non-trial) 프로젝트가 구조화된 채널 리스트를 갖게 됨 —
  에이전트가 `project.name`을 string-format해서 이름을 유도하는
  대신 config에서 읽는다. "rename storm 없이 채널명 바꾸기"가
  config 편집 한 줄로 끝난다.
- 여러 개발자가 trial 앱을 동시에 데모해도 서로 충돌하지 않는다.

**어려워지는 것**:
- trial-app schema 마이그레이션(V1 → V2)은 일회성 테이블 재구축이
  필요. 운영자 배포 키트의 `start.sh`가 시작 시 실행하며, 레거시
  `default` 네임스페이스를 삭제하지 *않는다* — 보이는 데이터 손실
  없음. 클린 슬레이트를 원하면 `rm data/trial.db`로 기존과 동일.
- `MattermostChannel` 추가는 `MattermostConfig`를 `deny_unknown_fields`로
  역직렬화하는 외부 코드에 한해 깨질 수 있는 형변경. 내부 호출자는
  그렇게 하지 않음 — 기존 config round-trip 테스트로 검증.

**못 하게 되는 것**:
- 채널 정의가 두 곳에 살 수 없다. "#scrum-{project_name}"을
  하드코딩한 템플릿은 `MattermostConfig.channels`에서
  `key = "scrum"`으로 lookup하도록 다시 쓴다. 후속 영향 —
  `agents/env.agents.tera`는 이제 별도 부트스트랩 단계에 의존한
  암묵적 `MM_SCRUM_CHANNEL_ID` 대신 `MM_SCRUM_CHANNEL_KEY`를
  emit한다.

## Verification

- Unit 테스트:
  - `crates/genasis-core/src/config.rs`: `derive_defaults_*` 시리즈 —
    project_name 누락, channels 누락, 단일 채널이 `"scrum"` 키로
    fallback 되는지 커버.
  - `crates/genasis-cli/src/cmd_init.rs`: `--probe-only` 모드에서
    `run_trial`이 기대한 토큰·채널 리스트를 쓰는지.
- Trial-app:
  - SQL 마이그레이션 테스트: V1 fixture DB에서 시작해 `migrate()` 후
    모든 레거시 행이 `team_token = 'default'`로 옮겨졌는지.
  - Bootstrap 라우트: 두 번째 호출에서 멱등적, 두 토큰을 순차로
    쓸 때 스코프 정상.
- E2E (`#[ignore]`로 라이브 trial-app 게이팅):
  `crates/genasis-providers/tests/trial_factory_e2e.rs`에 team-token
  스코핑 케이스 추가.

## References

- ADR-013 (trial-bridge config wiring) — 이 ADR은 그 라우팅 결정 위에
  식별자 레이어를 얹는다.
- ADR-014 (human roster provisioning) — 이 ADR이 구조화된 채널
  데이터로 형식화하는 "#scrum-{project_name}" 관습.
- ADR-015 (shared PostgreSQL + multi-tenant) — 실-Plane/실-Mattermost
  쪽의 유사한 테넌시 스토리. 이 ADR은 trial-app sim에 그것을 거울처럼
  재현한다.
- 구현:
  `crates/genasis-core/src/config.rs`,
  `crates/genasis-cli/src/cmd_init.rs`,
  `agents/genasis.toml.tera`,
  `agents-pool/trial-app/db/{index,sim}.ts`,
  `agents-pool/trial-app/app/api/trial/bootstrap/route.ts`,
  `agents-pool/trial-app/lib/trial-auth.ts`.
