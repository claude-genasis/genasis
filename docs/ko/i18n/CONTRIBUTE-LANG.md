# Genasis 에 새 언어 추가하기

> English: [../../i18n/CONTRIBUTE-LANG.md](../../i18n/CONTRIBUTE-LANG.md)

Genasis 는 영어(`en`) 와 한국어(`ko`) 를 기본 지원합니다. 다른 언어
(예: 일본어 `ja`) 추가는 4개의 병렬 surface 를 건드리는 기여입니다. 한
PR 에 묶어서 `[i18n] Add <Language> support` 제목으로 제출하세요.

## Surface

1. **Tera 템플릿 트리** — `crates/genasis-templates/templates/<lang>/`
   - `templates/en/` 의 레이아웃을 그대로 미러. 39개 `.tera`:
     1 GENASIS.md, 1 genasis.toml, 1 env.agents, 1 mcp.json,
     1 design-system, 10 agent-overlays, 16 commands, 6 skills, 6 hooks.
   - env vars (`${PLANE_TOKEN_*}`, `${MM_TOKEN_*}`), 경로, 코드블록, URL,
     Tera 태그 (`{{ var }}`, `{% if %}`) 보존.
   - `crates/genasis-templates/src/lib.rs` 의 `SUPPORTED_LANGS` 갱신.
2. **런타임 i18n 번들** — `crates/genasis-i18n/locales/<lang>.yml`
   - `en.yml` 의 모든 key 미러. 누락 key 는 런타임에서 영어로 fallback
     되지만, `lint-i18n` 이 warn 하고 `release-prep` 은 hard-fail.
   - `crates/genasis-i18n/src/lib.rs` 의 `Lang` enum + `parse()` 갱신.
3. **문서 트리** — `docs/<lang>/`
   - `ARCHITECTURE.md`, `PROVIDERS.md`, `MIGRATION-FROM-GENESIS.md`,
     `TOKEN-ECONOMICS.md`, `MONITOR.md`, `impact-of-multilang-prompts.md`,
     `ADR/ADR-001`–`ADR-008` 미러.
4. **README** — `README.<lang>.md`
   - `README.md` 의 18-절 구조 미러.
   - **기존 모든 README** 의 badge row 에 언어 추가
     (`README.md`, `README.ko.md`, 기타 `README.<lang>.md`).
   - 모든 README 의 bottom navigation 섹션에 언어 추가.
   - GitHub repo Topics 에 언어 태그 추가 요청 PR (`japanese`, `日本語` 등).

## 검증

PR 제출 전 실행:

```bash
scripts/check-i18n-drift.sh --check-mirror-not-empty
scripts/i18n-extract-keys.sh
cargo test -p genasis-i18n
cargo test -p genasis-templates
```

CI 의 `lint-i18n` job 이 PR 에서 다시 실행합니다.

## 번역 원칙

- **코드블록, env vars, CLI 명령, URL 은 절대 번역하지 마세요.**
- **Plane / Mattermost 라이프사이클 용어** (`Todo`, `In Progress`,
  `In Review`, `Done`, `PR`, `merge`, `squash`) 는 영어 그대로 — 실제
  Mattermost / Plane UI 와 일치.
- **`@멘션` 문법** 은 영어 그대로 (`@qa.{{ project_name }}`).
- **마크다운 헤딩** 은 번역, 레벨(`##` / `###`) 은 유지.

## 런타임에서 언어 활성화

merge 후 사용자는 다음으로 언어 선택:

```bash
genasis init --lang <lang>
genasis lang switch <lang>
```

`install.sh` 는 `$LANG` 을 파싱해 fallback (예: `ja_JP.UTF-8` → `ja`);
Bash `suggest_lang()` 함수에 해당 locale 분기를 추가하세요.
