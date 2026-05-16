<div align="center">

# Genasis

**실제 팀 협업을 위한 AI 에이전트 오케스트레이션**

여러분 팀이 이미 쓰는 협업 도구 (Plane / Mattermost) 위에 10인
에이전트 팀을 얹어보세요 — 같은 티켓 보드, 같은 채팅 스레드, 같은
스프린트 흐름.

[![CI](https://img.shields.io/github/actions/workflow/status/claude-genasis/genasis/ci.yml?branch=main&label=CI&style=flat-square&logo=github)](https://github.com/claude-genasis/genasis/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/claude-genasis/genasis?include_prereleases&style=flat-square&logo=github&label=release)](https://github.com/claude-genasis/genasis/releases)
[![License](https://img.shields.io/github/license/claude-genasis/genasis?style=flat-square)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20WSL-blue?style=flat-square)](#지원-플랫폼)

[**English**](README.md)&nbsp;&nbsp;|&nbsp;&nbsp;[**한국어**](README.ko.md)

</div>

---

<p align="center">
  <img src="docs/assets/genasis-banner-ko.png" alt="Genasis — AI 에이전트 팀" width="100%">
</p>

## 무엇을 얻나

`genasis` 는 10인 에이전트 팀 (PM, frontend, backend, devops,
designer, QA, planner, architect, code-reviewer, security) 을
**Plane 티켓** + **Mattermost 채팅** 위에 얹습니다. 사람이 평소처럼
채팅에 요청을 던지면 Claude 가 PM 페르소나로 받아 frontend → devops
→ QA 로 분배하고, 카드를 transition 하고, 스레드에 답변을 달고,
완성된 결과물을 운영자 호스팅 미리보기로 ship.

두 가지 사용 방식:

| Flavor | 정체 | 추천 |
|--------|------|------|
| **Trial** | 브라우저만 있으면 됨. `mmplane-trial.realstory.blog` 의 채팅 + 칸반 + 쇼케이스 iframe. | 첫 체험, 데모, 워크플로우 학습 |
| **Real** | 우리 팀이 실제 쓰는 Plane workspace + Mattermost 서버. 에이전트가 진짜 팀원 처럼 가입. | 실 업무 |

두 flavor 모두 같은 데몬 (`genasis listen`) + 같은 에이전트
overlay 사용. API URL 만 다름.

## Quick Path — 5분 만에 채팅하는 팀

```bash
# 1. 설치
curl -fsSL https://raw.githubusercontent.com/claude-genasis/genasis/main/install.sh | sh

# 2. 트라이얼 팀 bootstrap
mkdir my-team && cd my-team
genasis init --trial --name "My Team"
```

`init --trial` 단일 명령이 내부적으로 네 가지를 수행: 트라이얼 팀
bootstrap → 쇼케이스 활성화 → reactive 데몬 시작 → 팀 토큰이 query
param 으로 포함된 라이브 URL 출력.

3. **출력된 URL 열기**. Live Trial 탭에 칸반 + 채팅 + 쇼케이스 패널.
4. **채팅 패널에 요청 입력** — 한국어 OK:

   > 다크모드와 i18n 지원되는 퀴즈 앱 만들어줘

   1분 안에 PM 이 받아서 ack 하고, frontend 가 로컬 sandbox 에 Vite
   + React + TS 프로젝트 scaffold, devops 가 `npm run build &&
   genasis push` 로 운영자 trial-app 에 정적 자산 업로드, 좌측
   "결과보기" 핸들이 `준비중` → 라이브 iframe 으로 전환.
5. **끝낼 때**: `genasis stop`.

핵심은 한 명령으로 시작 + 한 명령으로 종료.

## Production 으로 — 실 Plane + Mattermost

트라이얼 졸업하면 실제 stack 으로. 양쪽 admin token 필요:

```bash
export PLANE_URL=https://plane.your-company.com
export PLANE_ADMIN_TOKEN=...        # Plane admin UI 에서 발급
export MM_URL=https://mm.your-company.com
export MM_ADMIN_TOKEN=...           # Mattermost system-admin PAT

genasis provision \
  --team "Marketing Squad" \
  --app "Quiz Demo" \
  --humans "Bravo Kim <gnoopy@gmail.com>"
```

자동 처리 (멱등, 재실행 안전):

1. Plane 프로젝트 생성 + 10개 default agent 첨부 + 인간 invite
2. Mattermost team (`team-<slug>`) + scrum channel (`scrum-<slug>`)
   + agent 별 MM user + PAT 발급 + 멤버십
3. `genasis.toml` (식별자) + `.env.local` (agent 별 토큰, chmod 600)
   프로젝트 루트에 작성

운영 중 멤버 변동은 re-provision 불필요:

```bash
genasis team list                                   # 현재 roster
genasis team add human "Charlie <charlie@x.com>"
genasis team add agent designer
genasis team remove agent designer                  # history 보존
```

**Slug 자동 약어** 5자 (`Marketing Squad` → `ms`). 충돌 시
`--team-slug` / `--app-slug` 로 override. 한글 팀 이름은 로컬
`claude` CLI 로 영문 번역 후 약어.

전체 명세는 [ADR-019](docs/ko/ADR/ADR-019-real-provisioning.md).

## 운영자용 (genasis-as-a-service 호스팅)

여러 테넌트를 호스팅한다면 비밀을 private repo 에 버전 관리:

```bash
export GENASIS_SECRETS_ROOT=/path/to/agents-pool/secrets
genasis provision --team "Tenant Co" --app "Their App" --humans "..."
# secrets/teams/<slug>/{genasis.toml.snapshot,.env.local,provision.log} 작성
git -C agents-pool add secrets/ && git commit && git push
```

Snapshot ownership 검증 (alpha.34+) 으로 자동 약어 충돌 시 두 테넌트가
서로의 리소스를 silently 상속받는 사고 차단 — 같은 snapshot dir 에서
재실행은 ID 일치로 Reuse, 신규 실행에 같은 slug 진입은 명시적
"another tenant owns this identifier" 에러.

## CLI 레퍼런스

```
# 팀 lifecycle
genasis init --trial --name "X"   # 한 명령 Quick Path
genasis init                       # 실 Plane/MM (`provision` 후)
genasis provision ...              # 실 flavor bootstrap (admin token)
genasis team {add|remove|list}     # day-2 멤버십

# Runtime
genasis status                     # 데몬 + URL + 최근 활동
genasis stop                       # 데몬 종료
genasis logs -f                    # 데몬 로그 follow
genasis monitor                    # 전체 TUI 대시보드

# Agent catalog
genasis agents                     # catalog 에서 agent browse + install
genasis humans {add|sync|list}     # 인간 협업자

# Showcase (alpha.39+)
genasis push                       # 정적 bundle 빌드해서 운영자에 ship

# Power-user
genasis listen {start|stop|status|logs|restart}   # 데몬 long form
genasis publish                    # 수동 app_status flip
genasis doctor                     # 환경 점검
genasis debug                      # drift / debug-history
```

## 에이전트가 사람과 협업하는 방식

- **같은 채널, 같은 보드**. 에이전트가 Mattermost 스레드에서 사람
  메시지 아래로 답변하고, Plane 카드를 일반 팀원처럼 transition.
  어제 standup 보는 사람이 어느 게 에이전트인지 구분 불필요 (또는
  구분이 필요한 부분은 actor chip 으로 명시).
- **PM 에이전트가 orchestrate**. 사람이 요청하면 PM 이 받아서 티켓
  분할 → frontend / devops 를 `Task` tool 로 dispatch. 각 sub-agent
  가 같은 스레드에 보고.
- **기존 agent 재사용**. ECC, knowledge-work-plugins 등으로 만든
  `.claude/agents/` 가 이미 있으면 overlay 가 marker-fence patch
  로 부착 — 기존 agent 정의는 그대로.

## 문제 해결

### "결과보기" 핸들이 "준비중" 회색

`genasis publish` 가 실행 안 됨. Quick Path 의 `init --trial` 이
자동 실행하지만 실패했을 수 있음. 수동:

```bash
cd <project-dir>
genasis publish
```

### 채팅 / 칸반 실시간 업데이트 안 됨

alpha.38+ 부터 양쪽 패널 SSE auto-reconnect 적용. 옛 binary 면 한
번 새로고침 후 daemon 도 최신으로 재시작:

```bash
~/.local/bin/genasis --version       # alpha.38+ 기대
genasis listen restart --trial       # 옛 binary 의 데몬이면 재시작
```

데몬은 추가로 10초 fallback poll 도 돌리므로 단일 SSE drop 으로 보드가
stale 되지 않음.

### 쇼케이스 iframe 이 "빌드 중" 인데 데몬 로그 조용함

데몬은 **reactive** — 사람이 채팅 패널에 메시지 입력해야 Claude
세션을 spawn. 트라이얼 seed message ("환경 준비 완료") 는 `publish`
가 보낸 것이지 실제 빌드가 아님. 채팅 패널에 실제 요청 입력:

> 다크모드 + i18n 지원되는 퀴즈 앱 만들어줘

30초 안에 PM 이 ack.

### Plane / Mattermost slug 충돌 (multi-tenant 호스팅)

```
Error: Plane project identifier "QUIZ" already exists (id=...).
Pick a different `--app-slug`.
```

두 팀이 같은 자동 약어로 들어옴. `--team-slug` / `--app-slug` 명시.
ADR-019 §Slug collision 참조.

## 지원 플랫폼

Linux x86_64, Linux aarch64, macOS (Intel + Apple Silicon), Windows
은 WSL2. Linux 는 static musl binary, 그 외 `cargo install --git`.

## 아키텍처

요약: Rust CLI + JSONL hook bus + Plane/MM adapter + reactive
listen 데몬 (사람 메시지마다 `claude --print` spawn). 전체 도식 +
데이터 흐름은 [docs/ko/ARCHITECTURE.md](docs/ko/ARCHITECTURE.md).

## 문서

- [TUTORIAL](docs/ko/TUTORIAL.md) — 실습 walkthrough
- [ARCHITECTURE](docs/ko/ARCHITECTURE.md) — 구성요소 + 데이터 흐름
- [PROVIDERS](docs/ko/PROVIDERS.md) — Plane / Mattermost adapter 계약
- [ADR](docs/ko/ADR/) — 설계 결정 ([019 real provisioning](docs/ko/ADR/ADR-019-real-provisioning.md), [020 showcase push](docs/ko/ADR/ADR-020-showcase-push.md))
- [CONTRIBUTING](CONTRIBUTING.ko.md)

## 라이선스

MIT — [LICENSE](LICENSE).
