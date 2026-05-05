> English: [../CREDITS.md](../CREDITS.md)

# 크레딧 & 오픈소스 출처

Genasis는 훌륭한 오픈소스 프로젝트들 위에 구축되었습니다.
큐레이션하고 통합하며 가치를 더합니다 — 바퀴를 재발명하지 않습니다.

## 에이전트 소스

이 프로젝트들이 genasis 에이전트 마켓플레이스에서 제공하는 큐레이션된
에이전트 정의의 원천입니다. 모두 명시된 라이선스 하에 사용합니다.

| 프로젝트 | Stars | genasis 활용 내용 | 라이선스 |
|---|---|---|---|
| [everything-claude-code (ECC)](https://github.com/affaan-m/everything-claude-code) | 173K+ | `code-reviewer` (골드 스탠다드 — 80% 신뢰도 필터링), `architect` (시스템 + 코드 이중 레이어), `security-reviewer`, `silent-failure-hunter`, `a11y-architect`, `refactor-cleaner`, 언어별 리뷰어 (TypeScript, Python, Rust, Go) | MIT |
| [wshobson/agents](https://github.com/wshobson/agents) | 34K+ | `frontend-developer` (React 19, Next.js 15, RSC), `backend-developer` (API + 이벤트 소싱 + TDD), `full-stack-orchestration` (배포 + 성능 + 보안) | MIT |
| [VoltAgent/awesome-claude-code-subagents](https://github.com/VoltAgent/awesome-claude-code-subagents) | 19K+ | `qa-tester` (테스트 자동화 + 접근성), `sre-engineer`, `devops-engineer`, 인프라 클러스터 (17 에이전트), 언어 전문가 | MIT |
| [dl-ezo/claude-code-sub-agents](https://github.com/dl-ezo/claude-code-sub-agents) | 184+ | `planner` (프로젝트 플래너 + 요구사항 분석), 전체 PM 라이프사이클 (user-story-generator, progress-tracker, risk-manager, stakeholder-communicator) | MIT |
| [0xfurai/claude-code-subagents](https://github.com/0xfurai/claude-code-subagents) | 884+ | 모바일 전문가: `ios-expert`, `swiftui-expert`, `android-expert`, `kotlin-expert`, `react-native-expert`, `flutter-expert` | MIT |

## 인프라

| 프로젝트 | genasis 활용 목적 | 라이선스 |
|---|---|---|
| [Plane](https://github.com/makeplane/plane) | 이슈 추적 + 프로젝트 관리 — 에이전트가 티켓 소유 + 라이프사이클 전환 | AGPL-3.0 |
| [Mattermost](https://github.com/mattermost/mattermost) | 팀 메시징 — 에이전트 역할당 봇 1개, 사람과 스레드 토론 | Various (Apache-2.0 / 독점) |
| [Caddy](https://github.com/caddyserver/caddy) | 리버스 프록시 + 자동 TLS (자체 호스팅 Plane + Mattermost용) | Apache-2.0 |

## Rust 생태계

| 크레이트 | genasis 내 용도 | 라이선스 |
|---|---|---|
| [Ratatui](https://github.com/ratatui/ratatui) | 모니터 TUI 대시보드 (스프린트, 토큰, 에이전트, 배포) | MIT |
| [Tera](https://github.com/Keats/tera) | 오버레이 렌더링용 템플릿 엔진 (Plane/MM 프로토콜 주입) | MIT |
| [Clap](https://github.com/clap-rs/clap) | CLI 인자 파싱 | MIT / Apache-2.0 |
| [Reqwest](https://github.com/seanmonstar/reqwest) | Plane/MM API + 에이전트 카탈로그 fetch HTTP 클라이언트 | MIT / Apache-2.0 |
| [Tokio](https://github.com/tokio-rs/tokio) | 비동기 런타임 | MIT |
| [rust-i18n](https://github.com/longbridgeapp/rust-i18n) | 런타임 국제화 (en/ko) | MIT |
| [Dialoguer](https://github.com/console-rs/dialoguer) | interactive CLI 프롬프트 (에이전트 마켓플레이스 TUI) | MIT |

## 연구 & 커뮤니티 참조

| 자료 | genasis 설계에 미친 영향 |
|---|---|
| [docs/famous-agents.md](../../docs/famous-agents.md) | Claude Code 에이전트 생태계 종합 조사 (방법론, 비교, 추천) |
| [obra/superpowers](https://github.com/obra/superpowers) | 워크플로 방법론 (TDD → 계획 → 서브에이전트 주도) — 스프린트 프로토콜 설계에 참조 |
| [BMAD-METHOD](https://github.com/bmad-code-org/BMAD-METHOD) | 스크럼/애자일 라이프사이클 스킬 — 슬래시 명령 설계에 참조 |
| [Anthropic Claude Code 문서](https://code.claude.com/docs) | 공식 서브에이전트 사양 + MCP 통합 패턴 |
| [docs/impact-of-multilang-prompts.md](../../docs/impact-of-multilang-prompts.md) | 단일 언어 에이전트 컨텍스트의 필수성 연구 (Claude drift 버그, arXiv 2406.20052) |

## 활용 방식

1. **에이전트 정의**는 큐레이션(선별, 호환성 검증)되어 genasis 에이전트
   마켓플레이스로 배포됩니다. 출처 표기는 각 에이전트 파일의 `## Source`
   섹션에 보존됩니다.

2. **오버레이 프로토콜** (Plane/MM 라이프사이클 규칙)은 genasis 오리지널
   콘텐츠입니다 — marker fence를 통해 커뮤니티 에이전트 안에 주입되며,
   위 프로젝트에서 파생된 것이 아닙니다.

3. **인프라** (Plane, Mattermost)는 배포 대상으로 사용되며 fork 하거나
   벤더링하지 않습니다. genasis는 공개 REST API로만 통신합니다.

4. **Rust 크레이트**는 `Cargo.toml`에 선언된 표준 의존성입니다.

## 라이선스 준수

모든 큐레이션된 에이전트는 MIT 라이선스 저장소에서만 소싱됩니다.
genasis 자체도 MIT 라이선스입니다. 오버레이 프로토콜, CLI 도구,
genasis 오리지널 콘텐츠는 모두 독립적 저작물입니다.

Plane (AGPL-3.0)과 Mattermost는 API를 통한 외부 서비스로 사용되며 —
genasis가 이들의 소스 코드를 포함하거나 배포하지 않습니다.
