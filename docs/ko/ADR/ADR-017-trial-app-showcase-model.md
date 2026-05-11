> English: [`../../ADR/ADR-017-trial-app-showcase-model.md`](../../ADR/ADR-017-trial-app-showcase-model.md)

# ADR-017: trial-app 쇼케이스 모델 — example PRD를 데모 경로로

## Status

Proposed (2026-05-11). ADR-013의 trial-app 계약에서 스크립트 데모
절반을 대체하며, ADR-016의 팀별 멀티테넌시 위에 신규 "쇼케이스"
기능을 얹는다.

## Context

ADR-016 이후 trial-app은 사용자별 샌드박스를 `team_token`으로
안정적으로 격리한다. 그러나 사용자 대면 경험에는 멀티테넌시 작업만으로
풀 수 없는 신뢰성 문제가 남아 있다.

1. **스크립트 "Try it" 탭이 실제 작업과 구분되지 않는다.** 현재
   trial-app은 세 탭을 노출한다 — `Try it`(스크립트 데모),
   `Live trial`(실제 `genasis init --trial` 샌드박스), `Apply`(가입
   폼). 스크립트 데모가 라이브 모드와 *동일한* 칸반·채팅 위젯을
   애니메이션하므로, 첫 방문자는 어느 쪽이 "에이전트가 실제로 하는
   것"인지, 어느 쪽이 "녹화된 애니메이션"인지 분간할 수 없다. 두 탭이
   관심을 두고 경쟁하고, 어느 쪽도 단독으로는 설득력이 없다.

2. **`genasis example prd`가 일반적인 작업관리 PRD를 출력한다.** CLI에
   포함된 기본 PRD는 모호한 "task status" 기능을 설명한다. 에이전트가
   이걸 구현하면 일반 칸반을 만드는데, 이는 trial-app 자체와 시각적으로
   똑같아 — 사용자는 "이게 에이전트 팀의 결과"와 "이게 trial-app의 빌트인
   위젯"을 구분할 방법이 없다. 데모가 자기 발을 잡아먹는다.

3. **trial 흐름에 종착점이 없다.** 에이전트가 스프린트를 끝내도 trial-app
   에서 바뀌는 게 없다. 사용자는 "팀이 이걸 만들었구나" 순간을 못 가진다 —
   샌드박스는 영원히 샌드박스로 남는다.

4. **호스팅 Plane/Mattermost 경로 링크가 잘못됐다.** 두 README 모두
   `trial.realstory.blog`(접두사 `mmplane-` 없음)를 가리키는데, 이는
   ADR-016 이후로 틀린 값이다. Apply 탭 라벨도 "Apply" / "신청하기" —
   일반 신청처럼 비치고, 실제로 받는 것 — 운영자 서버
   `mmplane-trial.realstory.blog`의 실제 Plane + Mattermost를 빌리는
   것 — 을 드러내지 못한다.

## Decision

네 가지 조정 변경을 한꺼번에 출하해 trial-app을 "데모 + 샌드박스 +
가입" 삼중창에서 단일 end-to-end 쇼케이스 내러티브로 전환한다.

### 1. 스크립트 데모를 제거한다. Live trial이 유일한 데모.

- `Try it` 탭과 `DemoBoard.tsx` 컴포넌트 삭제.
- `lib/i18n.ts`의 `demo.*` 키 삭제.
- `app/page.tsx` 탭 리졸버에서 `tab=demo` 제거.
- `e2e/demo.spec.ts` 삭제. 팀별 흐름의 Playwright 커버리지만 남긴다.
- 기본 랜딩 탭은 `live`(이전 `demo`).

사용자는 항상 자기 팀이 실제로 하는 일을 본다.

### 2. `genasis example prd`가 구체적이고 구별 가능한 앱을 만든다

기본 PRD는 두 언어로 ship — `crates/genasis-cli/templates/examples/`
아래 `prd.en.md`와 `prd.ko.md`. CLI가 `genasis.toml`의 `[i18n].active`를
읽어 선택한다 (없으면 `en`). 한국어로 `genasis init`을 돌린 사용자는
한국어 PRD를, 영어 프로젝트는 영어 PRD를 받는다.

설명되는 앱은 **"나는 Claude Code 전문가 / I Am a Claude Code
Expert"** — 자기진단 퀴즈:

- 모바일 폰 테두리 단일 페이지 앱 (trial-app의 칸반/채팅과 시각적으로
  명확히 다름).
- 시작 버튼 → 초·중·고급 Claude Code 지식을 다루는 문제 은행에서
  무작위 5문제 출제.
- 점수가 레벨을 결정 (초급 / 중급 / 고급 → Beginner / Intermediate /
  Advanced).
- 다시시작은 새 5문제 샘플을 뽑는다.

PRD를 읽는 에이전트는 명확하고 경계가 잡힌 결과물을 가진다 —
trial-app의 자체 위젯과 시각적으로 명백히 구분된다.

### 3. trial-app이 레퍼런스 퀴즈를 임베드, 팀별 상태가 게이팅

- 레퍼런스 퀴즈 구현은 trial-app 내부에 React 컴포넌트로 산다
  (`app/components/QuizApp.tsx` + 문제 은행
  `lib/quiz-bank.ts`). 호스팅 서비스에 임의의 사용자 코드를 업로드하는
  토끼굴을 피한다.
- 신규 슬라이딩 패널 — `ShowcasePanel` — 이 라이브 트라이얼 뷰의
  좌측 가장자리에 닻을 내린다. LiveBoard 헤더의 버튼이 토글한다.
  외부 클릭 / Esc로 닫힌다.
- sim schema가 `user_version = 2`에서 `3`으로 마이그레이션:
  `sim_teams`에 `app_status TEXT` 컬럼 추가 (`NULL` | `'building'` |
  `'complete'`). 팀의 `app_status = 'complete'`일 때만 패널 토글이
  활성. default 테넌시 fallback (`DEFAULT_TEAM_TOKEN`)은 항상
  `'complete'`로 취급 — `genasis init`을 한 번도 안 돌린 익명 방문자도
  쇼케이스를 본다(가치 제안 애니메이션).

결과: trial을 거친 모든 팀이 명확한 "우리가 이걸 만들었다" 보상을
가진다. 퀴즈가 에이전트가 뭔가 했음을 증명하는 가시적 결과물이
된다.

### 4. 명시적 완료 신호: `genasis trial publish`

에이전트가 PRD를 끝내면 사용자(또는 에이전트 쉘 hook)가 호출한다:

```bash
genasis trial publish
```

명령은 `genasis.toml`에서 `[trial].team_token`을 읽고, trial-app의
`POST /api/trial/team-app/status`에 `{ team_token, status:
"complete" }`를 POST한다. 그리고 사용자가 쇼케이스를 보려면 열 URL을
출력한다 (`<trial_url>/?tab=live&team=<token>`, 이제 패널이 활성).

신호를 Plane 티켓 상태에서 유추하지 않고 명시적으로 받는 이유:
(a) trial-app sim은 에이전트 자신이 feed하므로 "모든 티켓 done"이
독립적 확증을 못 주고, (b) 운영자는 모든 이슈가 닫히기 전에 부분
마일스톤을 publish하고 싶을 수 있다.

### 5. Apply 탭 리네이밍: "Borrow real env" / "실환경 빌리기"

탭과 폼 헤딩이 "Apply" / "신청하기"에서 **"Borrow real env"** /
**"실환경 빌리기"**로 바뀐다. 폼의 목적은 변하지 않음 — 운영자에게
요청을 제출하고 `mmplane-trial.realstory.blog`의 공유 인프라에서
실제 Plane + Mattermost 프로젝트의 자격증명을 받는다. 새 라벨은
실제로 받는 것을 말한다: 호스팅된 "애플리케이션"이 아니라 빌린
실제 환경.

`README.md` / `README.ko.md`의 "Trial Server" 링크는
`trial.realstory.blog`에서
`https://mmplane-trial.realstory.blog/?tab=signup`으로 수정 —
클릭하면 요청 폼이 바로 뜬다. docs/PRD/i18n 문자열에 남아 있는
다른 `trial.realstory.blog` 참조도 같은 commit에서 모두
`mmplane-trial.realstory.blog`로 sweep.

## Consequences

**쉬워지는 것**:
- 한 화면이 전 이야기를 한다: 에이전트가 만들고, 칸반·채팅이 일하는
  과정을 보여주고, 슬라이딩 패널이 완성된 앱을 드러낸다. "어느 탭이
  진짜야?" 혼란 끝.
- 레퍼런스 PRD가 이제 trial-app과 시각적으로 구분되므로, 에이전트
  팀의 결과물이 팀의 작품으로 인지된다.
- `genasis example prd`의 한국어 / 영어 분기가 다른 모든 생성 파일
  (에이전트 프롬프트, GENASIS.md, 슬래시 명령)이 쓰는 `[i18n].active`
  관례와 일치 — 놀라움이 줄어든다.
- `Apply` → `실환경 빌리기` 리네이밍이 폼 제출 시 실제로 받는 것을
  명확히 한다.

**어려워지는 것**:
- sim schema 마이그레이션 한 번 더 (V2 → V3). ADR-016의 V1 → V2와
  같은 idempotent 패턴 — 단일 nullable 컬럼 추가.
- trial-app이 임베디드 퀴즈 구현을 짊어진다. 정전 "Claude Code 전문성"
  문제 세트가 drift하면 trial-app을 재배포해야 에이전트가 실제로
  만들 결과와 동기. 수용 가능 — 문제는 거의 안 바뀌고, 월간 카탈로그
  리프레시면 충분.

**못 하게 되는 것**:
- 자유 형식의 임의 사용자 앱 업로드. 결정은 고정 레퍼런스 구현을
  ship하는 것이지 에이전트가 만든 코드를 받는 것이 아니다. 실제
  에이전트 결과물을 어딘가에 호스팅하고 싶은 사용자는
  `실환경 빌리기` 경로로 서비스된다 (ADR-017 §5).
- 스크립트 데모 경로는 영영 사라진다. 사용자 테스트에서 라이브
  모드가 첫인상으로 너무 압도적인 것으로 드러나면 ADR-018이 부활시킬
  수 있지만, 현재 증거(§Context.1의 신뢰성 문제)는 반대 방향을 가리킨다.

## Verification

- Unit 테스트:
  - `crates/genasis-core/src/config.rs`: 기존 i18n 라운드트립 통과
    (schema 변경 없음).
  - `crates/genasis-cli/src/cmd_example.rs`: 신규 테스트 —
    `prd_emits_korean_when_active_lang_ko` /
    `prd_emits_english_when_active_lang_en`.
  - `crates/genasis-cli/src/cmd_trial_publish.rs`: 신규 테스트 —
    POST body 모양, target URL env-var 오버라이드.
- Trial-app:
  - 마이그레이션 테스트: V2 fixture → V3가 `sim_teams`에
    `app_status` 컬럼 부여, 기존 행은 `NULL`.
  - Status 라우트: 유효한 토큰으로 POST 시 `app_status = 'complete'`
    설정. GET이 현재 상태 반환. 반복 POST는 idempotent.
  - 퀴즈: 문제 은행 ≥ 15개, 점수 → 레벨 매핑 결정적 (0–1 → 초급,
    2–3 → 중급, 4–5 → 고급).
- Playwright e2e:
  - `demo.spec.ts` 삭제.
  - 신규 `showcase.spec.ts`: 패널 토글 동작, 외부 클릭 닫기,
    `'building'` 상태일 때 게이팅.

## References

- ADR-013 (trial-bridge config wiring) — 거기서 도입된 스크립트 데모는
  이 ADR §1에서 제거된다.
- ADR-014 (human roster) — `실환경 빌리기` 폼은 빌린 프로젝트의
  `[[humans]]`를 계속 채운다.
- ADR-016 (identifier alignment + multi-tenancy) — 이 ADR이 그 위에
  쌓인다. team_token이 여전히 테넌시 키이고, 이제는 `sim_teams.app_status`
  도 스코프한다.
- 구현:
  `crates/genasis-cli/src/{cmd_example,cmd_trial_publish}.rs`,
  `crates/genasis-cli/templates/examples/prd.{en,ko}.md`,
  `agents-pool/trial-app/{db,app/components,app/api/trial/team-app,lib/quiz-bank,e2e}/*`,
  `README.{md,ko.md}`.
