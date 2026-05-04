//! Render the external-mode `docs/design-system.md` pointer body.
//!
//! In pristine mode, `design-system.md` is the truth. In external mode it
//! becomes a thin pointer with three sections:
//!
//! - §A — points at `<external_dir>/DESIGN.md` as the 1st-class reference.
//! - §B — accumulates user overrides (per the conflict-resolution policy
//!   enforced by the `design-aware` skill).
//! - §C — operator manual (swap / restore / verify / override).
//!
//! M-D1 emits the §A/§B/§C skeleton with empty §B.2. M-D2's `override add`
//! command appends entries to §B.2 without rewriting the rest of the file.

use crate::mode::State;

const EN_TEMPLATE: &str = r#"---
mode: external
source: {source}
slug: {slug}
applied_at: {applied_at}
gallery_preview: {gallery_preview}
template_hash: {template_hash}
---

# Design System — External Reference

> ⚠️ This file is a **pointer** to an external design specification.
> Before any UI work, read in this order: §A → §B. §A is base; §B patches it.
> Never edit `{external_dir}/DESIGN.md` directly — use `genasis design swap`
> for source changes or §B for user overrides.

## §A. 1st-class truth — external DESIGN.md

- **Path**: `{external_dir}/DESIGN.md`
- **Origin**: {source}
- **Hash**: sha256:{template_hash}  (`genasis design verify` re-checks this)
- **Preview**: {gallery_preview}

All UI tokens, components, layout, and typography decisions originate from
that file. Treat it as read-only. To replace the active design, run
`genasis design swap <slug>` or `genasis design swap --from <path>`.

## §B. User overrides (applied on top of §A)

> When a human asks for a design/style change, the agent must consult §A
> first, surface conflicts, and append accepted overrides here. §B wins
> over §A on overlap.

### B.1 Conflict-resolution policy (enforced by the `design-aware` skill)

1. The agent quotes the relevant §A item and the user's request side-by-side.
2. If the request matches §A → proceed silently. No override is recorded.
3. If the request conflicts with §A → the agent asks explicitly:
   "DESIGN.md says X = `<old>`. You're asking for `<new>` — override
   DESIGN.md? [y/N]"
4. On `y`, the agent runs `genasis design override add "<request>"`, which
   appends a dated entry to §B.2 below and bumps `override_count` in
   `docs/.design-state.toml`.
5. On `n`, the agent honours §A and offers an alternative.

### B.2 Accumulated overrides (chronological)

<!-- genasis design override add appends here. Do not edit by hand. -->

## §C. Operator manual

| Action | Command |
|---|---|
| Swap to a different external design | `genasis design swap <slug>` |
| Swap from a local spec file         | `genasis design swap --from <path>` |
| Restore to the pre-external body    | `genasis design restore` |
| Verify DESIGN.md was not tampered   | `genasis design verify` |
| Add a user override (with conflict) | `genasis design override add "<text>"` |
| List current overrides              | `genasis design override list` |
| Remove an override                  | `genasis design override remove <id>` |
| Open the gallery in a browser       | `genasis design preview` |

Status snapshot: `genasis design status`.
"#;

const KO_TEMPLATE: &str = r#"---
mode: external
source: {source}
slug: {slug}
applied_at: {applied_at}
gallery_preview: {gallery_preview}
template_hash: {template_hash}
---

# Design System — 외부 참조 모드

> ⚠️ 이 파일은 외부 디자인 지침의 **포인터**입니다.
> UI 작업 전에 §A → §B 순서로 반드시 읽으세요. §A 가 base, §B 가 patch입니다.
> `{external_dir}/DESIGN.md` 를 직접 편집하지 마세요 — 본문 변경은
> `genasis design swap`, 사용자 요구는 §B 오버라이드로만 표현하세요.

## §A. 1차 진실 — 외부 DESIGN.md (필수 선참조)

- **경로**: `{external_dir}/DESIGN.md`
- **원본**: {source}
- **해시**: sha256:{template_hash}  (`genasis design verify` 가 재검증)
- **미리보기**: {gallery_preview}

UI 토큰·컴포넌트·레이아웃·타이포그래피의 모든 결정은 위 파일에서 비롯합니다.
읽기 전용으로 취급하세요. 활성 디자인을 교체하려면
`genasis design swap <slug>` 또는 `genasis design swap --from <path>` 를 사용하세요.

## §B. 사용자 오버라이드 (§A 위에 누적)

> 사용자가 design/style 관련 요구를 줄 때, 에이전트는 먼저 §A 의 관련 항목을
> 인용하고 충돌을 명시한 뒤 승인된 오버라이드만 여기에 누적합니다.
> §B 는 §A 와 충돌 시 우선합니다.

### B.1 충돌 해결 정책 (`design-aware` 스킬이 강제)

1. 요청과 §A 의 관련 항목을 나란히 인용한다.
2. 요청이 §A 와 일치 → 그대로 진행. 기록 없음.
3. 요청이 §A 와 상충 → 명시적으로 확인:
   "DESIGN.md 의 X = `<old>` 인데 요청하신 `<new>` 와 상충합니다.
    DESIGN.md 를 무시하고 진행할까요? [y/N]"
4. `y` 승인 시: `genasis design override add "<요청>"` 호출 → 아래 §B.2 에
   날짜와 함께 자동 append, `docs/.design-state.toml.override_count` 증가.
5. `n` 거부 시: §A 그대로 따르고 대안을 제시.

### B.2 누적 오버라이드 (시간순)

<!-- genasis design override add 가 자동 append. 직접 편집 금지. -->

## §C. 사용 매뉴얼

| 동작 | 명령 |
|---|---|
| 다른 외부 디자인으로 교체 | `genasis design swap <slug>` |
| 로컬 spec 파일에서 교체 | `genasis design swap --from <path>` |
| 외부 모드 진입 전으로 복원 | `genasis design restore` |
| DESIGN.md 변조 검증 | `genasis design verify` |
| 사용자 오버라이드 추가 (충돌 검토) | `genasis design override add "<text>"` |
| 현재 오버라이드 목록 | `genasis design override list` |
| 오버라이드 삭제 | `genasis design override remove <id>` |
| 갤러리 미리보기 열기 | `genasis design preview` |

상태 요약: `genasis design status`.
"#;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    En,
    Ko,
}

impl Locale {
    pub fn from_active(active: &str) -> Self {
        if active.starts_with("ko") {
            Locale::Ko
        } else {
            Locale::En
        }
    }
}

/// Render the pointer body from the active state + the configured external
/// directory. The output is deterministic given the same inputs (used by
/// golden tests).
pub fn render(state: &State, external_dir: &str, locale: Locale) -> String {
    let template = match locale {
        Locale::En => EN_TEMPLATE,
        Locale::Ko => KO_TEMPLATE,
    };
    template
        .replace("{source}", maybe_blank(&state.source))
        .replace("{slug}", maybe_blank(&state.slug))
        .replace("{applied_at}", maybe_blank(&state.applied_at))
        .replace("{gallery_preview}", maybe_blank(&state.gallery_preview))
        .replace("{template_hash}", maybe_blank(&state.template_hash))
        .replace("{external_dir}", external_dir)
}

fn maybe_blank(s: &str) -> &str {
    if s.is_empty() {
        "(unset)"
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mode::Mode;

    fn fixture() -> State {
        State {
            mode: Mode::External,
            slug: "posthog".into(),
            source: "getdesign/posthog".into(),
            source_command: "npx getdesign add posthog".into(),
            template_hash: "deadbeef".into(),
            applied_at: "2026-05-04T10:00:00Z".into(),
            previous_slug: String::new(),
            gallery_preview: "https://getdesign.md/posthog/design-md".into(),
            gallery_index: "https://getdesign.md/".into(),
            override_count: 0,
        }
    }

    #[test]
    fn en_render_substitutes_all_placeholders() {
        let body = render(&fixture(), "docs/design-system", Locale::En);
        assert!(body.contains("posthog"));
        assert!(body.contains("docs/design-system/DESIGN.md"));
        assert!(body.contains("sha256:deadbeef"));
        assert!(body.contains("genasis design swap"));
        assert!(!body.contains("{slug}"));
    }

    #[test]
    fn ko_render_uses_korean_headings() {
        let body = render(&fixture(), "docs/design-system", Locale::Ko);
        assert!(body.contains("외부 참조 모드"));
        assert!(body.contains("§A. 1차 진실"));
        assert!(body.contains("§B. 사용자 오버라이드"));
        assert!(!body.contains("{slug}"));
    }

    #[test]
    fn locale_from_active_handles_ko_variants() {
        assert_eq!(Locale::from_active("ko"), Locale::Ko);
        assert_eq!(Locale::from_active("ko-KR"), Locale::Ko);
        assert_eq!(Locale::from_active("en"), Locale::En);
        assert_eq!(Locale::from_active(""), Locale::En);
    }
}
