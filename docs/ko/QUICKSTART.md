> English: [../QUICKSTART.md](../QUICKSTART.md)

# 빠른 시작 — 전체 워크스루

genasis 설치부터 팀 구성, 첫 스프린트까지 — 에이전트가 Plane과
Mattermost에서 사람과 협업하기까지의 전 과정을 안내합니다.

## 사전 요구

- **Claude Code** 설치 + 인증 완료
- **Plane** 인스턴스 (자체 호스팅 또는 [실환경 빌리기](https://mmplane-trial.realstory.blog/?tab=signup))
- **Mattermost** 인스턴스 (자체 호스팅 또는 [실환경 빌리기](https://mmplane-trial.realstory.blog/?tab=signup))
- (선택) 소스 빌드 시 Rust 툴체인

## 1단계: genasis 설치

```bash
curl -fsSL https://raw.githubusercontent.com/claude-genasis/genasis/main/install.sh | sh
```

설치 프로그램이 하는 일:
- 최신 genasis 바이너리 다운로드
- 에이전트 카탈로그 fetch (492개 큐레이션된 에이전트)
- 선호 언어 선택 (영어/한국어)

## 2단계: 에이전트 선택 + 설치

```bash
genasis agents browse
```

interactive TUI가 카테고리를 보여줍니다. 필요한 에이전트를 선택하세요.
또는 프리셋 사용:

```bash
genasis agents install --preset web-app
# 설치됨: pm, architect, frontend-developer, backend-developer,
#         code-reviewer, qa-tester, security-reviewer, planner, designer
```

## 3단계: Plane + Mattermost 연결

`genasis.toml` 생성 (또는 `genasis init`이 자동 생성):

```toml
[project]
name = "my-project"

[plane]
url = "https://plane.yourdomain.com"
workspace_slug = "my-workspace"

[mattermost]
url = "https://mm.yourdomain.com"
```

환경 변수 설정 (`.env.agents`):

```bash
PLANE_API_KEY=your-plane-api-token
MM_ADMIN_TOKEN=your-mm-admin-token
MM_TEAM_ID=your-team-id
```

> **빌린 환경 사용 시?** [mmplane-trial.realstory.blog](https://mmplane-trial.realstory.blog/?tab=signup)
> 에서 받은 접속 정보를 붙여넣으세요.

## 4단계: 초기화

```bash
genasis init
```

수행 내용:
- Plane + Mattermost 연결 확인 (ping)
- Plane 프로젝트 + 라벨 생성
- Mattermost `#scrum-{project}` 채널 생성
- 각 에이전트 파일에 오버레이 프로토콜 주입

## 5단계: 검증

```bash
genasis doctor
```

모든 항목이 ✓ 로 표시되어야 합니다.

## 6단계: 작업 시작

프로젝트에서 Claude Code를 실행하세요. 에이전트가 준비되어 있습니다:

```
> /sprint-start
> /install-agent mobile     (나중에 에이전트 추가 필요 시)
```

에이전트가 Plane 티켓을 가져가고, Mattermost에 게시하고,
라이프사이클(Todo → In Progress → In Review → Done)을 진행하며,
여러분이 이미 사용하는 같은 채널에서 조율합니다.

## 다음 단계

- [디자인 교체 가이드](DESIGN-SWAP-GUIDE.md) — 디자인 시스템 커스터마이징
- [서버 설치](../../servers/README.md) — Plane + Mattermost 자체 호스팅
- [에이전트 마켓플레이스](AGENTS-MARKETPLACE.md) — 사용 가능한 모든 에이전트 탐색
- [모니터 TUI](../MONITOR.md) — 스프린트/토큰/에이전트/배포 대시보드
