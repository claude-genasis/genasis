# Genasis 기여 가이드

> English: [CONTRIBUTING.md](CONTRIBUTING.md)

이 가이드는 첫 PR 을 열기 전에 설치해야 하는 모든 도구와 셋업이 끝난 뒤의 작업 흐름을 안내합니다.

## 빠른 시작

```bash
# 1. 툴체인 (Rust + cargo)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal

# 2. 보조 CLI (옵션이지만 권장)
sudo apt-get install -y build-essential pkg-config libssl-dev   # Linux 빌드 의존성
brew install gh atlas duckdb                                    # macOS 개발 도구

# 3. Coverage 도구 (CI / coverage 작업 시에만)
. "$HOME/.cargo/env" && cargo install cargo-llvm-cov

# 4. 클론 + 빌드 + 테스트
git clone https://github.com/claude-genasis/genasis
cd genasis
cargo test --workspace --no-fail-fast
```

`cargo test` 가 "245+ passed" 를 출력하면 기여 준비 완료입니다.

---

## 로컬 regression 계층

Genasis 는 여러 계층의 regression 검증을 제공합니다. CI 는 매 푸시마다 L1–L3 + L8
을 자동 수행하고, 나머지는 로컬에서 직접 실행 가능합니다. 작업 범위에 맞는
계층을 선택해서 돌리세요.

| 계층 | 명령 | 검증 항목 | 소요 | CI 포함? |
|---|---|---|---|---|
| **L1** fmt + lint | `cargo fmt --all -- --check` · `cargo clippy --workspace --all-targets` | 스타일 + 린트 | ~10s | ✅ `ci.yml :: test` |
| **L2** 단위 + 통합 | `cargo test --workspace --all-targets` | 245+ Rust 테스트 (golden fixture 포함) | ~60s | ✅ `ci.yml :: test` |
| **L3** i18n drift | `scripts/check-i18n-drift.sh` · `scripts/i18n-extract-keys.sh` | EN↔KO 미러 + i18n 키 parity | ~5s | ✅ `ci.yml :: lint-i18n` |
| **L4** trial-app 빌드 | `cd trial-app && npm run typecheck && npm run build` | TypeScript + Next.js 15 빌드 | ~30s | ❌ |
| **L5** trial-app E2E | `cd trial-app && npx playwright test` | 23개 Playwright spec (M21) | ~5분 | ❌ |
| **L6** README parity E2E | `cargo test -p genasis-e2e` | README 의 모든 커맨드 (M19) | ~30s | ✅ L2 에 포함 |
| **L7** 라이브 서버 E2E | `scripts/e2e-test.sh [--mock\|--quick]` | 실제 Plane + Mattermost 풀 라이프사이클 | ~10분 | ❌ |
| **L8** 커버리지 | `cargo llvm-cov --workspace --lcov --output-path lcov.info` | 라인 커버리지 → Codecov | ~80s | ✅ `ci.yml :: coverage` |
| **L9** 야간 실서버 | `scripts/nightly-e2e.sh` (로컬 pre-push 게이트) | `servers/docker-compose.yml` 대상 L7 | ~10분 (로컬) | ❌ — 의도적으로 로컬 전용 (GitHub free runner 는 Plane 풀스택을 돌리기엔 너무 느림) |
| **L10** 소스 빌드 | `./build.sh` | release 바이너리 + `~/.local/bin` 설치 | ~3분 | (릴리즈 검증) |

**PR 푸시 전 빠른 경로**: `cargo fmt --all && cargo test --workspace` —
L1 + L2 + L6 한 번에 돌리며 `ci.yml :: test` 와 동일한 검증을 수행합니다.

전체 계층별 지침, 시나리오별 선택 가이드 ("X 를 바꿨을 때 무엇을
돌려야 하는가"), 트러블슈팅은 **[`docs/ko/TESTING.md`](docs/ko/TESTING.md)**
를 참조하세요.

---

## 각 prerequisite 가 필요한 이유

목록은 짧게 유지했고, 어떤 종류의 기여에 필요한지로 묶었습니다.

### 모든 코드 변경에 필수

| 도구 | 필요한 이유 | 설치 |
|---|---|---|
| **rustup** + **cargo** + **rustc** (stable 채널) | Genasis 는 10개 Rust crate 의 Cargo workspace 입니다. cargo 가 빌드/테스트/clippy/fmt 모두 구동합니다. `rust-toolchain.toml` 이 채널을 `stable` 로 고정해서, 저장소에 `cd` 하면 rustup 이 자동으로 올바른 컴파일러를 선택합니다. | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh -s -- -y --default-toolchain stable --profile minimal` |
| **rustfmt** + **clippy** 컴포넌트 | CI 가 `cargo fmt --all --check` 와 `cargo clippy --workspace --all-targets` 를 돌립니다. 둘 다 통과하지 못한 PR 은 자동으로 거부됩니다. 위 `--profile minimal` 설치는 `rust-toolchain.toml` 선언을 통해 둘을 함께 추가합니다. | (위 rustup 설치에 포함) |
| **git** | 소스 컨트롤. 모든 commit / PR 이 git 으로 진행됩니다. | `sudo apt-get install git` (Debian/Ubuntu) · `brew install git` (macOS) |
| **C 컴파일러 & OpenSSL 헤더** | 일부 transitive crate 의존성이 native 코드를 빌드합니다. Linux 는 `build-essential` (gcc + make) 와 `libssl-dev` 가 필요. macOS 는 Xcode CLT 로 충분. | Linux: `sudo apt-get install build-essential pkg-config libssl-dev` · macOS: `xcode-select --install` |

### 설치/릴리즈 경로에만 필요

| 도구 | 필요한 이유 | 설치 |
|---|---|---|
| **bash** ≥ 4 | `install.sh` 는 POSIX-적 bash 입니다. `--lang both` 거부 banner 의 heredoc 이 bash 의미를 요구합니다. | macOS: `brew install bash` (시스템 bash 3.2 는 prereq 매트릭스를 못 다룸) · Linux: 기본 포함 |
| **curl**, **tar**, **sha256sum** (macOS 는 `shasum -a 256`) | `install.sh` 가 release 바이너리를 다운로드하고, 체크섬 검증한 뒤 untar 합니다. | Linux/macOS 기본 이미지에 모두 포함 |
| **gh** (GitHub CLI) | `genasis init` 의 GitHub branch-protection 헬퍼와 `release-prep` 워크플로가 사용. 로컬 개발에는 옵션이지만, CI 의 GitHub-API 경로를 건드리는 PR 에는 필수. | `brew install gh` · `sudo apt-get install gh` (GitHub CLI repo 추가 후) |

### Monitor TUI + RTK 토큰 경제 런타임에만 필요

| 도구 | 필요한 이유 | 설치 |
|---|---|---|
| **rtk** (Rust Token Killer) | `genasis monitor` 의 Tokens 위젯이 `rtk gain --json` 으로 RTK 의 절감 카운터를 노출. rtk 없이도 Genasis 는 동작 — 위젯이 0 만 표시. | `cargo install rtk` (또는 rtk 프로젝트 README 참조) |
| **node** ≥ 18 + **npm** + (이후) **playwright** | `crates/genasis-cli/scripts/provision-plane-users.mjs` 가 Playwright 로 Plane 사용자 자동 프로비저닝하는 Node 서브프로세스. M4 (Plane user provisioner) 작업할 때만 관련. | `nvm install 18 && nvm use 18` · `npm install --prefix crates/genasis-cli/scripts` |
| **claude** (Claude Code CLI) | 빌드/테스트에는 불필요. 실제 에이전트 팀에서 genasis 를 dogfood 하려는 경우만. | `npm install -g @anthropic-ai/claude-code` |

### Schema kernel 에만 필요

| 도구 | 필요한 이유 | 설치 |
|---|---|---|
| **atlas** | `genasis db migrate` 의 default 마이그레이션 도구 (postgres / mysql / sqlite). DB 파이프라인을 로컬에서 시험할 때만 필요. | `curl -sSf https://atlasgo.sh \| sh` |
| **psql** / **mysql** / **sqlite3** / **duckdb** | 실제로 사용하는 DB driver 별로 1개. `genasis db query` 가 read-only 경로에 이들을 shell out 합니다. | `apt-get install postgresql-client mysql-client sqlite3` · `brew install postgresql mysql sqlite duckdb` |

### Coverage / Codecov 작업에만 필요

| 도구 | 필요한 이유 | 설치 |
|---|---|---|
| **cargo-llvm-cov** | `coverage` CI job 이 `cargo llvm-cov --workspace --lcov` 를 돌리고 lcov.info 를 Codecov 에 업로드. coverage threshold 를 노리는 PR 을 열기 전에 로컬에서 실행하세요. | `cargo install cargo-llvm-cov` (1회) + `rustup component add llvm-tools-preview` |

### 문서 작업에만 필요

| 도구 | 필요한 이유 | 설치 |
|---|---|---|
| **markdownlint** (옵션) | CI 에서 강제하지는 않지만, 제출 전 markdown lint pass 로 stale anchor·broken cross-link 를 잡을 수 있음. | `npm install -g markdownlint-cli` |
| **ImageMagick** 또는 **rsvg-convert** | `docs/assets/og-image.svg` 와 `og-image.ko.svg` 가 source-of-truth; GitHub social-preview 슬롯용 PNG 변형은 `convert -background "#0b1320" -density 200 ... 1280x640` 로 렌더. | `apt-get install imagemagick` · `brew install imagemagick` |
| **asciinema** (옵션) | `docs/assets/demo.cast` 는 asciinema v2 포맷. demo 를 다시 녹화한다면 asciinema 먼저 설치. | `apt-get install asciinema` · `brew install asciinema` |

---

## 작업 흐름

1. **Fork 또는 branch.** 외부 컨트리뷰터는 fork, 메인테이너는 `main` 에서 branch. 브랜치 이름은 Conventional Commits 따름: `feat/`, `fix/`, `docs/`, `chore/`, `i18n/`.
2. **로컬 빌드 + 테스트.** `cargo build --workspace && cargo test --workspace --no-fail-fast`. 둘 다 통과해야 push.
3. **i18n drift lint.** `docs/ko/` 외부 `*.md` 또는 `.tera` 템플릿을 건드렸다면 `scripts/check-i18n-drift.sh --warn` 과 `scripts/i18n-extract-keys.sh` 실행. CI 도 둘 다 돌리고, release tag 는 drift 시 hard-fail.
4. **PR 열기.** GitHub 가 `.github/PULL_REQUEST_TEMPLATE.md` 를 적용; user-facing 문자열·문서 변경이라면 i18n 체크리스트 채우기.
5. **번역 흐름.** 새 `t!()` 키는 **반드시** `crates/genasis-i18n/locales/en.yml` 과 `ko.yml` 양쪽에 같은 commit 에서 추가. 영어 문서 변경은 mirror drift 를 warn; release tag 직전 `release-prep` 워크플로가 `[i18n] Translation completion` PR 자동 생성.
6. **새 로케일 추가.** [`docs/ko/i18n/CONTRIBUTE-LANG.md`](docs/ko/i18n/CONTRIBUTE-LANG.md) 참조.

## 코드 컨벤션

- **Conventional Commits** (`feat / fix / docs / chore / i18n`).
- **Squash-merge only** (repo 레벨에서 설정됨).
- **코드에 이모지 금지.** commit 메시지·CHANGELOG 는 OK.
- **주석보다 문서.** 동작 설명이 필요하면 해당 `docs/` 페이지나 함수의 rustdoc 에 적고, 일회성 코드 주석은 피함.
- **`unsafe` 는 반드시 `// SAFETY:` 블록 동반.** 예외 없음.

## 버그 발견 시

`.github/ISSUE_TEMPLATE/bug.md` 의 bug 템플릿으로 issue 열기. `genasis lang status` 출력과 OS / `uname -a` 행을 함께 첨부해 주세요.

## debug-history 패치 제출

ADR-012 §8 (Data-Only PR Model) 에 따라 사용자가 본인 프로젝트에서 만든
overlay 수정사항을 fork 없이 공유할 수 있다.

```bash
genasis debug status      # 매니페스트 대비 무엇이 drift 됐는지 확인
genasis debug collect     # 익명화된 patch.json 을
                          # ~/.genasis/debug-history/<project-hash>/ 에 작성
genasis debug submit      # patch 를 debug-history/patches/ 에 추가하는 PR
                          # 을 연다. 프로젝트당 24시간에 1회 제한.
```

기여자가 PR 에 포함할 수 있는 것은 **`debug-history/patches/*.patch.json`
하나뿐**이다. 템플릿 수정·코드 변경·문서 변경이 같은 PR 에 섞이면
`.github/workflows/debug-history-pr.yml` 에 의해 거부된다. 메인테이너는
누적된 패치를 `/debug-review` skill (`.claude/skills/debug-review/`) 로
처리해 별도 PR 에서 템플릿을 수정한다.
