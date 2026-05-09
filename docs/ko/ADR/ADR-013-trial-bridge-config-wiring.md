> English: [`../../ADR/ADR-013-trial-bridge-config-wiring.md`](../../ADR/ADR-013-trial-bridge-config-wiring.md)

# ADR-013: Trial 브릿지 설정의 단일 진실 원천(SSOT)을 `[trial]` 섹션으로

## Status

Accepted (2026-05-10). 이전 ralph/trial-webapp 브랜치에서 도입됐던 trial-app
연동 코드가 `[trial]` 섹션을 정의만 해두고 실제 라우팅에는 사용하지 않던
상태를 정리.

## Context

`genasis init --trial`은 trial-app(Next.js)을 띄워 Plane + Mattermost 없이도
에이전트 워크플로우를 체험할 수 있게 해준다. 초기 구현은 다음 두 곳에 트라이얼
설정을 흩어 두었다:

1. `[plane].url`, `[plane].flavor = "trial"`, `[mattermost].url`,
   `[mattermost].flavor = "trial"` — 실제 라우팅에 사용됨.
2. `[trial].enabled`, `[trial].url`, `[trial].shared_secret` — 정의는
   되지만 Rust 코드 어디에서도 읽지 않음. 사실상 죽은 설정.

결과적으로 다음 시나리오가 실패하거나 사일런트 오작동을 했다:

- `[trial].enabled = false` 인데 `flavor = "trial"` 이면 trial-app으로 호출이
  계속 가서 사용자가 의도와 다르게 트라이얼 모드를 끌 수 없음.
- `[trial].url`을 변경해도 무시됨. 사용자는 `[mattermost].url` /
  `[plane].url` 두 곳을 함께 바꿔야 함.
- `[trial].shared_secret`을 채워도 무시되고 `MM_ADMIN_TOKEN` /
  `PLANE_API_KEY` 환경변수가 시크릿 자리에 들어감. 트라이얼 모드인데
  Mattermost admin token을 강제로 요구하는 어색한 UX.

## Decision

**`[trial]` 섹션이 trial 라우팅의 단일 진실 원천이다.** 결정 사항:

1. **factory 시그니처에 `Option<&TrialConfig>` 추가**
   - `mattermost::factory::build()`, `plane::factory::build()`가 `flavor =
     Trial`일 때 `[trial].url`과 `[trial].shared_secret`을 사용한다.
   - 다른 flavor에서는 인자를 무시한다.
   - `flavor = Trial`인데 `trial = None`이거나 `enabled = false`면 명시적
     `Error::Config`로 실패시킨다.

2. **`[plane].url` / `[mattermost].url`은 trial 모드에서 무시되는 placeholder**
   - `genasis init --trial`이 생성하는 `genasis.toml` 템플릿에 명시적인
     `# Ignored when flavor = "trial"` 주석을 추가한다.
   - 향후 trial → real 전환 시 채울 자리만 마련해 두는 의미로 유지.

3. **`Config::load()`에 cross-section 검증 추가**
   - `validate_trial()` 메서드: `flavor = "trial"` 이면 `[trial]` 존재 +
     `enabled = true`를 요구. 부적합 시 `Error::Config`.
   - 사용자가 부분 수정한 잘못된 설정을 런타임 HTTP 실패가 아닌 로드
     단계에서 즉시 잡는다.

4. **trial 모드에서 환경변수 요구 완화**
   - `cmd_init` / `cmd_mm` / `cmd_plane` / `cmd_humans`가 `flavor = Trial`
     일 때 `MM_ADMIN_TOKEN` / `PLANE_API_KEY` 환경변수 요구를 건너뛴다.
   - trial 라우팅은 `[trial].shared_secret`이 시크릿이고 별도 admin token
     개념이 없다.

## Consequences

**Easier**:
- 트라이얼 사용자는 환경변수를 하나도 export 하지 않고 `genasis init
  --trial`만으로 전체 워크플로우를 체험할 수 있다.
- `[trial].url`을 변경하면 즉시 라우팅이 바뀐다 — 향후 호스팅된 trial-app을
  포인팅하거나 포트를 옮기는 시나리오가 자연스럽다.
- 잘못 손댄 config가 빠르고 명확한 에러 메시지로 막힌다.

**Harder**:
- factory `build()` 시그니처가 1개 인자 늘었다. 외부에서 호출하는 곳이
  네 곳뿐이라 영향이 작지만, 향후 SDK / 외부 통합이 생기면 마이너 버전 범프
  대상이다.

**Foreclosed**:
- "trial 모드인데 실제 Mattermost를 같이 쓰는" 하이브리드는 불가. 한 config
  내에서 Plane은 real, Mattermost는 trial인 식의 절반 트라이얼은 가능
  (각 섹션의 flavor가 독립).

## Verification

- 단위 테스트: `crates/genasis-providers/src/{mattermost,plane}/factory.rs`의
  `build_trial_*` 시리즈, `crates/genasis-core/src/config.rs`의
  `validate_trial_*` 시리즈.
- E2E: `crates/genasis-providers/tests/trial_factory_e2e.rs` —
  `#[ignore]`로 표시되며 trial-app이 떠 있을 때만 실행
  (`TRIAL_BASE`, `TRIAL_SECRET` 환경변수 필요).

## References

- 구현 변경: `crates/genasis-providers/src/{mattermost,plane}/factory.rs`,
  `crates/genasis-core/src/config.rs`,
  `crates/genasis-cli/src/{cmd_init,cmd_mm,cmd_plane,cmd_humans}.rs`.
- 관련 ADR: ADR-005 (Flavor system) — Trial은 그 시스템에 추가된 네 번째
  flavor.
- 트라이얼 앱 자체: `trial-app/` (Next.js, `[trial].url`이 가리키는 대상).
