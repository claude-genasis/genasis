# Genasis

<p align="center">
  <a href="README.md"><img src="https://img.shields.io/badge/lang-English-blue?style=flat-square" alt="English"></a>
  <a href="README.ko.md"><img src="https://img.shields.io/badge/%EC%96%B8%EC%96%B4-%ED%95%9C%EA%B5%AD%EC%96%B4-red?style=flat-square" alt="한국어"></a>
  <a href="docs/ko/i18n/CONTRIBUTE-LANG.md"><img src="https://img.shields.io/badge/+-add%20language-lightgrey?style=flat-square" alt="Add a language"></a>
</p>

> 🇰🇷 **한국어** | [🇺🇸 English](README.md)

> **Plane × Mattermost × TDD × Design × DB × Monitor — 어떤 Claude Code 에이전트 팀에든 overlay 방식(재작성 X)으로 부착.** curl 한 줄로 설치. 한국어·영어 지원.
>
> 태그: `claude-code` · `agentic-team` · `agent-orchestration` · `plane-issues` · `mattermost-bot` · `tdd` · `rust-cli` · `multi-agent` · `ratatui` · `i18n` · `한국어` · `에이전트` · `클로드`

**상태:** v0.0.1 (M12 — 다국어 지원). [progress.ko.md](progress.ko.md) 참조.

---

## 무엇인가

11k 줄짜리 bash 스크립트(`create-agentic-team.sh`, 통칭 "Genesis") 가
단일 프로젝트용 에이전트 팀을 부트스트랩했다면, **Genasis** 는 그 패턴을
다국어·모듈화된 후속으로 일반화합니다:

- **단일 Rust 바이너리** (대상 머신에 Python·Node 런타임 의존성 없음).
- **비파괴 overlay** — 기존 `.claude/agents/*.md` 를 재작성하지 않습니다.
  agent 마다 작은 marker-fence 블록을 주입하고 나머지는 그대로 둡니다.
- **가역적** — `genasis detach` 한 번으로 전부 제거.
- **멱등** — `attach` 두 번 실행해도 같은 결과.
- 부착 시각화 + 런타임 `genasis monitor` 대시보드를 위한 **풍부한 TUI**.

[blueprint.ko.md](blueprint.ko.md) 에서 전체 설계 문서를 읽으세요.

---

## 설치 (사용자)

```bash
curl -fsSL https://raw.githubusercontent.com/OWNER/genasis/main/install.sh | sh
```

자동 감지 결과 또는 `--lang` 플래그로 한국어/영어 중 하나를 선택합니다.
`--lang both` 는 거부됩니다 — Claude Code 의 언어 drift 위험 회피
([docs/ko/impact-of-multilang-prompts.md](docs/ko/impact-of-multilang-prompts.md) 참조).

```bash
curl -fsSL .../install.sh | sh -s -- --lang ko       # 명시적 한국어
curl -fsSL .../install.sh | sh -s -- --lang en       # 명시적 영어
curl -fsSL .../install.sh | sh                       # 대화형 prompt (TTY) 또는 $LANG fallback
```

설치기 동작:

1. OS / arch 감지 (Linux x86_64/arm64, macOS arm64/x86_64, WSL).
2. **선결 패키지 검사** (git, curl, tar; 옵션: node ≥18, gh, atlas, psql/mysql/sqlite3/duckdb, rtk, claude).
3. 누락된 항목에 대해 **OS별 설치 명령**을 출력 (자동 설치는 안 함).
4. 릴리즈 바이너리 다운로드, sha256 검증, `~/.local/bin/genasis` 에 설치.
5. (선택) `genasis attach` 자동 실행하여 현 프로젝트에 부착.

플래그:

```
install.sh [--lang=LANG] [--non-interactive] [-y|--yes]
           [--no-run] [--prefix=PATH] [--version=vX.Y.Z]
```

---

## 소스에서 빌드 (컨트리뷰터)

```bash
git clone https://github.com/OWNER/genasis
cd genasis
cargo build --release
./target/release/genasis --help
```

툴체인은 [rust-toolchain.toml](rust-toolchain.toml) 에 고정 (Rust 1.78+).

---

## 한눈에 보는 사용법

```bash
genasis init        # 빈 프로젝트 → ECC 팀 + overlay + Plane/MM 프로비저닝
genasis attach      # 기존 팀 → overlay 부착, 원본 파일 대체로 보존
genasis detach      # overlay 제거 (marker fence 만)
genasis doctor      # 환경/도구/권한 검증
genasis upgrade     # overlay 버전 업그레이드 (fence 해시 diff)

genasis monitor     # Ratatui TUI: sprint, tokens, agents, deploy, network, logs
genasis lang status            # 현재 active 언어 + 사용 가능 locale
genasis lang switch <en|ko>    # atomic locale 교체

genasis design swap <reference-url>
genasis db query "SELECT ..."     # SQL guard 가 적용된 read-only
genasis db migrate                 # Atlas / Drizzle Kit / DuckDB raw runner 위임
```

---

## 왜 "Genasis"?

`genesis` (스크립트) → `genasis` (프레임워크). 같은 어근, 더 넓은 범위.

---

## 라이선스

MIT — [LICENSE](LICENSE) 참조.

---

## 상태

이 저장소는 부트스트랩 단계입니다. 위에 설명된 기능은 **목표**이며,
현재 실행 가능한 상태가 아닙니다. 추적: [progress.ko.md](progress.ko.md).

`<OWNER>` placeholder 는 저장소가 공개되는 시점에 실제 GitHub owner 로
교체됩니다.

---

### 다른 언어 / Other languages
- 🇰🇷 [한국어](README.ko.md)
- 🇺🇸 [English](README.md)
