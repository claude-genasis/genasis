# 테스트 가이드

> English: [`../TESTING.md`](../TESTING.md)

Genasis 변경을 push 하기 전에 안전성을 검증하는 방법. 10초짜리
`cargo fmt` 부터 풀 Plane + Mattermost docker 스택을 띄우는 10분짜리
실서버 스모크까지, 프로젝트가 제공하는 모든 테스트 계층을 다루며 어떤
변경에 어느 계층을 돌려야 하는지 안내합니다.

---

## 한 줄 요약

| 변경 대상 | push 전 실행 |
|---|---|
| Rust 코드 (모든 crate) | `cargo fmt --all && cargo test --workspace` |
| `crates/genasis-providers/` | + `scripts/nightly-e2e.sh` |
| `crates/genasis-cli/` (init / attach 라이프사이클) | + `scripts/nightly-e2e.sh` |
| `servers/docker-compose.yml` | + `scripts/nightly-e2e.sh` |
| trial-app 프론트엔드 | _v0.6+ 에서 비공개 [agents-pool/trial-app/](https://github.com/claude-genasis/agents-pool/tree/main/trial-app) 로 이동 — 별도 운영. `genasis init --trial` 은 https://genasis-trial.realstory.blog 의 운영자-호스팅 인스턴스를 사용._ |
| `docs/ko/` 외부 `*.md` | `scripts/check-i18n-drift.sh --warn` |
| `crates/genasis-i18n/locales/*.yml` | `scripts/i18n-extract-keys.sh` |
| `tests/golden/**` 픽스처 | `BLESS=1 cargo test -p genasis-overlay` 후 diff 확인 |

CI 가 돌리는 것과 동일한 단일 명령:

```bash
cargo fmt --all -- --check && \
  cargo clippy --workspace --all-targets && \
  cargo test --workspace --all-targets
```

---

## 테스트 계층 개요

Genasis 는 10개 계층의 회귀 검증을 제공합니다. CI 는 매 push 마다
L1–L3 + L8 을 자동 수행합니다. 나머지 계층은 로컬에서 — 무료 CI
러너에서 돌리기엔 너무 무겁거나 (L9 는 전체 Plane 스택 필요), 개발자
환경의 자격 증명이 필요하기 (L7) 때문에 — 실행됩니다.

| 계층 | 내용 | 실행 시점 | 소요 | CI? |
|---|---|---|---|---|
| **L1** fmt + lint | `cargo fmt --check`, `cargo clippy` | 매 commit | ~10 s | ✅ `ci.yml :: test` |
| **L2** 단위 + 통합 | `cargo test --workspace --all-targets` | 매 PR | ~60 s | ✅ `ci.yml :: test` |
| **L3** i18n drift | `check-i18n-drift.sh`, `i18n-extract-keys.sh` | docs/template 변경 시 | ~5 s | ✅ `ci.yml :: lint-i18n` (PR 에서 warn-only) |
| ~~L4~~ trial-app 빌드 | _v0.6+ 에서 agents-pool/trial-app 으로 이동 — 아래 "L4/L5 retire" 노트 참조._ | — | — | — |
| ~~L5~~ trial-app E2E | _L4 와 동일 — retire 노트 참조._ | — | — | — |
| **L6** README parity | `cargo test -p genasis-e2e` | CLI 표면 변경 시 | ~30 s | ✅ L2 에 포함 |
| **L7** 실서버 | `scripts/e2e-test.sh` | 릴리즈 전 | ~10 분 | ❌ |
| **L8** 커버리지 | `cargo llvm-cov` | 정보 제공 | ~80 s | ✅ `ci.yml :: coverage` → Codecov |
| **L9** 실서버 스모크 | `scripts/nightly-e2e.sh` | provider / init / servers 변경 시 | ~10 분 | ❌ — 의도적 로컬 전용 |
| **L10** 소스 빌드 | `./build.sh` | 릴리즈 검증 | ~3 분 | (릴리즈 파이프라인) |

---

## L1 — `cargo fmt --check` + `cargo clippy`

**검출 대상**: 스타일 drift, 비-관용적 Rust, 흔한 버그 패턴.

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets
```

**실패 예시**:

```
Diff in crates/genasis-cli/src/cmd_init.rs:195
```

**복구**: `cargo fmt --all` (`--check` 빼고) 후 다시 stage. push 전에
둘 다 통과해야 합니다 — `ci.yml :: test` 가 어떤 테스트보다 먼저 이
둘을 돌리므로, fmt 실패는 CI 전체를 단락시킵니다.

---

## L2 — `cargo test --workspace --all-targets`

**검출 대상**: 10 개 workspace crate 어디든 회귀 + `tests/e2e`
라이프사이클 하네스 + golden fixture drift.

```bash
cargo test --workspace --all-targets
# 또는 더 빠르게, lib + integration 만 돌리려면 --all-targets 생략
cargo test --workspace
```

현재 floor: **10 crate 에 걸친 245 개 테스트**.

**흔한 실패**:
- 스키마 테스트 실패 → `crates/genasis-core/src/config.rs` 와 해당
  test module 확인.
- Golden fixture drift → 아래 [Golden 픽스처](#golden-fixture-추가)
  참고.
- `tests/e2e/` 의 통합 테스트 실패 → `cargo test -p genasis-e2e
  <name> -- --nocapture` 로 단일 테스트 풀 출력 확인.

---

## L3 — i18n drift (en ↔ ko 미러, en.yml ↔ ko.yml)

Genasis 는 엄격한 이중언어 프로젝트입니다 (CLAUDE.md §Bilingual Mirror
Policy). 두 스크립트가 parity 를 강제합니다:

```bash
scripts/check-i18n-drift.sh --warn   # drift 출력, warn 모드는 exit 0
scripts/check-i18n-drift.sh          # drift 출력, 있으면 exit 1
scripts/i18n-extract-keys.sh         # en.yml ↔ ko.yml 키 parity 보장
```

CI 는 PR 에서 둘 다 `--warn` 모드로 돌리지만, **release-prep** 에서는
drift 가 hard-fail 합니다. drift 해결 방법:
1. 더 오래된 미러를 갱신 (권장), 또는
2. 의도적 비대칭이라면 `progress.{md,ko.md}` 에 기록.

영어 소스 파일에 한국어 거부: CI 는 `docs/ko/` 외부의 `*.md`,
`*.tera`, `*.rs`, `*.sh`, `*.yml` 에서 `[가–힯]` 을 grep 합니다. 영어
소스에 한글이 섞이면 CI 가 즉시 실패합니다.

---

## L4 + L5 — retired (v0.6+)

Trial-app 소스 코드가 v0.6 에서 비공개
[claude-genasis/agents-pool](https://github.com/claude-genasis/agents-pool/tree/main/trial-app)
으로 이동했습니다. genasis 본 리포는 더 이상 Next.js 앱, Playwright
suite, `trial-app/` 디렉터리를 ship 하지 않습니다.

운영자-호스팅 배포는 **https://genasis-trial.realstory.blog** 에서 동작
하며, 운영자 호스트의 `/work/genasis-trial/` 에서 Caddy 가 reverse-proxy
합니다 (해당 디렉터리 내부의 README 참조).

### 본 리포에서 L4/L5 의 대체
- **Genasis CLI 사용자**: `genasis init --trial` 이 호스팅 인스턴스를
  자동 가리킵니다. Genasis 평가에 trial-app 로컬 설치 불필요.
- **trial-app 기여자**: `claude-genasis/agents-pool` (비공개) 클론 후
  `trial-app/` 편집, 거기서 `npm run typecheck && npm run build &&
  npm run e2e` 실행. 운영자가 `deploy.sh` 로 `/work/genasis-trial/`
  에 변경사항 ship.

원래 계층 번호(L1, L2, L3, L6, L7, L8, L9, L10) 는 유지 — 이전 PR /
커밋 메시지의 prose 참조가 그대로 유효하도록. 새 계층 번호는 4 또는
5 를 재사용하지 않습니다.

---

## L6 — README parity (`tests/e2e/`)

```bash
cargo test -p genasis-e2e
```

**검출 대상**: `README.md` / `README.ko.md` 가 광고하는 모든 명령
(`genasis init`, `genasis attach`, `genasis doctor` 등) 에 대한 스모크
테스트가 `tests/e2e/tests/` 에 있습니다. CLI 명령을 추가하거나
플래그 동작을 바꿀 때 여기에 매칭되는 테스트를 추가해, README 와
코드가 절대 drift 하지 않도록 보장합니다.

이 crate 는 workspace 에 속하므로 `cargo test --workspace` (L2) 가
이미 돌립니다. 별도로 호출하는 건 단일 실패 명령을 디버깅할 때
유용합니다.

---

## L7 — 실서버 E2E (`scripts/e2e-test.sh`)

```bash
# Mock 모드 — 라이브 서비스 불필요 (~30 초)
scripts/e2e-test.sh --mock

# Quick 모드 — Plane 프로비저닝 건너뜀 (~5 분)
scripts/e2e-test.sh --quick

# 전체 라이프사이클을 라이브 서비스에 (~10 분)
scripts/e2e-test.sh
```

**라이브 모드 전제 조건**:
- `target/release/genasis` (먼저 `cargo build --release` 실행)
- `agents-pool/.env.e2e` (agents-pool submodule 의 비공개 파일)
  **또는** 다음 환경변수 export:
  - `PLANE_URL`, `PLANE_API_KEY`, `PLANE_WORKSPACE_SLUG`
  - `MM_URL`, `MM_ADMIN_TOKEN`, `MM_TEAM_ID`

결과는 `test-results/e2e-<timestamp>.log` 에 기록되고 최신 요약은
`test-results/e2e-latest.json` 에 저장됩니다. `--mock` 모드는 실
서비스 안 건드리고 스크립트 plumbing 검증할 때 유용합니다 — API
호출은 단락하지만 와이어링은 그대로 실행됩니다.

---

## L8 — 커버리지 (`cargo llvm-cov` → Codecov)

```bash
cargo install cargo-llvm-cov   # 첫 회만
cargo llvm-cov --workspace --lcov --output-path lcov.info
```

CI 는 `main` 으로 push 마다 `lcov.info` 를 Codecov 에 업로드합니다.
README 의
[![Coverage](https://img.shields.io/codecov/c/github/claude-genasis/genasis?branch=main&style=flat-square&logo=codecov)](https://codecov.io/gh/claude-genasis/genasis)
뱃지가 최신 결과를 반영합니다. 강제 floor 는 없으며 — 커버리지는
정보 제공이지 게이트가 아닙니다.

업로드 없이 로컬에서 확인:

```bash
cargo llvm-cov --workspace --html --open
```

---

## L9 — 실서버 스모크 (`scripts/nightly-e2e.sh`)

가장 무거운 계층. `servers/docker-compose.yml` 에서 **전체** Plane +
Mattermost docker 스택을 부팅하고 `genasis init --probe-only` 로
프로빙합니다. **의도적으로 로컬 전용** — GitHub 무료 러너 (7 GB RAM,
2 vCPU) 는 풀 Plane 스택을 안정적으로 호스팅할 수 없기 때문입니다
(postgres + redis + minio + rabbitmq + plane-api/web/space/admin/live
+ caddy + Mattermost + mm-postgres). 개발자 워크스테이션은 ~10 분에
끝내지만, GitHub 는 health 대기에서만 ~30 분 timeout 이 보통.

```bash
# 전체 (~10 분): 빌드 → 부팅 → cargo test (release) → probe → tear down
scripts/nightly-e2e.sh

# 더 빠르게 (~3 분): 부팅 + probe 만, cargo test 건너뜀
scripts/nightly-e2e.sh --skip-test

# 검사 모드: probe 후 docker 스택 그대로 유지
scripts/nightly-e2e.sh --no-down
# 끝나고 직접 정리:
( cd servers && docker compose down -v --remove-orphans )

# 인라인 도움말
scripts/nightly-e2e.sh --help
```

**전제 조건**:
- `docker` + `docker compose` v2 플러그인
- `cargo` (stable toolchain)
- 첫 docker 이미지 pull 시 ~10 GB 여유 디스크
- 실행 중 스택용 ~6 GB 가용 RAM

스크립트는 `EXIT` trap 으로 docker 스택을 항상 다시 내립니다 (단,
`--no-down` 을 주면 예외).

**실행 시점**:
- `crates/genasis-providers/` (실서버 flavor 로직) push 전
- `crates/genasis-cli/` (init / attach 라이프사이클) push 전
- `servers/docker-compose.yml` push 전
- 릴리즈 태그 cut 전

다른 변경 (프론트엔드, agent overlay, 문서) 은 `cargo test
--workspace` 만으로 충분합니다 — L9 는 과합니다.

---

## L10 — 소스 빌드 (`./build.sh`)

```bash
./build.sh
```

README 가 광고하는 install 경로를 검증 — Rust 미설치 시 설치, release
바이너리 빌드, `~/.local/bin/genasis` 로 복사. 릴리즈 cut 전에
fresh-machine install 이 여전히 작동하는지 확인할 때 유용.

---

## 시나리오별 테스트 매핑

| 변경 대상 | 최소 게이트 | 신뢰도 부스트 (선택) |
|---|---|---|
| genasis-core 의 Rust 함수 1개 | L1 + L2 | — |
| genasis-providers 어디든 | L1 + L2 | **L9** |
| `cmd_init.rs` / `cmd_attach.rs` | L1 + L2 + L6 | **L9** |
| 새 `[[humans]]` 류 config 필드 | L1 + L2 + golden 픽스처 | L9 |
| Agent overlay `.tera` 템플릿 | L2 + L3 | — |
| 새 `t!()` 키 | L2 + L3 (i18n 키 parity) | — |
| `docs/**` 만 (코드 무관) | L3 | — |
| trial-app 프론트엔드 (agents-pool 리포) | (agents-pool 의 trial-app L4/L5 거기서 실행) | — |
| `servers/docker-compose.yml` | L9 | — |
| GitHub workflow 파일 | (없음 — push 후 관찰) | — |
| 릴리즈 준비 | L1 + L2 + L6 + **L7** + **L9** + L10 | — |

---

## CI vs 로컬 — 분리 근거

CI (`.github/workflows/ci.yml`) 는 싸고 빠른 루프입니다. 매 push /
PR 마다 ~50 초 소요. 흔한 실패 클래스를 잡도록 튜닝됨: fmt, clippy,
깨진 테스트, 누락된 번역, 커버리지 하락. 의도적으로 무거운 계층은
**돌리지 않습니다** — 이유:

1. ~~**L4/L5 (trial-app)**~~ 더 이상 해당 안 됨 — trial-app 소스가 v0.6+
   에서 비공개 agents-pool 리포로 이동 ("L4 + L5 — retired" 섹션 참조).
   원본 주: 이전에는 CI 시간을 두 배로 늘리고 Playwright
   브라우저가 필요. 사소한 Rust PR 을 막을 가치 없음.
2. **L7 (실서버)** 는 repo secrets 에 진짜 Plane + Mattermost
   자격증명이 필요. 공개 repo 의 보안 위험.
3. **L9 (실서버 스모크)** 는 무료 러너에 안 맞음 — GitHub 로 옮겼더니
   green 결과 없이 ~30 분/run 만 태웠음.

이 계층들은 개발자 워크스테이션에서 돌립니다 — 그곳에서는:
- 자원이 충분 (16+ GB RAM 의 노트북, 빠른 디스크)
- 개발자가 올바른 자격증명 / 환경변수 보유
- 실패가 즉시 표면 (비동기 CI run 이 아니라)

이 분리는 의도적이며 CONTRIBUTING.md 에 codify 되어 있습니다.

---

## 새 테스트 추가

### 새 단위 테스트
코드 옆 `#[cfg(test)] mod tests` 블록에 추가. CI 의 L2 가 자동
픽업.

### tests/e2e 의 새 통합 테스트
`tests/e2e/tests/` 아래에 새 `#[test]` 함수 추가, `tests/e2e/src/lib.rs`
의 헬퍼 사용 (`harness` 모듈이 `spawn_genasis()` 등 노출). CI 의 L2
가 자동 픽업.

### 새 Golden 픽스처
1. 입력 프로젝트 상태로 `tests/golden/<name>/input/` 생성
2. 픽스처 설명용 `tests/golden/<name>/README.md` 작성
3. `BLESS=1 cargo test -p genasis-overlay` 로
   `tests/golden/<name>/expected/` 생성
4. diff 검토, 정상이면 commit, `BLESS=1` 없이 다시 돌려 픽스처가
   안정적인지 확인

### 새 Playwright spec
Trial-app Playwright spec 은 **비공개 agents-pool 리포** 의
`trial-app/e2e/` 아래에 있습니다 (이 리포 아님).
[agents-pool/trial-app](https://github.com/claude-genasis/agents-pool/tree/main/trial-app)
에서 테스트 러너 셋업을 참조하세요.

### 새 테스트 헬퍼
crate 간 공유 헬퍼는 `tests/e2e/src/lib.rs`. `tests/unit/` 디렉터리는
특정 crate 에 맞지 않는 작은 유틸리티 테스트용.

---

## 트러블슈팅

### `cargo test` 가 느림 (>5 분)
- 타깃 지정: `cargo test -p <crate-name>` 로 다른 9 crate 건너뜀.
- `cargo test --no-run` 으로 컴파일만 하고 단일 테스트만 반복:
  `cargo test --no-fail-fast <test_name>`.

### `nightly-e2e.sh` 에서 Mattermost health 체크 timeout
Mattermost 컨테이너는 cold boot 에 첫-실행 스키마 마이그레이션 (~2-3
분) 을 합니다. 스크립트는 총 15 분 대기. 계속 timeout 되면
`( cd servers && docker compose logs mm )` 으로 무엇이 막혔는지 확인.

### `npm run e2e` 가 "starting webServer" 에서 멈춤
3000 포트 충돌. 잔여 Next.js dev server 를 kill:

```bash
lsof -i :3000
kill <pid>
```

### Codecov 뱃지가 "unknown" 표시
`CODECOV_TOKEN` repo secret 누락 또는 최근 push 가 Rust 파일을
건드리지 않아 커버리지 재업로드 안 됨. ci.yml 수동 트리거 또는
cargo 관련 변경 push.

### `BLESS=1` 픽스처 재생성이 노이즈 발생
일부 픽스처는 timestamp 나 임시 경로를 포함. `genasis-overlay`
crate 의 golden 하네스는 비교 전 정규화하지만, 의도하지 않은 diff
가 보이면 `crates/genasis-overlay/tests/golden_helpers.rs` 의
정규화 룰 확인.

---

## 참고

- **[CONTRIBUTING.ko.md](../../CONTRIBUTING.ko.md)** — 같은 L1-L10
  표의 압축 버전 + 일반 기여 흐름
- **[docs/ko/PROVIDERS.md](PROVIDERS.md)** — flavor 시스템
  (upstream / agent-aware / trial), L9 가 왜 풀 스택이 필요한지
  이해에 유용
- **[docs/ko/MIGRATION-FROM-GENESIS.md](MIGRATION-FROM-GENESIS.md)**
  — legacy bash → Rust 테스트 parity 룰
- **[docs/ko/ADR/ADR-013-trial-bridge-config-wiring.md](ADR/ADR-013-trial-bridge-config-wiring.md)**
  — CI 의 `flavor=trial` 경로가 왜 L9 를 대체할 수 없는지
