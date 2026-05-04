> English: [../../ADR/ADR-008-i18n-install-time-selector.md](../../ADR/ADR-008-i18n-install-time-selector.md)

# ADR-008: i18n — install 시점 언어 선택 + active-language 단일성

## 상태

승인됨 (2026-05-04). blueprint §19 (M12) 의 결정 기록.

> 본 ADR 의 한국어 본문은 M12.7.b 번역 단계에서 영어 원본으로부터 완성됩니다.
> 그 전까지는 영어 원본
> [`../../ADR/ADR-008-i18n-install-time-selector.md`](../../ADR/ADR-008-i18n-install-time-selector.md)
> 을 source-of-truth 로 참조하세요.

## 결정 요약

1. **active-language 단일성**: 사용자 repo 의 `.claude/` 안에는 단 한 언어의
   overlay 만 설치됩니다 (`genasis.toml [i18n] active`).
2. **`--lang en|ko` 설치 시점 선택** + interactive prompt + `$LANG` fallback.
3. **`--lang both` 거부** — Claude Code 의 언어 drift 버그 회피
   ([#46846](https://github.com/anthropics/claude-code/issues/46846),
   [#24941](https://github.com/anthropics/claude-code/issues/24941)).
4. **`genasis lang switch <lang>`** — 1 commit atomic 교체 (prompt cache 효율).
5. **런타임 i18n: rust-i18n** (fluent-rs 보다 가볍고 메시지 규모에 적합).
6. **문서 듀얼 트리**: 영어 source + `*.ko.md` / `docs/ko/` mirror.
7. **CI 3-tier**: PR warn / release-prep strict / 자동 translation-completion PR.

## 검토된 대안

A. 두 언어 동시 install — 거부 (instruction divergence + cache cost + 모델 drift).
B. 영어만 + 외부 wrapper (claude-ts) — install 시점 한국어 미지원으로 진입장벽.
C. Crowdin / Weblate — 1차 over-engineering.
D. fluent-rs — 한국어 복수형 변화 없음, 50개 메시지에 표현력 과잉.
E. PR 마다 hard-fail drift — 컨트리뷰터 진입장벽 ↑, 3-tier 게이트 채택.

## 참조

- 조사: [`../impact-of-multilang-prompts.md`](../impact-of-multilang-prompts.md)
- Plan: [`../../../blueprint.ko.md`](../../../blueprint.ko.md) §19
- Tracker: [`../../../progress.ko.md`](../../../progress.ko.md) M12
- 관련 ADR: ADR-001 (overlay marker fence), ADR-002 (Rust 단일 바이너리),
  ADR-005 (provider flavor system).
