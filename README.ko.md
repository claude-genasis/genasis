<div align="center">

# Genasis

**AI 에이전트를 진짜 팀원으로 — 사람과 함께 일하게.**

하나의 명령어로 Plane과 Mattermost를 통해 사람 팀원과 동일한 워크플로로 협업하는 에이전트 개발팀을 설치합니다.

[![CI](https://img.shields.io/github/actions/workflow/status/claude-genasis/genasis/ci.yml?branch=main&label=CI&style=flat-square&logo=github)](https://github.com/claude-genasis/genasis/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/codecov/c/github/claude-genasis/genasis?branch=main&style=flat-square&logo=codecov)](https://codecov.io/gh/claude-genasis/genasis)
[![Nightly E2E](https://img.shields.io/github/actions/workflow/status/claude-genasis/genasis/nightly-e2e.yml?branch=main&label=nightly%20E2E&style=flat-square&logo=github)](https://github.com/claude-genasis/genasis/actions/workflows/nightly-e2e.yml)
[![Release](https://img.shields.io/github/v/release/claude-genasis/genasis?include_prereleases&style=flat-square&logo=github&label=release)](https://github.com/claude-genasis/genasis/releases)
[![License](https://img.shields.io/github/license/claude-genasis/genasis?style=flat-square)](LICENSE)
[![Stars](https://img.shields.io/github/stars/claude-genasis/genasis?style=flat-square&logo=github)](https://github.com/claude-genasis/genasis/stargazers)
[![Rust](https://img.shields.io/badge/rust-stable-orange?style=flat-square&logo=rust)](rust-toolchain.toml)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20WSL-blue?style=flat-square)](#지원-플랫폼)

[**English**](README.md)&nbsp;&nbsp;|&nbsp;&nbsp;[**한국어**](README.ko.md)

</div>

---

`genesis` · `genasis` · `agent-creation` · `agent-harness` · `agentic-team` · `agent-team` · `agentic-scrum` · `agentic-sdlc` · `claude-code` · `claude-code-plugins` · `claude-code-subagents` · `agentic-ai` · `ai-agent-orchestration` · `multi-agent-system` · `plane-project-management` · `mattermost-bot` · `sprint-automation` · `tdd` · `scrum-automation` · `coding-agents` · `rust-cli` · `self-hosted-ai` · `ai-software-development`

---

<p align="center">
  <img src="docs/assets/genasis-banner-en.png" alt="Genasis — AI 에이전트 팀: 아이디어를 말하면, 우리가 만듭니다" width="100%">
</p>

## 문제

오늘날 AI 코딩 에이전트는 **고립된 도구**입니다:
- 이슈 트래커에서 티켓을 가져가지 않습니다
- 작업하면서 상태를 업데이트하지 않습니다
- 팀 채팅에서 질문하지 않습니다
- 사람이 볼 수 있는 채널에서 다른 에이전트와 협업하지 않습니다
- 사람 개발자와 스프린트 세레모니를 함께하지 않습니다

한편, **Claude Code**를 쓰는 모든 엔지니어링 팀은 결국 같은 접착제를 만듭니다: 에이전트를 Plane/Linear/Jira에 연결하고, Mattermost/Slack 봇을 연동하고, TDD 게이트를 강제하고, 디자인 핸드오프를 관리합니다. 대부분 유지보수하고 싶지 않은 bash 스크립트입니다.

## Genasis가 하는 일

Genasis는 하나의 Rust 바이너리로 AI 에이전트를 **진짜 팀원**으로 만듭니다:

| 기능 | 동작 방식 |
|---|---|
| **에이전트 마켓플레이스** | [ECC](https://github.com/affaan-m/everything-claude-code), [wshobson/agents](https://github.com/wshobson/agents), [VoltAgent](https://github.com/VoltAgent/awesome-claude-code-subagents), [dl-ezo](https://github.com/dl-ezo/claude-code-sub-agents)에서 선별한 20+ 에이전트. 카테고리별 브라우징, 개별/프리셋 설치. |
| **이슈 트래커 연동** | Plane REST API 직접 연동. 에이전트가 티켓 소유, 라이프사이클 전환 (Todo → In Progress → In Review → Done), 하위 이슈 생성. |
| **팀 채팅 연동** | 에이전트 역할당 Mattermost 봇 1개. 티켓당 스레드 1개. 사람과 같은 채널에서 토론, 에스컬레이션, 조정. |
| **비파괴 오버레이** | `.claude/agents/*.md` 안에 marker fence. 기존 에이전트 정의 그대로 보존. `genasis detach`로 깨끗하게 제거. |
| **스프린트 자동화** | 13개 슬래시 명령 + 5개 hook: `/sprint-start`, `/issue-done`, `/db-migrate`, 세션 hook, QA 게이트. |
| **디자인 시스템 관리** | `genasis design swap`으로 디자인 토큰 교체 + 영향 UI 영역 Plane 이슈 자동 생성. |
| **DB 스키마 관리** | SQL guard (읽기 전용), Atlas/Drizzle Kit 마이그레이션, DuckDB raw runner. |
| **실시간 모니터** | Ratatui TUI 대시보드: 스프린트, 토큰, 에이전트 활동, 배포 상태. |
| **완전 가역** | `genasis detach` — genasis가 추가한 모든 것 제거. 잔여물 없음. |

---

## 빠른 체험 — 5분 만에 에이전트 팀 가동

**5단계면 에이전트 팀이 스프린트를 돌립니다.** 서버 설치 불필요.

**1. 설치**

```bash
curl -fsSL https://raw.githubusercontent.com/claude-genasis/genasis/main/install.sh | sh
```

**2. Trial 모드로 초기화** — 인터랙티브 데모 + 체험 신청 화면이 브라우저에 열립니다

```bash
mkdir my-project && cd my-project && genasis init --trial
```

**3. 샘플 PRD 생성** — 에이전트가 바로 작업할 수 있는 요구사항 문서

```bash
genasis example prd
```

**4. 에이전트 팀 가동**

```bash
genasis init
```

**5. 스프린트 모니터링**

```bash
genasis monitor
```

끝입니다. 에이전트 팀이 PRD에서 코드까지 스프린트를 완주했습니다.
디자인 교체, PRD 확장, 에이전트 추가 등 실습은
[**전체 튜토리얼**](docs/ko/TUTORIAL.md)을 참조하세요.

<details>
<summary>소스에서 직접 빌드 (install.sh 대신)</summary>

```bash
git clone https://github.com/claude-genasis/genasis.git && cd genasis && ./build.sh
```

</details>

---

## 단계별 가이드

모든 단계를 직접 통제하고 싶은 팀을 위한 안내입니다.

### 1. 설치

```bash
curl -fsSL https://raw.githubusercontent.com/claude-genasis/genasis/main/install.sh | sh
```

### 2. Plane & Mattermost 준비

Genasis 에이전트는 **Plane** (이슈 관리)과 **Mattermost** (팀 채팅)를 통해 협업합니다.

**방법 A — Trial 서버 (가장 빠름, 설치 불필요)**

[**trial.realstory.blog**](https://trial.realstory.blog)에서 공유 환경을 신청합니다.
관리자에게 접근 요청 → 수 분 내 접속 정보. 협의 하에 기간 제한 없이 이용 가능.

**방법 B — 직접 설치 (완전한 통제)**

```bash
cd servers && docker compose up -d
```

Plane `localhost:8080`, Mattermost `localhost:8065`.
상세: [`servers/README.md`](servers/README.md).

설치 후 인증 정보 설정:

```bash
export PLANE_API_KEY="your-plane-api-key"
export MM_ADMIN_TOKEN="your-mattermost-token"
```

### 3. 연결 및 시작

```bash
genasis init
```

### 4. 검증

```bash
genasis doctor
```

---

## 에이전트가 사람과 협업하는 방식

```mermaid
sequenceDiagram
    actor Human as 사람 개발자
    participant Plane as Plane 보드
    participant MM as Mattermost
    participant FE as 프론트엔드 에이전트
    participant QA as QA 에이전트

    Human->>Plane: 티켓 생성
    Plane-->>FE: 할당됨 (webhook)
    FE->>Plane: Todo → In Progress
    FE->>MM: "#142 작업 시작합니다"
    FE->>FE: 코드 작성 + 테스트
    FE->>Plane: In Progress → In Review
    FE->>MM: "PR #87 리뷰 부탁드립니다"
    Human->>MM: "좋아요, L42 작은 수정 부탁"
    FE->>FE: 피드백 반영
    FE->>MM: "수정 완료 — PR 업데이트됨"
    QA->>QA: 테스트 스위트 실행
    QA->>Plane: In Review → Done
    QA->>MM: "모든 검증 통과 ✓"
    Human->>Plane: 보드에서 Done 확인
```

Plane 보드나 Mattermost 채널을 보는 사람은 업데이트가 사람에게서 온 건지 에이전트에게서 온 건지 **구분할 수 없고, 구분할 필요도 없습니다.**

## 사용 사례

| 팀 유형 | genasis 활용 방식 |
|---|---|
| **스타트업 (2-5명)** | AI 에이전트로 소규모 팀 배가. 리뷰, 테스트, 보안 스캔을 에이전트가 담당. 기존 Plane + Mattermost에 합류. |
| **에이전시 / 컨설팅** | 클라이언트 프로젝트별 에이전트 팀 즉시 배치. 프리셋 설치 → 바로 생산성. |
| **엔터프라이즈 스쿼드** | 기존 `.claude/agents/`에 비파괴 오버레이. 현재 워크플로 방해 없이 Plane/MM 연동 추가. |
| **솔로 개발자** | Claude Code 구독 하나로 PM + 아키텍트 + QA + 보안 팀 확보. 에이전트가 프로세스를, 당신이 비전을 담당. |
| **오픈소스 메인테이너** | PR 리뷰, 보안 스캐닝, 테스트 강제 자동화. 커뮤니티 기여자가 같은 이슈 스레드에서 에이전트 피드백 확인. |

## CLI 주요 명령

```bash
# 팀 라이프사이클
genasis init                   # Plane 프로젝트 + MM 채널 + 에이전트 설치
genasis attach                 # 기존 에이전트에 오버레이 (비파괴)
genasis detach                 # 오버레이 제거 (완전 가역)
genasis doctor                 # 환경 + 연결 검증
genasis upgrade                # 오버레이 프로토콜 최신화

# 에이전트 마켓플레이스
genasis agents browse          # 카테고리 → 선택 → 설치 (interactive)
genasis agents install <이름>  # 에이전트 1개 설치
genasis agents install --preset web-app  # 프리셋 팀 설치 (9역할)
genasis agents list            # 사용 가능한 에이전트 목록
genasis agents installed       # 이 프로젝트에 설치된 에이전트

# 운영
genasis monitor                # 실시간 TUI 대시보드
genasis design swap <ref>      # 디자인 시스템 교체
genasis db query "SELECT ..."  # 읽기 전용 SQL
genasis lang switch <en|ko>    # 에이전트 언어 전환
```

## 지원 플랫폼

| 플랫폼 | 상태 |
|---|---|
| **Linux** (x86_64, aarch64) | 지원 |
| **macOS** (Apple Silicon, Intel) | 지원 |
| **WSL** (Windows Subsystem for Linux) | 지원 |
| Windows (네이티브) | 미지원 — WSL 사용 |

## 아키텍처

```mermaid
flowchart TB
  L0["L0 — 기존 팀<br/>.claude/agents/*.md · src/ · DB"]
  L1["L1 — Genasis 오버레이<br/>marker fence · GENASIS.md · .claude/genasis/"]
  L2["L2 — Genasis CLI<br/>init · attach · agents · db · design · monitor"]
  L3["L3 — Plane · Mattermost · GitHub"]
  L0 -. "보존 (비파괴)" .-> L1
  L2 -- "생성 + 병합" --> L1
  L1 -- "REST API 직접 통신" --> L3
```

## 대안 비교

| 기능 | **Genasis** | [ECC](https://github.com/affaan-m/everything-claude-code) | [wshobson/agents](https://github.com/wshobson/agents) | [VoltAgent](https://github.com/VoltAgent/awesome-claude-code-subagents) |
|---|---|---|---|---|
| 이슈 트래커 연동 (Plane) | ✅ 직접 API | 수동 | — | — |
| 팀 채팅 연동 (Mattermost) | ✅ 역할당 봇 | — | — | — |
| 비파괴 오버레이 | ✅ marker fence | — | — | — |
| 에이전트 마켓플레이스 | ✅ 20+ 에이전트 | 48 (전체 설치) | 185 (플러그인) | 131 (복사) |
| 스프린트 자동화 | ✅ 13명령 + 5hook | — | — | — |
| 디자인 교체 | ✅ | — | — | — |
| 스키마 관리 | ✅ | — | — | — |
| 모니터 TUI | ✅ Ratatui | — | — | — |
| 완전 가역 (detach) | ✅ | — | — | — |
| 단일 바이너리 | ✅ Rust | bash | npm | shell |
| 다국어 | ✅ en/ko | — | — | — |

## 가이드

| 가이드 | 내용 |
|---|---|
| [**상세 빠른 시작**](docs/ko/QUICKSTART.md) | 설치 → 설정 → 첫 스프린트 전체 워크스루 |
| [**서버 설치**](servers/README.md) | Plane + Mattermost를 `docker-compose up` 하나로 자체 호스팅 |
| [**에이전트 마켓플레이스**](docs/ko/AGENTS-MARKETPLACE.md) | 카테고리 브라우징, 프리셋, `/install-agent` 명령 |
| [**디자인 교체**](docs/ko/DESIGN-SWAP-GUIDE.md) | 디자인 시스템 교체, 복원, 오버라이드, EPIC 모드 |
| [**크레딧 & OSS 출처**](docs/ko/CREDITS.md) | genasis가 참조한 오픈소스 프로젝트들 |

## 감사의 말

Genasis는 오픈소스 커뮤니티의 에이전트를 큐레이션하고 통합합니다.
전체 출처 및 링크: [**docs/ko/CREDITS.md**](docs/ko/CREDITS.md).

| 프로젝트 | 활용 내용 | 라이선스 |
|---|---|---|
| [everything-claude-code (ECC)](https://github.com/affaan-m/everything-claude-code) | code-reviewer, architect, security-reviewer 에이전트 | MIT |
| [wshobson/agents](https://github.com/wshobson/agents) | frontend-developer, backend-developer 에이전트 | MIT |
| [VoltAgent](https://github.com/VoltAgent/awesome-claude-code-subagents) | qa-tester, DevOps 에이전트 | MIT |
| [dl-ezo](https://github.com/dl-ezo/claude-code-sub-agents) | planner, 요구사항 라이프사이클 에이전트 | MIT |
| [Plane](https://github.com/makeplane/plane) | 이슈 추적 + 프로젝트 관리 플랫폼 | AGPL-3.0 |
| [Mattermost](https://github.com/mattermost/mattermost) | 팀 메시징 + 봇 플랫폼 | Various |
| [Ratatui](https://github.com/ratatui/ratatui) | 터미널 UI 프레임워크 (모니터 대시보드) | MIT |

## 문서

| | English | 한국어 |
|---|---|---|
| 블루프린트 | [blueprint.md](blueprint.md) | [blueprint.ko.md](blueprint.ko.md) |
| 진행 상황 | [progress.md](progress.md) | [progress.ko.md](progress.ko.md) |
| 아키텍처 | [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | [docs/ko/ARCHITECTURE.md](docs/ko/ARCHITECTURE.md) |
| 프로바이더 | [docs/PROVIDERS.md](docs/PROVIDERS.md) | [docs/ko/PROVIDERS.md](docs/ko/PROVIDERS.md) |
| 토큰 이코노믹스 | [docs/TOKEN-ECONOMICS.md](docs/TOKEN-ECONOMICS.md) | [docs/ko/TOKEN-ECONOMICS.md](docs/ko/TOKEN-ECONOMICS.md) |
| ADR | [docs/ADR/](docs/ADR/) | [docs/ko/ADR/](docs/ko/ADR/) |

## 기여

[`CONTRIBUTING.md`](CONTRIBUTING.md)를 참조하세요. **debug-history 패치** 기여도 가능합니다 — fork 없이 `genasis debug submit`으로 현장 수정 사항을 genasis 개선에 반영합니다.

## Star History

<a href="https://star-history.com/#claude-genasis/genasis">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=claude-genasis/genasis&type=Date&theme=dark">
    <img src="https://api.star-history.com/svg?repos=claude-genasis/genasis&type=Date" alt="Star history" width="640">
  </picture>
</a>

## 라이선스

MIT — [`LICENSE`](LICENSE) 참조.

<div align="center">

AI 에이전트를 고립된 코드 생성기가 아닌, 진짜 팀원으로 만드는 엔지니어링 팀을 위해.

[**English**](README.md)&nbsp;&nbsp;|&nbsp;&nbsp;[**한국어**](README.ko.md)

</div>
