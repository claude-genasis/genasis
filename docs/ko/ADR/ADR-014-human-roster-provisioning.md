> English: [`../../ADR/ADR-014-human-roster-provisioning.md`](../../ADR/ADR-014-human-roster-provisioning.md)

# ADR-014: 사람 로스터 프로비저닝 — 사람을 일급 팀원으로 등록

## 상태

제안됨 (2026-05-10).

## 맥락

Genasis 의 북극성(CLAUDE.md §Core Philosophy)은 "AI 에이전트를 사람이 이미
사용하는 협업 도구(Plane, Mattermost) 안에서 일급 팀원이 되게 한다" 이다.
하지만 v0.1 까지 자동 프로비저닝되는 것은 **에이전트 봇 계정**뿐이고
**사람 계정**은 수동으로 만들어야 했다 — 즉, 사람은 도구를 따로 셋업하고
agentic team 은 사람의 정체성을 알지 못한 채 모든 비-봇 메시지를
"누군가 사람일 것" 이라는 휴리스틱으로 처리하고 있었다.

### 구체적 결함

1. **온보딩 비대칭**: 신규 팀에서 `genasis init` 후 에이전트 10명은 즉시
   Mattermost / Plane 에 등장하지만 실제 사람 멤버는 별도로 가입해야 한다.
   "turnkey bootstrap" 미션 위배.

2. **요구사항 신뢰도 부재**: 에이전트는 누가 stakeholder 인지 모른다. 외부
   게스트, QA 인턴, 잘못 들어온 동료의 농담까지 모두 동등하게 "사람의 요청"
   으로 처리될 위험. 잘못된 발신자에게 결정 권한을 주는 건 큰 운영 사고의
   원인이 된다.

3. **신원 추적 불가**: 어떤 Mattermost 메시지가 어떤 사람에게서 왔는지를
   에이전트가 검증할 수 있는 메커니즘이 없다. 봇 사칭이나 채널 누수 시
   가드레일이 없다.

## 결정

`genasis.toml` 에 `[[humans]]` 배열을 1차 시민으로 추가하고, 에이전트와 동일
수준의 자동 프로비저닝 + 런타임 인식 프로토콜을 적용한다.

### 1. 데이터 모델 (genasis-core)

```toml
[[humans]]
name        = "Bravo"
email       = "gnoopy@gmail.com"
role        = "stakeholder"   # stakeholder | pm-human | reviewer | ...
mm_username = ""              # 비어 있으면 email local-part 에서 도출
locale      = "ko"            # 시스템 메시지 언어
```

`HumanEntry` 구조체와 `HumansLock` (`.genasis/humans.lock.toml`) 두 파일로
분리한다:
- `genasis.toml` 의 `[[humans]]` 는 **사람이 편집하는 SSOT** — 깨끗하게
  유지, 커밋 가능.
- `.genasis/humans.lock.toml` 은 **프로비저닝 부산물** (Mattermost user_id,
  Plane user_id, 임시 비밀번호) — gitignore 권장.

### 2. Mattermost 프로비저닝

`MattermostProvider::ensure_human_user(spec, team_id)` 트레잇 메서드 추가.
업스트림 구현은:

1. `GET /api/v4/users/email/{email}` 으로 멱등 조회 → 존재하면 `temp_password
   = None` 으로 반환.
2. 없으면 `POST /api/v4/users` 로 관리자 즉시 생성 + 24자 고엔트로피 임시
   비밀번호 (대/소/숫자/심볼 각 1개 보장 — Mattermost 의 가장 엄격한 비밀번호
   정책 통과).
3. 임시 비밀번호는 첫 로그인 시 변경 강제 (best-effort `/users/{id}/password`
   호출).
4. team_id 가 있으면 팀에 추가 (이미 있으면 skip — 멱등).
5. `HumanUserRef { user_id, username, email, temp_password: Some(...),
   must_change_password: true }` 반환. 임시 비번은 `.genasis/humans.lock.toml`
   에 기록되며, 사용자가 첫 로그인 후 빈 문자열로 갱신.

**선택 근거 (admin-create vs invite-email)**: invite-email 이 보안적으로 더
우수하지만 Mattermost SMTP 가 거의 사설 환경에서 미설정 상태인 경우가 많다.
Genasis 의 turnkey 미션을 우선해 admin-create 경로를 기본으로 삼고, 향후
SMTP 가 활성화된 환경을 위해 `[mattermost] human_provision_mode = "invite"`
플래그 도입을 v2 로 보류.

### 3. Plane 프로비저닝

기존 `provision-plane-users.mjs` Playwright 스크립트의 `ProvisionInput` 에
`humans: HumanRequest[]` 필드 추가. 사람은 `plane_role = "Member"` 로 워크
스페이스에 추가, PAT 은 발급하지 않는다 (사람은 UI 로 인증).

스크립트가 아직 stub 단계이므로 `humans` 출력은 placeholder ID 를 echo
한다. 실제 UI 자동화 포팅 시 이 필드만 채워주면 Rust 측 `humans.lock.toml`
는 자동으로 진짜 user_id 로 갱신된다.

### 4. 런타임 — 요구사항 인테이크 프로토콜

`agents/GENASIS.md.tera` 에 두 새 섹션을 추가:

#### `## 사람 로스터`
프로비저닝된 사람의 이름·이메일·MM username·역할 라벨을 표 형태로 노출.
모든 에이전트가 컨텍스트에서 이 표를 읽는다.

#### `### 요구사항 수신 프로토콜`
`#scrum-{project}` 의 새 메시지 발신자를 3 분류:

1. **로스터에 있는 사람**: 메시지는 **바인딩 stakeholder 요구사항**.
   - PM 5분 내 `🟢 접수: <한 줄 요약>` 회신
   - Plane 이슈 생성 또는 기존 이슈 연결 (원문 인용)
   - 적절한 역할로 `assignees` 라우팅
   - 우선순위: `stakeholder > pm-human > reviewer > 기타`

2. **로스터에 없는 사람**: PM 만 응답, `QUESTION` 라벨, 신원 확인 후 처리.

3. **봇** (`from_bot=true` 또는 `*-bot` username): 기존 에이전트-에이전트
   프로토콜 사용, 인테이크 건너뜀.

PM 과 Planner 의 overlay (en/ko 양쪽) 에도 동일 프로토콜을 짧게 mirror
하여, GENASIS.md 컨텍스트가 trim 되더라도 핵심 행동이 유지되도록 한다.

### 5. UX — TUI Wizard CRUD + CLI

- `genasis init` / `genasis attach` 의 wizard 에 5번째 단계 **Humans** 추가
  (Env → Lang → Team → Connect → **Humans** → Overlay → Done).
- `a` 추가, `e` 편집, `d` 삭제, `s` Mattermost+Plane 동기화, `Enter` 다음 단계.
- 동일한 작업을 `genasis humans add | edit | remove | list | sync` CLI 로도
  제공 (CI / 스크립트 친화).
- wizard 를 다시 실행하면 `[[humans]]` 와 `humans.lock.toml` 을 함께 로드해
  현재 상태를 그대로 보여주고 추가 편집 가능 — "재실행이 곧 편집기" UX.

## 결과

### 좋은 점
- 미션 일치: 사람-에이전트 비대칭 제거 (`genasis init` 한 번에 둘 다
  도구에 등장).
- 요구사항 신뢰도 향상: 에이전트가 "이 발신자는 우리 stakeholder 인가?"
  를 결정 가능. 봇 사칭에도 1차 가드레일.
- 기존 워크플로 보존: 미등록 발신자는 여전히 사람으로 간주 (PM 이 식별
  → 처리). 후방 호환.

### 비용
- 임시 비밀번호 보관: `humans.lock.toml` 에 평문으로 잠시 저장 (첫 로그인
  후 자동 클리어). gitignore 필수.
- Mattermost SMTP 가 활성화된 환경에서는 invite-email 모드가 더 적합 —
  v2 에서 플래그로 노출.
- Plane Playwright 자동화가 아직 stub 이라 실제 user_id 매핑은 후속 M
  에서 완결.

### 후속 작업
- M20.1: schema + tests
- M20.2: MM `ensure_human_user` 구현
- M20.3: Plane provisioner humans 필드
- M20.4: cmd_humans CLI + cmd_init wiring
- M20.5: TUI Humans 단계
- M20.6: GENASIS.md / overlay 프롬프트 갱신
- M20.7: ADR + progress + 이중언어 미러 (이 ADR)
- v2 후속: invite-email 모드, Plane Playwright UI 포트, OAuth/SSO 통합

## 대안과 거부 이유

- **모든 비-봇 메시지를 사람으로 간주 (현재 동작 유지)**: 미션 위배,
  외부 사칭에 무방비.
- **엄격 모드 — 등록된 사람만 응답**: 외부 손님/QA 인턴 인바운드를 막아
  유연성 손실. 옵션으로만 v2 에 도입 검토.
- **invite-email 우선**: SMTP 가 비활성인 환경이 너무 많아 turnkey 미션
  위배. v2 에서 플래그로 추가.
