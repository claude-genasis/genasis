> English: [../TUTORIAL.md](../TUTORIAL.md)

# Genasis 튜토리얼

## 빠른 체험 — 5단계로 완전 마스터

Step 1~5만 완료하면 에이전트 팀이 실제 스프린트를 돌리는 것까지 체험합니다.
Genasis가 무엇을 하는지 이해하는 가장 빠른 방법입니다.

### Step 1 — 설치

```bash
curl -fsSL https://raw.githubusercontent.com/claude-genasis/genasis/main/install.sh | sh
```

### Step 2 — Trial 모드로 초기화

```bash
mkdir my-first-project && cd my-first-project
genasis init --trial
```

빈 프로젝트를 생성하고 기본 에이전트 팀을 설치한 뒤, 브라우저에서
**Trial 데모 앱**을 엽니다. 데모에서는 에이전트가 티켓을 가져가고, 채팅에
글을 올리고, 칸반 카드를 이동하는 스프린트 시뮬레이션이 자동으로 재생됩니다.

데모 앱에서 **Trial Plane + Mattermost 환경을 신청**할 수도 있습니다 —
서버 설치 없이 바로 시작.

### Step 3 — 샘플 PRD 생성

```bash
genasis example prd
```

에이전트 팀이 바로 작업할 수 있는 `PRD.md`를 생성합니다.
인증, CRUD, 반응형 UI를 갖춘 간단한 todo-app PRD입니다.

### Step 4 — 에이전트 팀 가동

```bash
genasis init
```

PM 에이전트가 `PRD.md`를 읽고 Plane 티켓으로 분해, 역할을 배정하고
팀이 작업을 시작합니다. Plane과 Mattermost에서 진행을 볼 수 있습니다.

### Step 5 — 스프린트 모니터링

```bash
genasis monitor
```

Ratatui TUI 대시보드를 열어 실시간으로 확인합니다: 어떤 에이전트가 무슨
작업을 하고 있는지, 토큰 사용량, 티켓 상태, 채팅 활동.

---

**기본을 마스터했습니다.** 에이전트 팀이 PRD에서 작동하는 코드까지
스프린트를 돌렸습니다. 아래는 더 탐색하고 싶을 때 시도할 수 있는
선택적 연습입니다.

---

## 더 해보기

위에서 만든 프로젝트를 기반으로 진행합니다. 각 연습은 genasis의 다른
기능을 보여줍니다.

### Exercise 6 — PRD2로 기능 확장

```bash
genasis example prd2
```

로그인, 관리자 백오피스, 사용자 관리 등 추가 기능이 담긴 `PRD2.md`를
생성합니다. 에이전트가 PRD2를 읽고 새 티켓을 만들어 기존 코드를 확장합니다.
에이전트 팀으로 점진적 개발하는 과정을 체험합니다.

### Exercise 7 — 디자인 시스템 교체

```bash
genasis example design
genasis design swap --from docs/design-system.md
```

새로운 디자인 토큰(색상, 타이포그래피, 간격)이 담긴 `docs/design-system.md`를
생성합니다. swap을 실행하면 영향받는 모든 UI 영역에 대한 Plane 이슈가
자동 생성됩니다. 프론트엔드 에이전트가 이 이슈를 자동으로 가져갑니다.

### Exercise 8 — 전문 에이전트 추가

```bash
genasis agents browse
```

에이전트 카탈로그를 탐색하고 전문가를 추가합니다:
- `seo-specialist` — SEO 감사, 메타 태그 생성
- `sre-engineer` — 모니터링, 헬스체크 설정
- `ios-expert` — 모바일 앱 지원 추가

### Exercise 9 — 완전히 새로운 프로젝트 시작

```bash
mkdir another-project && cd another-project
genasis init --bootstrap
```

이번에는 `--trial` 없이 — 직접 운영하는 Plane과 Mattermost에 연결합니다
(자체 호스팅 또는 trial 서버).

### Exercise 10 — 기존 프로젝트에 붙이기

```bash
cd /path/to/existing-project
genasis attach
```

기존 `.claude/agents/*.md` 파일을 감지하고 비파괴적으로 Plane/Mattermost
연동을 overlay합니다. marker fence 밖의 에이전트 정의는 그대로 유지됩니다.
