# genasis-team-plan.md — `genasis provision` / `genasis team` 다음 구현

> 임시 작업 계획. 본 구현 단계 완료되면 [ADR-019](docs/ADR/ADR-019-real-provisioning.md)
> 의 "Implementation status" 섹션 업데이트하고 이 파일 삭제 또는 archive.

상태: **alpha.27 까지 scaffold + ADR + README 완료**. 이 문서는 다음
사이클의 REST 어댑터 + `cmd_team` 본문 + writer 구현 계획.

## 0.5 사용자 결정 반영 (alpha.27 이후 추가)

§9 검증 후 사용자 결정:

- **두 케이스 분기**:
  - **자체 운영팀**: 개발팀이 plane/mattermost 직접 운영. 그들이 admin
    token + 기존 agent user 들 + 사람 계정 직접 관리. `genasis provision`
    은 admin token + URL 받아 그 위에 동작.
  - **운영자 호스팅 (`*.realstory.blog`)**: 운영자(= 우리)가 agent id
    발급 책임. 10개 default + post-provision 추가 agent.

- **Plane multi-tenancy 모델 (운영자 호스팅)**:
  - workspace = **`agentic` 하나만 사용**. per-team workspace 생성 시도
    삭제 (§2.2 의 path 1 제거).
  - agent users = workspace member 로 등록 → **자동으로 모든 project
    가시**. project 격리 검증은 하지 않고 진행.
  - 사람 = **각자 자기 project 에만 invite**. workspace 전역 member
    안 함. cross-team leakage 차단을 사람 단위에서만 적용.

- **Mattermost multi-tenancy 모델 (운영자 호스팅)**:
  - team = **신청자별 1개** (`team-<team-slug>`).
  - agent users = 모든 team 의 member.
  - 사람 = **자기 team 에만 invite**. team 단위 격리.
  - scrum channel = 각 team 안에 1개 (`scrum-<app-slug>`).

- **Daemon 측 격리 보장 (반드시 지켜져야 함)**:
  - agent (claude) 는 자기 팀의 env 안의 `PLANE_PROJECT_ID` /
    `MM_TEAM_ID` / `MM_SCRUM_CHANNEL_ID` 만 사용. MCP server 의 wrapper
    가 다른 project/channel 로 호출 시도하면 reject.
  - daemon 의 D-098 env passthrough 가 팀별로 분리된 env set 을 각
    orchestrator claude 에 전달 → 같은 PAT 이라도 호출하는 project_id
    는 자기 팀 것 뿐.
  - MCP tool input validation 에서 project_id / channel_id 가 env 와
    일치 안 하면 fail-fast. 이건 §11 에 추가.

## 1. 모듈 구조 — 변경됨

## 0. 시작 전 확인 사항

### 필요 자격증명
- `secusy/.env.local` 의 운영자 admin token (사용자가 이전 turn 에 제공)
- `PLANE_URL=https://plane.realstory.blog` + `PLANE_ADMIN_TOKEN=...`
- `MM_URL=https://mm.realstory.blog` + `MM_ADMIN_TOKEN=...`

### 사전 검증 명령
```bash
# Plane admin token 살아있나
curl -H "X-API-Key: $PLANE_ADMIN_TOKEN" "$PLANE_URL/api/v1/workspaces/"

# Mattermost admin token 살아있나
curl -H "Authorization: Bearer $MM_ADMIN_TOKEN" "$MM_URL/api/v4/users/me"
```

두 호출 모두 200 으로 응답하지 않으면 본 구현 시작 전에 사용자에게 새
admin token 받아야 함.

## 1. 모듈 구조

```
crates/
├── genasis-providers/src/
│   ├── plane/
│   │   ├── mod.rs                       ← 기존 (수정: pub mod real_provisioner)
│   │   └── real_provisioner.rs          ← NEW
│   ├── mattermost/
│   │   ├── mod.rs                       ← 기존 (수정: pub mod real_provisioner)
│   │   └── real_provisioner.rs          ← NEW
│   └── lib.rs                           ← 변경 없음
└── genasis-cli/src/
    ├── cmd_provision.rs                 ← 수정: run() 의 bail! 제거 → live flow
    ├── cmd_team.rs                      ← 수정: bail! 제거 → 본문 구현
    └── provision_writer.rs              ← NEW: genasis.toml + .env.local writer
```

## 2. Plane REST adapter (`plane/real_provisioner.rs`)

### 2.1 공통 client
```rust
pub struct PlaneAdmin {
    pub url: String,        // strip trailing /
    pub token: String,      // X-API-Key header
    client: reqwest::Client,
}
```

Header pattern (Plane v0.x community edition): `X-API-Key: <token>`.
실제 API 호출 직접 시도해서 응답 보고 정확한 header 확인 필요.

### 2.2 Workspace 두 path 처리

```rust
pub enum WorkspaceMode {
    PerTeam { slug: String, name: String },
    Shared { slug: String },   // "agentic"
}

pub async fn provision_workspace(
    admin: &PlaneAdmin,
    team_slug: &str,
    team_name: &str,
) -> Result<WorkspaceMode> {
    // 1차: per-team workspace 생성 시도
    let resp = admin.client.post(format!("{}/api/v1/workspaces/", admin.url))
        .header("X-API-Key", &admin.token)
        .json(&json!({ "slug": team_slug, "name": team_name }))
        .send().await?;
    
    match resp.status() {
        // 201 created OR 400/409 with "slug already taken" + GET 으로
        // 우리 admin 이 그 workspace 의 owner 인지 검증
        StatusCode::CREATED => Ok(WorkspaceMode::PerTeam {...}),
        StatusCode::FORBIDDEN => {
            // admin token 이 workspace 생성 권한 없음 → fallback
            // shared "agentic" workspace 존재 확인 (없으면 admin 이 미리
            // 만들어둬야 함)
            ensure_shared_workspace_exists(admin).await?;
            Ok(WorkspaceMode::Shared { slug: "agentic".into() })
        }
        StatusCode::CONFLICT => {
            // 이미 있음 — owner 확인 후 그대로 사용
            verify_owner_and_reuse(admin, team_slug).await?;
            Ok(WorkspaceMode::PerTeam {...})
        }
        s => bail!("unexpected response {s} from POST /workspaces"),
    }
}
```

### 2.3 Project

`POST /api/v1/workspaces/<slug>/projects/`
```json
{ "name": "Quiz Demo", "identifier": "QUIZ", "description": "..." }
```

응답에서 `id` 받아 저장.

Fallback workspace 일 때는 name 을 `<team>-<app>` 으로 (예: `ms-quiz`).

### 2.4 Agent users (10명)

Plane 의 user 생성 endpoint 가 self-hosted 에서 어떻게 노출되는지 확인
필요. 가능성:
- **A**: `POST /api/v1/users/` (admin only, password 와 함께)
- **B**: `POST /api/v1/workspaces/<slug>/invitations/` (email invite,
  user 가 link 클릭해야 활성화 — bot 으로는 부적합)
- **C**: Plane 의 instance admin API 가 별도 존재 (`/api/v1/instances/admin/users/`)

자체 호스트 + admin token 으로 user record 직접 생성하는 path 가 best.
실제 운영자 인스턴스에서 admin token 으로 어떤 endpoint 가 200 하는지
**구현 전에 curl 로 검증**해야 함.

agent 별:
```
email:    <role>-<team-slug>@genasis.bot
password: 랜덤 32-char (저장 안 함, agent 는 PAT 만 사용)
name:     "<Role> agent (<team-name>)"
```

### 2.5 PAT 발급

`POST /api/v1/workspaces/<slug>/api-tokens/` 또는 사용자별
`POST /api/v1/users/<id>/api-tokens/`. Plane 문서 확인 필요.
응답의 token 값을 `PLANE_AGENT_TOKEN_<ROLE_UPPER>` 환경변수로 저장.

### 2.6 Membership

`POST /api/v1/workspaces/<slug>/members/` + `POST
/api/v1/workspaces/<slug>/projects/<id>/members/`.

Role mapping:
- 인간: `Admin` (id 20)
- PM agent: `Admin`
- 그 외 agent: `Member` (id 15)

### 2.7 Human invite

`POST /api/v1/workspaces/<slug>/invitations/` with `email` +
`role`. 사용자가 받은 invitation link 를 클릭해 가입 — 이 부분은
사용자 동작 필요하므로 prouvision 후 안내 메시지 출력.

### 2.8 멱등성 helpers

```rust
async fn workspace_exists(admin: &PlaneAdmin, slug: &str) -> Result<bool>;
async fn project_exists(admin: &PlaneAdmin, ws_slug: &str, identifier: &str) -> Result<Option<ProjectRef>>;
async fn user_by_email(admin: &PlaneAdmin, email: &str) -> Result<Option<UserRef>>;
async fn ensure_member(admin: &PlaneAdmin, ws_slug: &str, user_id: &str, role: i32) -> Result<()>;
```

모든 create 전에 GET 으로 존재 확인.

## 3. Mattermost REST adapter (`mattermost/real_provisioner.rs`)

### 3.1 공통 client
```rust
pub struct MmAdmin {
    pub url: String,
    pub token: String,        // Authorization: Bearer <token>
    client: reqwest::Client,
}
```

Header: `Authorization: Bearer <admin_pat>`.

### 3.2 Team

`POST /api/v4/teams`
```json
{ "name": "team-ms", "display_name": "Marketing Squad", "type": "O" }
```

응답에서 `id` 저장.

멱등성: `GET /api/v4/teams/name/team-ms` 가 200 이면 그대로 사용.

### 3.3 Scrum channel

`POST /api/v4/channels`
```json
{ "team_id": "...", "name": "scrum-quiz", "display_name": "Scrum — Quiz Demo", "type": "O" }
```

### 3.4 Agent users + PAT

agent 별:
1. `POST /api/v4/users` with email/password (admin 권한 필요)
2. `POST /api/v4/users/<id>/tokens` for PAT (admin only)
3. `POST /api/v4/teams/<team_id>/members` (add to team)
4. `POST /api/v4/channels/<channel_id>/members` (add to channel)

PAT response 의 `token` 을 `MM_AGENT_PAT_<ROLE_UPPER>` 환경변수로 저장.

### 3.5 Human invite

두 path:
- 이미 가입된 인간: `GET /api/v4/users/email/<email>` → 있으면 team
  추가
- 신규 인간: `POST /api/v4/teams/<team_id>/invite/email` → 이메일
  invitation 발송 (사용자가 클릭해서 가입)

### 3.6 멱등성

`GET /api/v4/users/email/<email>` 이 200 이면 user 그대로, 404 면 생성.
team / channel 도 동일 패턴.

## 4. genasis.toml + .env.local writer (`provision_writer.rs`)

### 4.1 genasis.toml 빌더

```rust
pub fn write_genasis_toml(
    dir: &Path,
    plan: &ResolvedProvisionPlan,
    plane_result: &PlaneProvisionResult,
    mm_result: &MmProvisionResult,
) -> Result<()> {
    // toml crate 사용. 이미 dependency 있음.
    let cfg = toml::toml! {
        [provision]
        provisioned_at = "..."  // chrono::Local::now()
        team_slug = "..."
        app_slug = "..."
        
        [plane]
        flavor = "real"
        url = "..."
        workspace_slug = "..."
        project_id = "..."
        project_identifier = "..."
        
        [mattermost]
        flavor = "real"
        url = "..."
        team_id = "..."
        scrum_channel_id = "..."
        scrum_channel_name = "..."
        
        // [[humans]] 와 [[agents]] arrays
    };
    
    let path = dir.join("genasis.toml");
    // 기존 파일 있으면 merge (provision 만 update, project 같은 다른
    // 섹션은 보존). 또는 backup 생성 후 overwrite.
    atomic_write(&path, &toml::to_string(&cfg)?)?;
    Ok(())
}
```

### 4.2 .env.local writer

```rust
pub fn write_env_local(
    dir: &Path,
    plane_result: &PlaneProvisionResult,
    mm_result: &MmProvisionResult,
) -> Result<()> {
    let path = dir.join(".env.local");
    let mut buf = String::new();
    
    // 헤더
    buf.push_str("# Generated by `genasis provision`. DO NOT COMMIT.\n");
    buf.push_str("# Per-agent API tokens — leak = full Plane/MM access for that role.\n\n");
    
    // Plane
    buf.push_str(&format!("PLANE_URL={}\n", plane_result.url));
    buf.push_str(&format!("PLANE_WORKSPACE_SLUG={}\n", plane_result.workspace_slug));
    buf.push_str(&format!("PLANE_PROJECT_ID={}\n\n", plane_result.project_id));
    
    for agent in &plane_result.agents {
        let role_upper = agent.role.replace('-', "_").to_uppercase();
        buf.push_str(&format!("PLANE_AGENT_TOKEN_{}={}\n", role_upper, agent.pat));
        buf.push_str(&format!("PLANE_AGENT_USERID_{}={}\n", role_upper, agent.user_id));
    }
    
    // Mattermost
    // ... 동일 패턴
    
    // 인간들
    for human in &plane_result.humans {
        let user_upper = derive_username(&human.email).to_uppercase();
        buf.push_str(&format!("HUMAN_PLANE_USERID_{}={}\n", user_upper, human.user_id));
    }
    
    // 쓰기 + chmod 600
    std::fs::write(&path, buf)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}
```

### 4.3 .gitignore 보정

`.env.local` + `.env` 패턴이 `.gitignore` 에 있는지 확인. 없으면
append. 이미 있으면 no-op.

## 5. cmd_provision live flow

```rust
pub async fn run(args: Args) -> Result<()> {
    let plan = resolve_plan(&args)?;
    print_plan(&plan, args.dry_run);
    
    if args.dry_run {
        return Ok(());
    }
    
    // 사용자 확인 (interactive 이고 --yes 아닐 때)
    if !args.non_interactive && !args.assume_yes {
        let confirmed = prompt_confirm("Proceed with live provisioning? [y/N]")?;
        if !confirmed { bail!("aborted by user"); }
    }
    
    let plane_admin = PlaneAdmin::new(&plan.plane.url, &plan.plane.admin_token);
    let mm_admin = MmAdmin::new(&plan.mattermost.url, &plan.mattermost.admin_token);
    
    // 1. Workspace + project
    println!("→ Plane: ensuring workspace…");
    let ws_mode = provision_workspace(&plane_admin, &plan.team_slug, &plan.team_name).await?;
    println!("  workspace = {} ({})", ws_mode.slug(), ws_mode.label());
    
    println!("→ Plane: ensuring project…");
    let project = provision_project(&plane_admin, &ws_mode, &plan).await?;
    
    // 2. Agents
    let mut plane_agents = Vec::new();
    for role in &plan.agents {
        println!("→ Plane: provisioning agent {role}…");
        let agent = provision_agent(&plane_admin, &ws_mode, &project, role, &plan.team_slug).await?;
        plane_agents.push(agent);
    }
    
    // 3. Humans
    let mut plane_humans = Vec::new();
    for h in &plan.humans {
        println!("→ Plane: inviting human {} <{}>…", h.name, h.email);
        let human = invite_human(&plane_admin, &ws_mode, &project, h).await?;
        plane_humans.push(human);
    }
    
    // 4~6. 같은 패턴으로 Mattermost
    ...
    
    // 7. 출력 파일 write
    write_genasis_toml(...)?;
    write_env_local(...)?;
    ensure_gitignore(...)?;
    
    print_summary(...);   // "✓ provisioned 10 agents, 2 humans. Next: `cd ... && genasis listen --real`"
    Ok(())
}
```

## 6. `cmd_team` 본문 구현

각 명령은 다음 순서:
1. `genasis.toml` load → 현재 roster 파악
2. `.env.local` load → 기존 토큰 보존
3. 해당 REST adapter 호출 (idempotent)
4. genasis.toml + .env.local diff-merge 후 atomic write

### 6.1 `team add human "Name <email>"`
- parse_human_spec (이미 있음)
- 이미 roster 에 있으면 informational message + no-op
- Plane invite → Mattermost team add → toml/env append

### 6.2 `team add agent <role>`
- 이미 roster 에 있으면 no-op
- Plane user + PAT → MM user + PAT → toml/env append

### 6.3 `team remove human <email>`
- Plane deactivate (`POST /api/v1/users/<id>/deactivate`) — history 보존
- MM deactivate (`DELETE /api/v4/users/<id>`)
- toml/env 제거

### 6.4 `team remove agent <role>`
- 같은 패턴

### 6.5 `team list`
- toml load + 각 user 의 health (last login, active flag) 표시

## 7. 테스트 전략

### 7.1 Unit tests (mock)
- 모든 GET-before-create 분기 (already-exists, not-found)
- 모든 error path (401, 403, 409, 500)
- 출력 파일 형식 (toml + env)

### 7.2 Integration tests (운영자 instance)
- 실제 `plane.realstory.blog` + `mm.realstory.blog` 에 testing-only
  team slug (`test-N` 같이) 으로 호출
- 멱등성 검증: 같은 명령 두 번 실행해도 동일 결과
- rollback 없음 검증: 부분 실패 후 재실행 시 이어서 진행

### 7.3 Self-host integration
- `secusy/server/docker-compose.yml` 띄운 뒤 `PLANE_URL=http://localhost:8080`
  로 동일 명령 검증

## 8. 구현 순서 (작은 PR 들로 쪼개기)

1. **PR-1**: Plane REST adapter (`plane/real_provisioner.rs`) + unit
   tests (mock). cmd_provision 은 아직 안 부름.
2. **PR-2**: Mattermost REST adapter 동일 패턴.
3. **PR-3**: `provision_writer.rs` (genasis.toml + .env.local) +
   tests with `tempfile`.
4. **PR-4**: `cmd_provision` 의 bail! 제거 → live flow. **운영자
   인스턴스에서 실제 호출** 한 번 성공시켜 멱등성 + identifier
   convention 검증.
5. **PR-5**: `cmd_team add human`.
6. **PR-6**: `cmd_team add agent`.
7. **PR-7**: `cmd_team remove human` + `remove agent`.
8. **PR-8**: `cmd_team list`.
9. **PR-9**: Self-host (docker-compose) 통합 검증 + README 의
   self-host 섹션 보강.

각 PR 은 alpha 한 cycle 으로 ship 가능한 크기.

## 9. 리스크 + 사용자 결정 필요 사항

### 9.1 Plane workspace 생성 권한 — 해결됨 (§0.5)
- 검증 결과: workspace API key 는 workspace-scoped, 새 workspace
  생성 불가. → **항상 `agentic` 사용**.

### 9.2 Plane agent user 생성 — 해결됨 (§0.5)
- 사용자 결정: agent user 는 한 번만 글로벌 등록 (이미 있음 — gnoopy
  계정 owner 가 admin UI 에서 한 번 등록한 결과). `genasis provision`
  은 기존 agent user 들을 reuse 하고 새 project 의 member 로 add 만
  수행. workspace member 면 모든 project 자동 가시 (격리 검증 skip).
- **post-provision agent 추가 시**: workspace 에 없는 새 role 이면
  운영자에게 안내 (admin UI 에서 user 만들고 PAT 발급해서 우리에게
  알려달라). 즉시 자동화 안 됨 — follow-up.

### 9.3 Mattermost user PAT 발급 권한
- system-admin token 으로 다른 user 의 PAT 발급 가능한지 (`POST
  /api/v4/users/<id>/tokens`) 인스턴스의 `EnableUserAccessTokens`
  설정 + admin-only flag 에 의존
- 운영자 인스턴스 config 검증을 PR-2 첫 단계로

### 9.4 멱등성 vs 보안
- agent password 는 PAT 만 사용하니 저장 안 하는 게 맞음
- 그러나 재실행 시 같은 agent 의 PAT 새로 발급해야 하면 충돌
- 해결: 이미 PAT 있으면 (env 에 토큰 있으면) revoke 안 하고 그대로
  사용. 없을 때만 새로 발급.

## 11. Daemon-side cross-team leakage 차단

§0.5 의 사용자 강조 — agent 가 다른 팀 project/channel 로 절대 쓰지
못하게. agent PAT 는 모든 project 의 member 라 cross-project API
호출이 *기술적으로* 가능하지만, daemon 코드가 강제로 자기 팀 ID
만 사용하도록 wrapping.

### 11.1 MCP server wrapper validation
- `mcp-plane` / `mcp-mattermost` 의 모든 tool 호출 input 의
  `project_id` / `channel_id` 가 env (`GENASIS_PLANE_PROJECT_ID` 등)
  와 일치하는지 검증. 다르면 fail-fast 로 reject + 명시적 error.
- trial-app 의 MCP 가 이미 env 의 `GENASIS_TEAM_TOKEN` 으로 자동
  주입하는 패턴 그대로.

### 11.2 daemon 의 env-set 분리
- `genasis listen --real` 은 한 팀 sandbox 1개에 대응. 다른 팀의
  env 는 daemon process 안에 절대 존재 안 함 (process 단위 격리).
- 운영자가 여러 팀의 daemon 을 한 머신에 띄울 때 process 별로 env
  격리. D-098 의 env passthrough 가 이미 보장.

### 11.3 PAT 공유 모델의 명시
- agent PAT 는 글로벌. workspace member 라 PAT 하나로 cross-project
  API 호출 가능. daemon + MCP wrapper 가 강제 격리.
- 사용자 측 leak 시점 (`.env.local` 공유, log 노출) 의 보안은
  파일 chmod 600 + gitignore + 사용자 교육으로 mitigation.

## 10. 완료 정의

- [ ] PR-1 ~ PR-9 모두 main 에 merge
- [ ] 운영자 인스턴스에서 `genasis provision --team "Test Team N"
      --app "Test App"` 실행 시 10 agent + 1 human invite + Plane
      workspace + project + MM team + scrum channel 모두 생성
- [ ] 같은 명령 재실행 시 모든 step "already exists, skipping" 으로
      no-op
- [ ] `genasis listen --real` 띄운 후 사용자가 MM 의 scrum 채널에
      메시지 입력 → PM agent 가 응답 → frontend agent dispatch →
      Plane 카드 transition → … 전체 reactive chain 이 trial flavor
      와 동일하게 동작
- [ ] self-host (`docker-compose up`) 에서 같은 시나리오 PASS
- [ ] ADR-019 의 "Implementation status" 섹션 업데이트
- [ ] 이 파일 (`genasis-team-plan.md`) 삭제 또는 `docs/archive/` 로
      이동

---

**다음 사이클 시작 시 첫 작업**: §9.1 / §9.2 / §9.3 — Plane + MM
admin token 으로 핵심 endpoint 들 curl 로 직접 호출해서 실제 응답
shape + 권한 검증. 그 결과로 §2.4 / §2.7 / §3.4 의 실제 endpoint
path 확정.
