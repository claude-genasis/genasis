> English: [../impact-of-multilang-prompts.md](../impact-of-multilang-prompts.md)

# 다국어 prompt 가 Claude 에이전트에 미치는 영향

> 2026-05-04 조사 — genasis M12 (i18n) 아키텍처 결정 근거.
> **TL;DR — 두 언어를 같은 agent context 에 동시에 넣지 마세요.**
> 설치 시 한 언어를 선택하고, 다른 언어는 디스크 reference docs 로만 두세요.
>
> 본 문서의 한국어 번역은 M12.7.b 에서 완성됩니다. 그 전까지는 영어 원본
> [`../impact-of-multilang-prompts.md`](../impact-of-multilang-prompts.md) 를
> 참조하세요. ADR-008 (이미 영어 원본) 이 결론을 한 페이지로 요약합니다.

## 결론 요약 (한국어)

1. **active-language singularity**: 사용자 repo 의 `.claude/` 안에는 단 한
   언어의 overlay 만 설치된다 (`genasis init --lang en|ko`).
2. **`--lang both` 거부**: Claude Code 자체에 언어 drift 버그가 존재하므로
   ([anthropics/claude-code#46846](https://github.com/anthropics/claude-code/issues/46846))
   두 언어 동시 설치는 protocol drift 위험이 너무 크다.
3. **`genasis lang switch <lang>`**: 1 commit 으로 atomic 교체 — prompt cache
   prefix 가 한 번에 회전.
4. **문서 듀얼 트리**: 영어 source-of-truth + `*.ko.md` / `docs/ko/` mirror.
   release 직전에 `release-prep.yml` 이 자동 translation-completion PR 을 생성.

전체 근거 (Anthropic 공식 가이드, arXiv 2406.20052 한국어 line-level confusion,
prompt cache 메커니즘, OSS 컨센서스 분석) 는 영어 원본을 참조.
