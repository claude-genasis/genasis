# PRD: 나는 Claude Code 전문가

> 상태: Draft
> Owner: TBD
> 생성: `genasis example prd` (ADR-017)
> 대상: trial-app의 좌측 슬라이딩 쇼케이스 패널 안에 렌더되는 웹 앱

## 1. 개요

사용자의 Claude Code 지식을 **초급 / 중급 / 고급**으로 등급화하는
단일 페이지 자가 진단 퀴즈. 모바일 폰 프레임 (둥근 chrome, 좁은 고정
너비) 안에 렌더되어 — trial-app 자체의 칸반·채팅 위젯과 시각적으로
명확히 구분된다. 사용자는 한눈에 "이게 에이전트 팀이 만든 거구나"를
알 수 있다.

에이전트가 빌드를 끝내면 `genasis trial publish`를 호출해 팀의
`app_status`를 trial-app에서 `complete`로 전환한다. 이 시점부터
팀의 슬라이딩 쇼케이스 패널이 이 퀴즈를 드러낸다.

## 2. 사용자 스토리

> Claude Code를 처음 탐색하는 개발자로서, 빠르게 내 위치를 가늠하고
> 싶다 — `/help`만 막 배운 초보인지, 아니면 직접 subagent와 hook을
> 쓰고 있는 사람인지. 5문제면 자리매김에 충분하고, 새 문제로
> 다시 시작하면 내가 성장하고 있는지 알 수 있다.

## 3. 요구사항

### 3.1 레이아웃 — 모바일 폰 프레임

- 앱은 "폰 새시" 컨테이너 안에 렌더된다:
  - 너비 데스크탑에서 약 360px 고정, 모바일에서는 풀 폭.
  - 둥근 모서리 (≥ 32px), 어두운 베젤, 은은한 그림자.
  - 부모 쇼케이스 패널 안에서 수평 중앙 정렬.
- 새시 내부에서 세 화면이 제자리에서 교체된다 — **시작**,
  **문제**, **결과**.

### 3.2 시작 화면

- 앱 제목: **"나는 Claude Code 전문가"** (현재 CLI 로케일).
- 한 줄 설명: "5개의 문제를 풉니다. 당신의 레벨을 알려드릴게요."
- 단일 primary 버튼: **"시작"** — 탭하면 첫 문제로 이동.

### 3.3 문제 화면

- 상단에 진행 표시 (예: "2 / 5").
- 그 아래 문제 텍스트.
- 답안 4개를 풀-너비 탭 영역으로 (라디오 / 버튼 하이브리드 —
  하나만 선택 가능).
- 답 선택 시 "다음" 버튼 활성화.
- "다음" 탭 시 다음 문제로. 5번째에서는 결과 화면으로.
- 문항별 정오 여부는 기록하지만 결과 전까지 보여주지 않음.

### 3.4 결과 화면

- 점수: **N / 5 정답**.
- 점수에 따른 레벨 라벨:
  - **0–1 정답 → 초급**
  - **2–3 정답 → 중급**
  - **4–5 정답 → 고급**
- 레벨에 맞는 한 단락 격려 메시지.
- **"다시 시작"** 버튼 — 문제 은행(§3.5)에서 5문제 새 샘플을
  뽑고 시작 화면으로 돌아간다.

### 3.5 문제 은행

난이도(`beginner` / `intermediate` / `advanced`)로 태깅된 최소
15문제의 정적 은행. `시작`과 `다시 시작` 시 세 레벨에 걸치도록
가중된 5문제를 뽑는다 (예: 초급 2 + 중급 2 + 고급 1) — 단일 세션이
레벨을 판별할 수 있도록. 정확한 샘플링 분포는 구현 세부사항이고,
제약은 "한 세션에서 같은 문제 두 번 안 나옴" + "연속 두 세션에서
≥ 70% 문제는 다름".

초급 / 중급 / 고급 예시 (은행이 이를 두루 다뤄야 함):

- **초급**: `/clear`는 무엇을 하는가? Claude Code 설정은 어느
  디렉터리에 있는가? 세션 중간에 활성 모델은 어떻게 전환하는가?
- **중급**: 슬래시 명령 파일은 어떤 구조인가? hook과 skill은 어떻게
  다른가? `.claude/agents/` 맥락에서 marker fence란 무엇인가?
- **고급**: subagent vs skill을 언제 작성하는가? 프롬프트 캐싱이
  Anthropic SDK의 batch API와 어떻게 상호작용하는가? Plane / Mattermost
  프로바이더 트레잇이 trial flavor에 강제하는 불변 조건은?

(에이전트는 난이도 의도를 충족하고 일반 프로그래밍이 아닌 Claude
Code를 다룬다면 자기 문제를 생성해도 좋다.)

## 4. API / 영속성

없음. 퀴즈는 완전 클라이언트 사이드. 문제 선택은 세션별 시드에
대해 결정적 (시드 자체는 세션마다 무작위지만, 향후 "결과 공유"
기능을 원하면 재현 가능). 로그인 없음, 저장 없음, 텔레메트리 없음.

## 5. trial-app 통합 (ADR-020 push-to-operator 모델)

에이전트 팀은 **프로젝트 루트의 `app/` 하위에 standalone React
앱** 을 빌드한 뒤, `genasis push` 로 정적 자산을 운영자의 trial-app
에 업로드한다. 운영자가 `/dev/<team_token>/` 경로로 직접 serve 하므로
사용자 머신이 꺼져도 데모 URL 이 살아있고, 다른 디바이스에서도 같은
URL 로 접근 가능하다.

요구 layout (사용자가 `mkdir my-team && cd my-team && genasis init
--trial` 로 만든 프로젝트 루트 기준 — Claude project 표준):

```
my-team/                       ← cwd / project root
├── .claude/agents/            ← genasis 가 만든 에이전트 정의
├── .genasis/                  ← daemon state
├── genasis.toml               ← provider + trial config
├── PRD.md                     ← 이 파일
└── app/                       ← frontend 가 scaffold 하는 React 앱
    ├── package.json           # vite + react-ts 의존성
    ├── vite.config.ts
    ├── index.html
    ├── src/
    │   ├── main.tsx
    │   ├── App.tsx            # QuizApp 마운트
    │   ├── components/
    │   │   └── QuizApp.tsx    # §3 + §4 quiz 컴포넌트
    │   └── lib/
    │       └── quiz-bank.ts   # 문제 은행
    └── dist/                  ← `npm run build` 산출물 (genasis push 가 업로드)
```

워크플로우 (PM 이 순서대로 dispatch):

1. **frontend** Task — `npm create vite@latest app -- --template
   react-ts --yes` + `(cd app && npm install)` + §3-§4 따라
   `app/src/components/QuizApp.tsx` / `app/src/lib/quiz-bank.ts`
   작성 + `(cd app && npx tsc --noEmit)` typecheck.
2. **devops** Task — `(cd app && npm run build)` 로 `app/dist/`
   생성 + `[ -f app/dist/index.html ]` 확인 + `genasis push --dir
   ./app/dist` 호출. push 가 trial-app 에 정적 자산을 업로드하면
   `sim_teams.showcase_pushed_at` 이 갱신되고 showcase panel 의
   placeholder ("준비중") 가 즉시 사용자 빌드 결과 iframe 으로 바뀐다.
3. **app_status** 는 publish 시 자동으로 `complete` 로 전환되므로
   별도 호출 불필요.

cwd 자체 (`.` ) 를 vite root 로 잡지 말 것 — `genasis.toml` / `.genasis/`
가 빌드 산출물에 섞이고 사용자 layout 이 깨진다. 항상 `app/` 하위로.

이전 v0.5.x — `announce_dev_server_url` + localhost iframe — 흐름은
ADR-020 으로 폐기됐다. 사용자 localhost 가 dev server 를 띄울 필요가
없으므로 노트북을 닫아도 데모가 유지된다.

## 6. 수용 기준

- 시작 화면이 모바일 새시 안에 활성 로케일의 올바른 제목으로 렌더.
- 한 세션 전체 (시작 → 5문제 → 결과) 가 답을 잃지 않고 완주.
- 레벨 매핑이 §3.4와 정확히 일치 (0–1 / 2–3 / 4–5).
- 다시 시작이 연속 10회 중 ≥ 7회 다른 샘플을 산출 (통계적,
  엄격 아님).
- 다시 시작이 시작 화면으로 돌아간다 (바로 문제 화면으로 가지 않음).
- 쇼케이스 패널이 LiveBoard 버튼으로 토글되고, 외부 클릭이나 Esc로
  닫힌다.
- `app_status`가 `NULL`이거나 `'building'`인 팀에게는 패널 토글 버튼이
  비활성 (또는 "에이전트가 아직 빌드 중…" 힌트 표시).

## 7. Non-goals (v1)

- 사용자별 점수 히스토리 / 리더보드.
- trial-app 내부의 문제 편집 UI.
- 결과를 SNS에 공유.
- 문제 은행 자체의 현지화 (v1에서는 은행이 영어 전용이어도 OK,
  UI chrome — 버튼, 라벨, 레벨명 — 만 활성 로케일을 따른다).
- 에이전트가 업로드한 코드 수락. 퀴즈 구현은 고정.
