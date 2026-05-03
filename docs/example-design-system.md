# SecureStage Design System — Studio-Refined Edition

> **Ownership**: `designer` agent.
> **Version**: 3.0 — 2026-04-13. Re-calibrated against the live
> [the9thave.com](https://the9thave.com/) home page, which sampled
> navy-on-white with 1-px borders, 5-px *navy* hard-offset shadow, and
> type weights capped at 500. v2's heavy 2-px black borders / 4-px black
> shadows / 600–700 weights read as loud neobrutalism; v3 dials that back.
>
> **Aesthetic label**: *Studio-refined brutalism* — the brutalist building
> blocks (hard-offset shadow, thin hairline borders, sharp rectangles) are
> present, but executed with editorial restraint. Think 9th-Avenue studio
> first, Positivus/neobrutalism second.

## 0. Revision history

| Version | Date | Highlights |
|---|---|---|
| v2 (Gumroad-era) | — | Warm paper bg, lime accent, 2 px black borders, 4 px diagonal shadow, weights 600–700. |
| **v3.0** | 2026-04-13 | Rewrite against the9thave.com — white canvas, navy accent, 1 px border, 5 px vertical **navy** shadow, weights 500 max, radius halved (2/4/6). |
| **v3.1** | 2026-04-13 | Container `padding-inline` 20 → 32 px so grid first/last columns breathe. `--color-fg-inverse-muted` cascade on `.card--dark`. Shadcn neobrutalism tokens (`bg-main`, `shadow-shadow`, `rounded-base`, …) wired via `@theme`. |
| **v3.2** | 2026-04-13 | Radius further reduced (`md 6`, `lg 10`) per "곡률이 크면 못생겨". `.step` + `.cia` circular utilities. Nested-card anti-pattern rule. `.stack` over `space-y-*`. CIA disc with reserved 3-colour system (blue/green/amber) via `<CIABadge>`. |
| **v3.3** | 2026-04-13 | Card backgrounds no longer pure white — default `--surface-cream`; palette-hue tints for navy/lime/amber/violet/coral. Neobrutalism-only component policy (§0.1) hardened. CI greps. |
| **v3.4** | 2026-04-13 | `.tag` utility (2 px radius — sharpest) for dense quiz/tech-stack labels. `--ink-*` AAA text tokens for status text on light surfaces. `.state` wrapper pairs surface + ink automatically. Fixes contrast failure of bright accents used as text. |
| **v3.5** | 2026-04-13 | Header collapsed from two rows to **one 44-px row**. Wordmark row deleted. Top nav is a single horizontal row of PART category chips; each chip opens a dropdown listing lessons. CryptoStage link moves into the same row (right-aligned). See §10. |
| **v3.6** | 2026-04-13 | Card-to-card vertical gap halved (`.stack` 40→20, `--loose` 48→24, `--tight` 16→8). Top nav chips shrink to `--text-xs` so 8 PART chips fit a 1240 bar without scroll. CryptoStage chip removed from the nav. **Three chip states** visually distinct: default (ghost), **active** (= current page's PART, navy fill), **open** (= temporarily peeking, saffron fill). Submenu panel gets brutalist 2-px border + **5 × 5 diagonal navy shadow** (right + bottom) — the one sanctioned diagonal shadow in the system. Submenu-link hover outline (`.submenu-link`) — §10.1c. |
| **v3.7** | 2026-04-13 | **Curriculum restructure**. Removed: `/pfs`, `/ssh-debug`, `/ssh-files`, `/web-nodes`, `/vpn-protocols`, `/pipa-breach`. Added PART 5 **Web App 보안** (5p: overview, injection, supply-chain, auth, devops). PART 6 **VPN** rewritten concept-first (사설망 / NAT / SSH-tunnel / WireGuard 실습 / 시나리오) — dropped algorithm comparison. Final structure: 9 PART (0~8), 30 pages. See §11a for the ported curriculum map. |
| **v3.8** | 2026-04-13 | **Accordion pattern standardised** (§9.4f). `/ssh-keygen` `.ssh` 폴더 설명을 카드 아코디언으로 변경 — 각 항목의 실제 파일 내용이 펼쳐짐. **`/auth` 에 MFA 3-factor 섹션 신설** — What I know / have / am × 5-step strength ladder + AiTM 등 공격 조건 + L1~L5 단계별 추천 구현. **Page-root template (§10a)** + **13-grep enforcement (§10b)** 문서화. 24개 파일에서 금지 패턴(`<article max-w-*>`, `py-8` page-root, `font-bold`, `space-y-5/16`) 제거 — 전 학습 페이지 root 패턴 통일. |
| **v3.9** | 2026-04-24 | **Typography + accessibility + radius consistency pass.** (a) Pretendard Variable promoted to primary sans/display font — Space Grotesk / Inter kept as Latin fallbacks only (§3.1). CDN link added in `app/layout.tsx`. (b) Semantic status tokens (`--color-success/warning/danger/info`) and the legacy `--color-cia-i` alias rebound from the bright brand primitives (lime/coral/saffron/violet — which failed WCAG AA as text, e.g. lime `#B9FF66` on white = 1.5 : 1) to the AAA `--ink-*` variants. Bright fills remain opt-in via new `*-fill` tokens / `.badge--*-fill` classes. (c) `.badge--success/warning/danger/info` switched from full-saturation fill to pale-surface + ink-text pattern (matches `.tag--*` and `.state--*`). (d) CIA-I disc moved from green-600 → green-700 to clear AA with white; confidentiality bumped blue-600 → blue-700. (e) `.eyebrow` colour `--color-fg-subtle` → `--color-fg-muted` and `--color-smoke` lifted gray-500 → gray-600 so tertiary text clears AA on every pale tint. (f) **Control radius HARD RULE** codified (§5.2a) — `rounded-md` (6 px) is the default for buttons, inputs, chips, small boxes; `rounded-lg` (10 px) is reserved for full-page hero cards only. Sweep: 46 `rounded-lg` occurrences on cryptography / SSH / PIPA simulators changed to `rounded-md`; 16 inline `borderRadius` literals (`0.5rem` / `0.625rem` / `0.75rem`) replaced with `var(--radius-md)`. Automated contrast scan (`.audit/contrast-scan.mjs`) goes from **114 failing → 0 failing** across all 31 learning pages with every accordion expanded. |
| **v3.10** | 2026-04-25 | **Button + chip HARD RULE** (§7a) — every interactive control in a learning-page surface must use the `.btn` / `.btn--sm` utility (brutalist border + 5 px navy shadow + translate-on-press). Hand-rolled `px-4 py-2 bg-* rounded-* text-* font-medium` constructions banned — they rendered as flat color boxes indistinguishable from chips. 40+ call sites migrated across `HashingLiveDemo`, `AESLiveDemo`, `RSALiveDemo`, `SSHChallengeDemo`, `TLSHandshakeDemo`, `PasswordStorageDemo`, `BcryptCostDemo`, `RainbowTableDemo`, `PEMParser`, `PFSSimulator`, `DHSimulator`, `HMACDemo`, `AESDemo`, `AsymmetricDemo`, `ChallengeResponseDemo`, `SSHSimulator`, `CertErrorScenarios`. Small inputs similarly normalised to `.input` utility. **Chip padding HARD RULE** (§9.4) — `.chip` switched from fixed `height` to `min-height` + `line-height: 1.15` + explicit per-side `padding-top/bottom: 4px + padding-left/right: 16px` so long labels never clip or kiss the border; `.chip--sm` bumped from `0 / 12 px` to `2 / 12 px`; `.tag` vertical padding lifted 2 → 3 px. All `px-2 py-0.5 rounded-full` ad-hoc pills (PKI/TOFU labels, hash-status chips, ssh-keygen recommendation chips, PART header pills on `/glossary` `/pipa-*`) replaced with `.chip .chip--sm`. **Cascade fix** — `neobrutalism.css` now `@import`ed with `layer(components)` so it beats Tailwind v4's `@layer base` preflight reset on Chromium; the unlayered duplicate `* { padding: 0 }` inside `globals.css` was deleted (it was clobbering the vertical padding on `.chip/.btn/.input` even at higher specificity). `.btn/.input/.chip/.tag` use per-side physical longhand (`padding-top/bottom/left/right`) instead of logical shorthand for the same reason. **Menu panel re-skin** — the `TopNav` dropdown now renders on a navy surface with white text, lime current-page fill, ink-soft drop shadow. Cream-on-cream was indistinguishable from content cards; bold navy satisfies the neobrutalism "nav chrome ≠ content" separation rule. New `.submenu-link--on-dark` modifier carries the inverted hover state. **Stack spacing lift** — `.stack` gap 20 → 24 px, `.stack--tight` 8 → 12 px, `.stack--loose` 24 → 32 px so the 5 px navy shadow always clears the next card; previously an 8 px gap fused shadow + next border into a "double line" artefact. Result: consistent 28–36 px control touch targets, visible neobrutalism shadow on every button and between every stacked card, chip text never overflows on 320 px viewport, menu panel reads as distinct nav chrome. |
| **v3.11** | 2026-04-25 | **Single source of truth**. `docs/design-guide.md` (the v2-era quick-reference cheat sheet) **deleted** — it had accumulated stale rules (font-weight max-700, radius 4/6-only, no `--ink-*` AAA tokens, no v3.9/v3.10 button/chip/menu hard rules) and created two parallel docs that silently diverged. Practical-workflow content that only lived in the guide (editing-decision tree, "tighten/loosen" tuning recipes, horizontal-gap density table, global hygiene grep set, `:root`-vs-component edit meta-rule) absorbed into this document as §0.2–§0.4 + §2.6 + §4a. `CLAUDE.md` and all agent definition files (`.claude/agents/*.md`) and command files (`.claude/commands/*.md`) now mandate **design-system.md as the single design reference** — any future "디자인 가이드" lookup must land here. |

---

## 0.0 What changed vs v2

| Axis | v2 value | v3 value | Reason |
|---|---|---|---|
| Canvas | `#F3F3F3` paper | **`#FFFFFF` pure white** | 9thave uses white; paper read as vintage |
| Primary accent | lime `#B9FF66` | **navy `#001B5E`** | 9thave home is navy-dominant; lime is too playful |
| Border width | 2 px default | **1 px default** | 9thave CTA uses 1px; 2px is loud |
| Border color | black | **navy** on navy-themed, black on text | matches 9thave shadow |
| Shadow | `4px 4px 0 0 #000` | **`0 5px 0 0 navy`** | 9thave signature — vertical only, navy tint |
| Corner radius | 2/4/6 px | **4/8/12 px** | half of 9thave's 16/24; user brief "절반 정도" |
| Heading weight | 600–700 | **500** | 9thave caps at 500 |
| Body weight | 500 | **400–500** | 500 only below `--text-sm`; 400 at base fine |
| Card bg | colored accents per card | **white by default**, navy for hero block | restraint |
| Hover pattern | `translate(2px,2px) shadow-xs` | **`translate(0,3px) shadow-none`** | vertical push, not diagonal |

Result: calmer, more corporate-studio, still with brutalist affordance via
navy offset shadow.

---

## 0.2 Where to edit — single source-of-truth meta-rule (v3.11)

**This document is the only design reference.** If you came here from
`docs/design-guide.md`: that file was merged in and deleted on
2026-04-25. Any agent/command instruction that points at the old guide
should be updated to point here.

**Rule**: toggle the knob in exactly one place.

```
"글자 크기를 바꿔야겠다"
    │
    ├── 한 요소만?          → 컴포넌트 파일에서 style={{ fontSize: 'var(--text-xl)' }}
    └── 사이트 전체?         → src/styles/neobrutalism.css :root 의 --text-* 토큰 수정

"여백을 바꿔야겠다"
    │
    ├── 한 카드만?           → 컴포넌트 파일에서 padding / gap utility
    ├── 섹션 전체?           → --space-section 토큰 조정
    └── 컨테이너 폭?          → --container-lg / --container-md 조정

"색을 바꿔야겠다"
    │
    └── 모든 경우             → :root 토큰만 수정 (--color-main / --color-lime / --color-bg)
                               컴포넌트 파일에 hex 직접 입력 금지
```

Never hardcode `px` literals, `#hex`, or `font-weight < 500` in a
component file. The enforcement grep set in §0.4 is the audit tool.

---

## 0.3 Tuning workflows — "조금만 크게 / 좁게 / 넓게" (v3.11, from guide)

These are the concrete recipes for common fine-tuning requests. Each
recipe names one token to move so the change propagates everywhere.

### 0.3.1 "본문이 너무 작다 / 크다"

- 한 곳만 키우기:
  ```tsx
  <p style={{ fontSize: 'var(--text-lg)' }}>...</p>
  ```
- 사이트 전체 본문 크기:
  ```css
  /* neobrutalism.css :root */
  --text-base: clamp(1.0625rem, 1rem + 0.2vw, 1.125rem);  /* 17 → 18 px */
  ```
  `clamp()`의 가운데(`preferred`) 값을 1단계 키우는 것만으로 충분.

### 0.3.2 "화면이 너무 빡빡해" — 여백 늘리기

1. 섹션 간격이 좁다 → `--space-section` 을 올림:
   ```css
   --space-section: clamp(4rem, 3rem + 4vw, 6rem);  /* 64 → 96 px */
   ```
2. 카드 안쪽이 답답 → 해당 카드에 `padding: var(--space-8)` (32 px).
3. 버튼이 납작 → `.btn--lg` 로 교체 (52 px 높이). `.btn` 기본도 44 px.
4. 좌우 여백 부족 → `--container-gut` 하한을 올림.

### 0.3.3 "너무 넓어" — 좁히기

- 섹션 간격 좁히기: `--space-section: clamp(2rem, 1.5rem + 3vw, 4rem);`
- 카드 안쪽: `padding: var(--space-4)` (16 px).
- 리스트 행 간격: `gap: var(--space-2)` (8 px).

### 0.3.4 굵기·줄간격 미세 조정 한계

- 굵기: **500 미만 금지** (a11y). 히어로가 얇게 보이면 `--text-display`
  의 clamp 하한을 올리는 쪽이 먼저. 500→이상은 §1의 "restrained type"
  원칙을 깨므로 designer agent RFC 필요.
- 줄간격: `--leading-body` 는 1.55 기본. 긴 글은 1.65까지 허용
  (`--leading-prose`). 1.7 초과는 prose 전용 영역에서만.

---

## 0.4 Global hygiene grep set (v3.11, from guide)

Run before every PR. All three must return zero hits in `src/app` and
`src/components` (learning + demos + layout). Section-specific CI greps
also live in §5.2a (radius), §7a.6 (buttons/chips), §7.5 (stack gaps),
§10b (page-root template) — this set is the catch-all.

```bash
# 1. Hex literal in component files (only tokens.css / neobrutalism.css
#    @theme / primitive palette may carry hex)
grep -rnE "#[0-9a-fA-F]{3,8}\b" src/components src/app \
  | grep -v "data-\|url(" \
  | head

# 2. Arbitrary px / rem literal bypassing the space/radius/text scales.
#    Text size, padding, margin, gap are the enforced-token axes.
#    `min-w-[14rem]` / `max-w-[500px]` style constraints for responsive
#    flex behaviour are allowed — intentional per-instance sizing.
grep -rnE ":\s*[0-9]+(\.[0-9]+)?px\b" src/components src/app \
  | grep -v "// \|/\*" | head
grep -rnE 'className="[^"]*\b(text|p|pt|pb|pl|pr|px|py|m|mt|mb|ml|mr|mx|my|gap|gap-x|gap-y|space-x|space-y)-\[' \
  src/components src/app | head

# 3. Low font-weight (a11y minimum is 500)
grep -rnE "font-weight:\s*(100|200|300|400)" src/
grep -rnE 'className="[^"]*\bfont-(thin|extralight|light|normal)\b' src/
```

A non-empty result from any of these is a design-system violation.
Route the fix through the appropriate section above (never add an
exception in-place).

---

## 0.1 Component policy — neobrutalism-only (v3.3, HARD RULE)

> **This is a hard rule. Pull requests that introduce bespoke
> primitives are rejected on sight and must be rewritten against the
> approved component set below.**

SecureStage ships **one** design vocabulary. Every interactive surface,
every card, every button, every chip, every badge, every alert — it
comes from the approved set below. No component is "too simple" or
"just this once" to bypass this rule. Drift accumulates and the next
agent that touches the page has to undo it.

### Approved sources (the only ones)

1. **React components** in `src/components/ui/*` — these are the
   [neobrutalism.dev](https://www.neobrutalism.dev/) shadcn set already
   wired to v3 tokens:
   `Accordion`, `Alert`, `Badge`, `Button`, `Card`, `Checkbox`, `Dialog`,
   `Drawer`, `DropdownMenu`, `Form`, `Input`, `Label`, `NavigationMenu`,
   `Popover`, `Progress`, `RadioGroup`, `Select`, `Sheet`, `Skeleton`,
   `Slider`, `Switch`, `Table`, `Tabs`, `Textarea`, `Tooltip`, …

2. **CSS utility classes** in `src/styles/neobrutalism.css`:
   - Layout: `.container`, `.container--sm|md|xl|wide`, `.stack`, `.stack--tight`, `.stack--loose`, `.row`, `.row--wrap`, `.divider`, `.divider--strong`
   - Typography: `.eyebrow`, `.lede`
   - Buttons: `.btn`, `.btn--main`, `.btn--ghost`, `.btn--outline`, `.btn--danger`, `.btn--sm`, `.btn--lg`
   - Cards: `.card`, `.card--ghost`, `.card--plain`, `.card--ivory`, `.card--navy-tint`, `.card--lime-tint`, `.card--amber-tint`, `.card--violet-tint`, `.card--coral-tint`, `.card--dark`, `.card--flat`, `.card--feature`, `.card--interactive`, `.card--compact`
   - Chips & markers: `.chip`, `.chip--active`, `.chip--sm`, `.step`, `.step--lg`, `.step--ghost`, `.cia`, `.cia--sm|md|lg`, `.cia--c|i|a`, **`.tag`**, **`.tag--navy|lime|amber|coral|violet`** (v3.4)
   - Inputs: `.input`, `.input--invalid`
   - Badges: `.badge`, `.badge--navy|success|warning|danger|info|mono`
   - Status wrappers (v3.4): **`.state`**, **`.state__title`**, **`.state--success|danger|warning|info`**
   - Other: `.alert`, `.codeblock`, `.tabs__list`, `.tabs__trigger`, `.rail`, `.rail__fill`, `.dialog`, `.marquee`, `.cta`, `.cta--dark`

  Token families exposed on `:root` that components consume directly:
   - Surface tints (v3.3): `--surface-cream|ivory|navy|lime|amber|violet|coral`
   - AAA status inks (v3.4): `--ink-success|danger|warning|info|accent`
   - Radii: `--radius-none|xs (2) |sm (3) |md (6) |lg (10) |pill`

3. **CIA disc** — always via `<CIABadge>` from `@/components/learning/CIABadge`.
   See §9.4b.

### Forbidden

- Hand-rolling an inline `<div className="rounded border p-4 bg-white shadow">` — this is a `.card` with extra steps. Use `.card`.
- Inline `<button className="px-4 py-2 bg-blue-600 text-white rounded">` — this is a `.btn--main`. Use it.
- Copy-pasting a partial set of Tailwind utilities that happens to look like a chip — use `.chip` or `<Badge>`.
- Creating a file in `src/components/learning/<name>.tsx` that re-implements something already in the approved list (e.g. a "ConceptCard" that wraps a `<div>` with border + shadow). Reject this in review.
- Adding a new global utility class in `neobrutalism.css` without an RFC through the designer agent.
- Using Tailwind raw colour utilities (`bg-blue-600`, `text-orange-500`, `border-gray-300`) on anything shipping to a user. The five major-palette tokens (`navy`, `lime`, `saffron`/`amber`, `violet`, `coral`) and the semantic tokens (`--color-main`, `--color-fg`, etc.) are the complete colour vocabulary. Exception: `<CIABadge>`'s three internal fills (blue/green/amber) — these live inside the component and are the only allowed exceptions.

### How to extend the system safely

If you hit a case the approved set doesn't cover:

1. First, try to compose from existing classes (e.g. `.card .card--lime-tint .card--compact` before reaching for inline CSS).
2. If a variant is genuinely missing, propose it in the designer agent's queue. The designer either (a) approves → adds the variant to `neobrutalism.css` **and** documents it here in §9, or (b) rejects → you compose from what exists.
3. Never ship the bespoke code to a feature branch ahead of the designer sign-off.

### Enforcement

CI runs these greps on learning pages; any match fails:

```bash
# Inline style overrides of things tokens already cover
grep -rn "boxShadow: [\"']" src/app/\\(learning\\) src/components/learning | grep -v "var(--shadow"

# Bespoke card patterns
grep -rnE "className=\\\"[^\\\"]*rounded-(md|xl|2xl)[^\\\"]*border[^\\\"]*(bg|shadow)" src/app/\\(learning\\) src/components/learning

# Raw Tailwind colour utilities
grep -rnE "(bg|text|border)-(red|orange|amber|yellow|lime|green|emerald|teal|cyan|sky|blue|indigo|violet|purple|fuchsia|pink|rose|gray|zinc|slate|neutral|stone)-[0-9]+" src/app src/components/learning | grep -v "src/components/ui"
```

All three must print zero matches before a PR merges.

---

## 1. Design principles (v3.4)

1. **White canvas, tinted cards**. The page canvas is `#FFFFFF`. Cards sit
   on top in one of the palette-tinted surfaces (`--surface-cream` default,
   optionally `--surface-navy|lime|amber|violet|coral`). **Never** paint
   a card with pure white — it disappears against the canvas.
2. **Navy is the brand**. One accent — navy `#001B5E`. Primary CTA fill,
   active chip fill, shadow colour, dark hero card.
3. **Hairline borders**. 1 px default on every surface; 2 px only on the
   single most important CTA; 3 px reserved for dialog / modal.
4. **Vertical navy shadow**. `0 5px 0 0 var(--color-navy)`. Never black,
   never diagonal, never blurred. Hover translates `(0, 2px)` + reduces
   shadow; active translates `(0, 5px)` + removes shadow.
5. **Tight corners** (v3.2+, reinforced v3.9 §5.2a). `--radius-md: 6`
   is the default for **every** button, input, chip, single-line box,
   small card. `--radius-lg: 10` is **hero-only**. `--radius-sm: 3`
   for dense table tags, `--radius-xs: 2` for `.tag`. Circular shapes
   (`.cia`, `.step`, nav chips, dots) use `--radius-pill`.
   **No in-between values** (8, 12, 14, 20 px) — Tailwind's
   `rounded-lg` on a small control, or inline `"0.5rem" / "0.625rem" /
   "0.75rem"` literals, are banned. Anything that needs to look "round"
   must be pill.
6. **Restrained type**. Heading weight **500**, body **400–500**. Never bold.
   Size contrast (not weight contrast) carries hierarchy.
7. **Bright accent pops only as status signal**. `--color-lime/saffron/coral/violet`
   are reserved for status meaning. When used on light backgrounds they
   are NEVER text colour (contrast fails) — use `--ink-success/danger/warning/info`
   for the text, and `--surface-*` for the background. `.state state--*`
   pairs them automatically.
8. **CIA letters = circular discs**. Any C / I / A in copy renders through
   `<CIABadge>` with reserved colours (C blue, I green, A amber). No other
   rendering is allowed.
9. **Compact but airy**. Section rhythm `clamp(3rem, 2.25rem + 3vw, 5rem)`.
   Container inner padding 32 px (≥ md) so grid first/last columns breathe
   wider than the card-to-card gap.

### 1.1 What got removed from v2
- Asymmetric radius (`--radius-asymmetric`) — dropped; does not match 9thave.
- `--color-paper` as bg — relegated to `--color-ash` subtle-stripe use only.
- 3-tone intro entry cards (lime / violet / saffron) — replaced with
  navy hero + white cards with numeric eyebrows.
- `translate(2px, 2px)` diagonal hover — replaced with `translate(0, 3px)`
  vertical press.

---

## 2. Colour palette

### 2.1 Primitives

```css
:root {
  /* Neutral spine */
  --color-white:       #FFFFFF;
  --color-ash:         #F3F3F3;   /* subtle section band only */
  --color-ash-2:       #F1EEF3;   /* alt subtle band — 9thave observation */
  --color-line:        #E5E7EB;   /* hairline divider */
  --color-ink:         #000000;
  --color-ink-soft:    #18181B;   /* zinc-900 — 9thave observed */

  /* Brand */
  --color-navy:        #001B5E;   /* primary accent, 9thave brand */
  --color-navy-soft:   #22356F;   /* hover shade */
  --color-navy-deep:   #000D33;   /* pressed shade */

  /* Support colours — status signalling only */
  --color-lime:        #B9FF66;   /* success / highlight */
  --color-saffron:     #FFDC58;   /* warning */
  --color-coral:       #FF6B6B;   /* danger */
  --color-violet:      #C4A1FF;   /* info */

  /* Muted text */
  --color-graphite:    #3A3F4B;   /* body muted */
  --color-smoke:       #4B5563;   /* meta, captions — gray-600 since v3.9 (was gray-500 #6B7280) */
}
```

`--color-smoke` was lifted from gray-500 (#6B7280, 4.66 : 1 on white) to
gray-600 (#4B5563, 7.0 : 1 on white, ≥ 5 : 1 on every pale tint in §2.5)
in v3.9 so every consumer of `--color-fg-subtle` and `.eyebrow` clears
WCAG AA on every surface the design system ships.

### 2.2 Semantic (consumed by components)

```css
:root {
  --color-bg:           var(--color-white);
  --color-bg-subtle:    var(--color-ash);       /* optional section band */
  --color-bg-elevated:  var(--color-white);
  --color-bg-inverse:   var(--color-navy);

  --color-fg:           var(--color-ink);
  --color-fg-muted:     var(--color-graphite);
  --color-fg-subtle:    var(--color-smoke);
  --color-fg-inverse:   var(--color-white);

  --color-border:       var(--color-ink-soft);  /* 1 px hairline on light */
  --color-border-navy:  var(--color-navy);      /* navy on navy surfaces */
  --color-line:         var(--color-line);      /* ultra-subtle divider */

  --color-main:         var(--color-navy);
  --color-main-fg:      var(--color-white);
  --color-main-hover:   var(--color-navy-soft);
  --color-main-active:  var(--color-navy-deep);

  /* v3.9 — semantic status tokens rebound to the AAA "ink" tone of
     each hue. Binding these to the bright primitives failed WCAG AA as
     text colour (e.g. lime `#B9FF66` on white = 1.5 : 1). The bright
     fills remain available through the parallel `*-fill` token. */
  --color-success:      var(--ink-success);      /* #14532D deep green text */
  --color-success-fill: var(--color-lime);       /* opt-in — saturated fill / border */
  --color-warning:      var(--ink-warning);      /* #78350F deep amber text */
  --color-warning-fill: var(--color-saffron);
  --color-danger:       var(--ink-danger);       /* #7F1D1D deep red text */
  --color-danger-fill:  var(--color-coral);
  --color-info:         var(--ink-info);         /* #3730A3 deep indigo text */
  --color-info-fill:    var(--color-violet);

  --color-ring:         var(--color-navy);      /* focus is navy, not black */
  --color-overlay:      rgba(0, 13, 51, 0.72);
}
```

The AAA `--ink-*` primitives used by `--color-success` etc. are defined
in §9.4a-3 "AAA text-safe darks for status signalling". The split lets
inline code like `color: var(--color-success)` pass contrast
automatically — no demo component has to remember which token is
safe on which surface.

### 2.3 Dark mode

```css
:root[data-theme="dark"] {
  --color-bg:           var(--color-ink-soft);
  --color-bg-subtle:    #22252D;
  --color-bg-elevated:  #000;
  --color-bg-inverse:   var(--color-white);

  --color-fg:           #E8EAED;
  --color-fg-muted:     #A1A6B0;
  --color-fg-subtle:    #6B7280;

  --color-border:       #3A3F4B;
  --color-border-navy:  var(--color-navy);

  --color-main:         var(--color-lime);      /* invert: lime works on dark */
  --color-main-fg:      var(--color-ink);
}
```

### 2.5 Card surface tints (v3.3 — MANDATORY)

**Rule**: `.card` backgrounds must **not be pure white** (`#FFFFFF`). Pure
white on a white page canvas creates a "floating rectangle with shadow"
that reads as generic SaaS. Neobrutalism wants cards to feel *tinted* —
a flat wash of colour that stays in the brand's hue family.

Every card tint sits at **L ≥ 0.95** so ink-black text still hits AAA
contrast. The hue range is locked to the five major palette hues — no
invented colours.

```css
/* Ready-to-use surface tokens (src/styles/neobrutalism.css) */
--surface-cream:   #F7F9F0;  /* DEFAULT  — neutral cream with lime undertone */
--surface-ivory:   #F4F2E9;  /* alternate neutral, warmer */
--surface-navy:    #EDF0F7;  /* navy hue @ L 0.96 */
--surface-lime:    #EFF7D9;  /* lime hue @ L 0.96 */
--surface-amber:   #FDF3CF;  /* amber hue @ L 0.96 */
--surface-violet:  #EEE4FA;  /* violet hue @ L 0.95 */
--surface-coral:   #FEECE5;  /* coral hue @ L 0.97 */
```

Hue-family derivation (OKLCH): each surface shares the exact **hue** of
its major-palette sibling but lifts lightness to 0.95–0.97 and clamps
chroma to 0.02–0.04. This keeps the page monochromatic in feel — every
surface is discernibly *in the palette family* rather than random pastel.

Usage:

| Class | Purpose | When to use |
|---|---|---|
| `.card` (default) | cream | every "plain" content card — 95 % of cards |
| `.card--navy-tint` | pale navy | factual tables, system status panels |
| `.card--lime-tint` | pale lime | success / integrity callouts |
| `.card--amber-tint` | pale amber | warning / availability callouts |
| `.card--violet-tint` | pale violet | info / further reading |
| `.card--coral-tint` | pale coral | danger / misuse examples |
| `.card--ghost` | flat ash | inline ghost (no shadow, no border) |
| `.card--ivory` | warmer neutral | prose-heavy long-form sections |
| `.card--dark` | navy | hero feature card (1 per page max) |
| `.card--plain` | pure white | **rare opt-in**; only if a photo/chart needs true-neutral backing |

Anti-pattern: `style={{ background: "#FFFFFF" }}` on a card. If the page
canvas is white and the card is white, the card reads as non-existent.

### 2.4 Contrast verification

| Pair | Ratio | Use |
|---|---|---|
| `--color-ink` on `--color-white` | 21 : 1 | body |
| `--color-graphite` on `--color-white` | 10.8 : 1 | muted |
| `--color-smoke` on `--color-white` | 7.0 : 1 (v3.9) | captions + eyebrow on every tint — AA on all `--surface-*` |
| `--color-white` on `--color-navy` | 15.3 : 1 | CTAs, navy panels |
| `--color-ink` on `--color-lime` | 14.5 : 1 | success badges |

---

## 3. Typography

### 3.1 Stack

```css
:root {
  /* Pretendard is the sole primary. It carries Korean + Latin in a
     single variable font (45–920 weight axis). System + legacy Latin
     grotesques remain only as structural fallbacks. */
  --font-display: "Pretendard Variable", "Pretendard", -apple-system,
                  BlinkMacSystemFont, "Space Grotesk", "Inter",
                  system-ui, sans-serif;
  --font-sans:    "Pretendard Variable", "Pretendard", -apple-system,
                  BlinkMacSystemFont, "Inter", "Space Grotesk",
                  system-ui, sans-serif;
  --font-mono:    "JetBrains Mono", ui-monospace, "SF Mono", Menlo,
                  Consolas, monospace;

  --font-weight-regular: 400;   /* body @ base size */
  --font-weight-medium:  500;   /* body @ small, headings, buttons */
  --font-weight-strong:  500;   /* reserved alias — never 600+ */
}
```

Pretendard is loaded from jsDelivr via `<link>` in
`src/app/layout.tsx`. To self-host instead, install the `pretendard`
npm package and import `pretendard/dist/web/variable/pretendardvariable.css`
from `src/styles/globals.css`. The stack is unchanged either way.

**Rule**: no weight above 500. Emphasis comes from size and colour, not
weight. `<b>`/`<strong>` elements keep weight 500; use size jump + colour
swap if more emphasis is needed.

### 3.2 Scale

```css
:root {
  --text-xs:      clamp(0.8125rem, 0.78rem + 0.15vw, 0.875rem);   /* 13–14 */
  --text-sm:      clamp(0.9375rem, 0.9rem + 0.15vw, 1rem);         /* 15–16 */
  --text-base:    clamp(1rem,    0.96rem + 0.2vw, 1.0625rem);      /* 16–17 */
  --text-lg:      clamp(1.125rem, 1.05rem + 0.3vw, 1.25rem);       /* 18–20 */
  --text-xl:      clamp(1.375rem, 1.25rem + 0.5vw, 1.625rem);      /* 22–26 */
  --text-2xl:     clamp(1.75rem,  1.5rem + 1vw,  2.25rem);         /* 28–36 */
  --text-3xl:     clamp(2.25rem,  1.9rem + 1.4vw, 3rem);           /* 36–48 */
  --text-display: clamp(2.5rem,   1.75rem + 3.4vw, 4.25rem);       /* 40–68 */

  --leading-display: 1.0;       /* tight — 9thave h1 is 60/60 line-height */
  --leading-heading: 1.15;
  --leading-body:    1.55;
  --leading-prose:   1.7;

  --tracking-normal: 0;
  --tracking-tight: -0.01em;    /* h1 only, subtle */
  --tracking-caps:  0.04em;     /* UPPERCASE eyebrow */
}
```

### 3.3 Hierarchy

| Role | Size | Weight | Family | LH |
|---|---|---|---|---|
| h1 | `--text-display` | 500 | display | 1.0 |
| h2 | `--text-2xl` | 500 | display | 1.15 |
| h3 | `--text-xl` | 500 | sans | 1.25 |
| h4 | `--text-lg` | 500 | sans | 1.35 |
| body | `--text-base` | **400** | sans | 1.55 |
| small | `--text-sm` | 500 | sans | 1.5 |
| eyebrow | `--text-xs` | 500 | mono | 1.4 UPPERCASE + tracking-caps |

- Body at base size uses weight 400 (matches 9thave body). Drops to 500
  below `--text-sm` (`14 px`) for sub-pixel legibility.
- Display `line-height: 1.0` is tight and deliberate; headings
  `text-wrap: balance`.

---

## 4. Spacing

```css
:root {
  --space-1:  0.25rem;    /*  4 */
  --space-2:  0.5rem;     /*  8 */
  --space-3:  0.75rem;    /* 12 */
  --space-4:  1rem;       /* 16 */
  --space-5:  1.25rem;    /* 20 */
  --space-6:  1.5rem;     /* 24 */
  --space-7:  1.75rem;    /* 28 */
  --space-8:  2rem;       /* 32 */
  --space-10: 2.5rem;     /* 40 */
  --space-12: 3rem;       /* 48 */
  --space-16: 4rem;       /* 64 */
  --space-20: 5rem;       /* 80 */
  --space-section: clamp(3rem, 2.25rem + 3vw, 5rem);   /* 48–80 */
  --container-gut: clamp(1rem, 0.75rem + 1vw, 2rem);
}
```

- **Section rhythm** = `--space-section`. Between sibling blocks inside a
  section use `--space-10`.
- **Card padding** default `--space-8`; dense dashboards `--space-5`.
- Prefer padding with `--space-5` / `--space-8` for buttons (9thave used
  `padding: 20px 36px` — our base btn targets `padding: 0 var(--space-7)`
  on a 44px height).

### 4a. Horizontal gap density table (v3.11, from guide)

`.stack` in §7.5 covers the *vertical* rhythm between sibling cards.
For *horizontal* gaps inside `flex` or `grid` rows pick the token by
layout density:

| Density | Example | Token | Value |
|---|---|---|---|
| Dense dashboard (table-like chip rows) | L1–L5 strength ladder, `ssh-keygen` field list | `--space-2` | 8 px |
| Standard list / inline chip row | Part-chip group, CIA badge row | `--space-3` | 12 px |
| Card grid | 3-up "what I know / have / am" | `--space-4` | 16 px |
| Hero / bento | Intro top 3-up, trust-model 3-column | `--space-6` | 24 px |

Rule: `gap-[var(--space-1)]` (4 px) is banned between shadow-bearing
cards — see the shadow-breathing rule in §7.5. Pick from this table
before inventing a new Tailwind arbitrary value.

---

## 5. Borders, radius, shadow

### 5.1 Border widths

```css
:root {
  --border-hair:   1px;   /* default on everything */
  --border-base:   1px;   /* alias — kept for v2 consumers */
  --border-strong: 2px;   /* emphasized primary CTA or hero hero card */
  --border-heavy:  3px;   /* dialog/modal only */
}
```

Rule: 1 px is the default. Reach for 2 px only when the element is the
single most important focus target on the screen.

### 5.2 Radius — tight, closer to sharp (v3.2)

```css
:root {
  --radius-none: 0;
  --radius-xs:   2px;     /* inner chip rectangles, dividers */
  --radius-sm:   3px;     /* input, small tag */
  --radius-md:   6px;     /* card, button — DEFAULT */
  --radius-lg:   10px;    /* hero card, large panel */
  --radius-pill: 9999px;  /* dots, CIA discs, step markers, progression chips */
}
```

User briefs:
- "corner radius = half of the9thave, closer to sharp" (v3.0)
- "곡률이 커지면 너무 못생겨보여" (v3.2) — further reduced.

9thave ships 16 / 24 px. We now ship 6 / 10 — **less than half**, deliberately
pulling toward sharp. Anything ≥ 12 reads as "too round" and is banned except
for full pills.

**Rule**: squares that look square (card, button, input, table cell) use
`--radius-md` max. Fully circular elements (step marker, CIA disc, nav chip)
use `--radius-pill`. There is **no legitimate use for an in-between value
like 14 or 20 px** — if something needs to "look rounder" make it pill.

### 5.2a Control radii — HARD RULE (v3.9)

A single-line text container — `<input>`, `<button>`, chip, tag,
numbered cell, short demo box — must render with **one** of only two
radii:

| Shape | Token | Value | Use on |
|---|---|---|---|
| Near-sharp control | `--radius-md` | **6 px** | buttons, inputs, chips, tag-like boxes, 1-line highlight boxes, simulator cells — **default** |
| Small label | `--radius-sm` | **3 px** | tags inside dense tables, chip rectangles that must look sharper than a button |
| Pill / circle | `--radius-pill` | **9999 px** | dots, CIA discs, step markers, nav chips, progress rails, avatars |

`--radius-lg` (10 px) is reserved **only** for genuine full-page hero
cards (the top intro card on `/intro`, the dark navy hero on `/trust`).
Using it on a button / input / small card is an explicit violation.

**Banned patterns** in any cryptography / SSH / PIPA / web-app simulator
or learning page:

```
❌ className="... rounded-lg ..."         on <button>, <input>, 1-line <span>, or any card narrower than 600 px
❌ className="... rounded-xl ..."         anywhere in src/app or src/components/learning
❌ style={{ borderRadius: "0.5rem"    }}  → use "var(--radius-md)"
❌ style={{ borderRadius: "0.625rem"  }}  → use "var(--radius-md)"
❌ style={{ borderRadius: "0.75rem"   }}  → use "var(--radius-md)"
❌ style={{ borderRadius: "0.375rem"  }}  → use "var(--radius-md)"  (equivalent literal; token preferred for auditability)
❌ style={{ borderRadius: "0.25rem"   }}  → use "var(--radius-sm)"
```

Exception — geometric per-corner values (`"6px 6px 0 0"` on the top row
of a stacked set of nodes) are permitted when the design intent is
"round the outer edge of the whole stack, not each cell". Keep the
pixel value equal to the corresponding token so the reader can still
audit at a glance. Don't invent a third radius.

**CI grep** to enforce (should return zero hits in `src/app/(learning)`
and `src/components/learning/*`):

```bash
grep -rnE "rounded-(lg|xl|2xl|3xl)" src/app/\(learning\) src/components/learning
grep -rnE 'borderRadius:\s*"(0\.5|0\.625|0\.75|0\.375|0\.25)rem"' src/app/\(learning\) src/components/learning
```

Rationale: v3.2's "곡률이 크면 너무 못생겨" brief — small controls at
10 px + cards at 6 px reads as visually noisy. Standardising every
control to 6 px makes the page feel authored by one hand. Pill / circle
retains its expressive, specifically-communicative role.

### 5.3 Shadow — navy offset

```css
:root {
  --shadow-none:   none;
  --shadow-xs:     0 2px 0 0 var(--color-navy);
  --shadow-sm:     0 3px 0 0 var(--color-navy);
  --shadow-base:   0 5px 0 0 var(--color-navy);   /* DEFAULT — 9thave signature */
  --shadow-lg:     0 6px 0 0 var(--color-navy);
  --shadow-xl:     0 8px 0 0 var(--color-navy);

  /* Optional black variant — only when the element is ON a navy background */
  --shadow-base-ink: 0 5px 0 0 var(--color-ink-soft);
}
```

Every shadow is **vertical-only** (`0 Ypx 0 0`), navy-tinted. Diagonal
`Xpx Ypx` offsets from v2 are banned.

### 5.4 Hover-press pattern (vertical)

```css
.pressable {
  transform: translate(0, 0);
  box-shadow: var(--shadow-base);
  transition: transform 120ms cubic-bezier(0.2,0,0,1),
              box-shadow 120ms cubic-bezier(0.2,0,0,1);
}
.pressable:hover  { transform: translate(0, 2px); box-shadow: var(--shadow-sm); }
.pressable:active { transform: translate(0, 5px); box-shadow: var(--shadow-none); }
```

The motion reads as "the button sinks straight down" rather than the
diagonal "slam" of pure neobrutalism. More studio-appropriate.

---

## 6. Layout — gutter-locked against 9thave

The gutter system is calibrated from live measurement of the9thave.com at
1440 × 900 viewport:

- outer section padding = **20 px** (`px-5`)
- inner content max-width = **1240 px** (`max-w-[1240px] mx-auto`)
- marquee / full-bleed band = **1280 px** (`max-w-[1280px]`)
- header / nav padding = 32 px inside the outer section

### 6.1 Container tokens

```css
:root {
  --container-sm:  40rem;     /*  640 — long-form reading (articles) */
  --container-md:  64rem;     /* 1024 — dashboard / compact */
  --container-lg:  77.5rem;   /* 1240 — **default marketing / learning** */
  --container-xl:  80rem;     /* 1280 — wide band (marquee, full-bleed row) */
}
```

**v3 change (2026-04-13)**: `--container-lg` was **72 rem (1152 px)** — too
narrow vs 9thave's 1240. The 8 % extra width restores the intended rhythm
and the h1 reads at the right ratio against the viewport.

### 6.2 Outer gutter (inner container padding)

```css
:root {
  --gutter-xs: 1rem;       /* 16  — mobile */
  --gutter-sm: 1.25rem;    /* 20  — small */
  --gutter-md: 2rem;       /* 32  — **desktop default, matches 9thave `px-8` nav** */
  --gutter-lg: 2.5rem;     /* 40  — wide special-case */
  --container-gut: clamp(var(--gutter-xs), 0.5rem + 2vw, var(--gutter-md));
}
```

**Grid symmetry rule (important)**: the container's `padding-inline` must
be ≥ the in-grid `gap` so the first and last column breathe *more* than
the gap between cards. v3 ships 32 px inner padding + 20 px grid gap →
first-card-left breathing (32 px) > inter-card gap (20 px). Asymmetric in
the right way — the outer edges feel framed, the inner gap feels rhythmic.

Anti-pattern observed during testing: inner padding < gap makes the first
and last cards appear "glued" to the column edge while cards in the middle
float with air around them. If you see a 3-up grid where the first/last
column hugs the edge, **this is the fix** — widen the container padding,
not the grid gap.

Learning-page layout wrapper rule:

```tsx
<div
  style={{
    marginInline: "auto",
    maxWidth: "var(--container-lg)",
    paddingInline: "var(--container-gut)",
  }}
>
  {children}
</div>
```

Or prefer the `.container` utility class shipped in
`src/styles/neobrutalism.css`:

```tsx
<div className="container">{children}</div>
```

Do **not** mix `mx-auto` + `w-full` Tailwind utilities on the same element
— `w-full` forces `width:100%` and defeats the centring margins; the
centring then silently collapses to `0 / (viewport − max-width)` and the
page lists to the left. Use `.container` (which uses explicit
`margin-inline: auto`) or `mx-auto` **without** `w-full`.

### 6.3 Gutter math at reference viewports (v3.1 update)

| Viewport | Content bbox | Inner pad | Outer gutter | Usable content | First-card left breathing |
|---|---|---|---|---|---|
| 1440 | 1240 | **32** | 100 | 1176 | 132 (outer 100 + inner 32) |
| 1280 | 1240 | 32    | 20  | 1176 |  52 |
| 1024 | 1024 | 28    |  0  |  968 |  28 |
|  768 |  768 | 23    |  0  |  722 |  23 |
|  640 |  640 | 19    |  0  |  602 |  19 |
|  414 |  414 | 16    |  0  |  382 |  16 |
|  360 |  360 | 16    |  0  |  328 |  16 |

Breakpoints to visually regress: **360, 414, 768, 1024, 1280, 1440, 1920**.

### 6.4 Hero h1 cap

Observed: 9thave h1 = 60 px @ weight 500 @ line-height 60 on 1440 viewport.
Our old clamp computed to 72 px at 1440 → too large. New clamp:

```css
--text-display: clamp(2.25rem, 1.2rem + 2.8vw, 3.75rem);  /* 36 → 60 */
```

Hero vertical cap still 70 vh; the hero feature card on the right is
`--container-lg` × 5/12 column share.

### 6.5 Grid gap defaults

- 3-up card grid: `gap: var(--space-5)` (20 px — matches 9thave cards).
- 2-up hero + sidecar: `gap: var(--space-6)` (24 px).
- Dense list items: `gap: var(--space-3)` (12 px).

---

## 7. Motion

```css
:root {
  --duration-instant: 80ms;
  --duration-fast:    120ms;
  --duration-normal:  200ms;
  --ease-snap:        cubic-bezier(0.2, 0, 0, 1);
  --ease-out:         cubic-bezier(0.16, 1, 0.3, 1);
}
```

Animating transforms + opacity only. Section entrances get a 200 ms
slide-up + fade at most. Respect `prefers-reduced-motion`.

---

## 7a. Button + chip neobrutalism — HARD RULE (v3.10)

Every interactive control on every learning-page surface (cryptography,
SSH, PIPA, Web App, VPN) must render with the neobrutalism signature:

```
1 px ink-soft border + 5 px vertical navy shadow + 2 px translate on :hover + 5 px translate on :active
```

This is **non-negotiable**. A flat `bg-navy text-white rounded-md px-4 py-2`
button has the same pressable affordance as a static chip — the user can't
tell them apart. Neobrutalism communicates "pressable" through the drop
shadow + border pair.

### 7a.1 Button API

Always use the `.btn` utility. Never hand-roll a button from Tailwind
atoms.

```tsx
/* ❌ BANNED — indistinguishable from a colored chip */
<button className="px-4 py-2 bg-bg-inverse text-fg-inverse rounded-sm text-sm font-medium hover:opacity-90">
  암호화
</button>
<button className="px-5 py-2.5 rounded-md text-sm font-medium"
        style={{ background: "var(--color-accent)", color: "var(--color-bg)" }}>
  저장 값 계산
</button>

/* ✅ REQUIRED */
<button className="btn btn--sm btn--main">암호화</button>
<button className="btn btn--sm btn--main">저장 값 계산</button>
```

Variants — pick exactly one per role:

| Variant class | Use for | Example |
|---|---|---|
| `.btn .btn--sm .btn--main` | primary action (submit, compute, encrypt, generate key) | "저장 값 계산", "🔑 Ed25519 키쌍 생성" |
| `.btn .btn--sm .btn--danger` | destructive / attack button | "🚨 DB 유출!", "1비트 변조 →" |
| `.btn .btn--sm .btn--outline` | secondary (sample loader, undo, toggle-between) | "샘플 PEM 불러오기", "다시 시작" |
| `.btn .btn--sm .btn--ghost` | tertiary, reset-style, inline `text-xs` actions | "↺ 다시 시도", "복사" |
| `.btn .btn--sm` (no variant) | neutral white-fill alternative | step-complete confirmation pills |

Only add `style={...}` when per-instance color is semantic and can't be
expressed by a variant (e.g. CA-signature red from data, 5-cost heat-map
color). Never use `style` to recreate padding / height / border-radius —
those already live in the utility.

### 7a.2 Input API

Pair `.btn` with `.input` so the row heights match (44 px default, 36 px
when both carry `--sm`):

```tsx
/* ❌ BANNED */
<input className="w-full rounded-md px-3 py-2 text-sm font-mono"
       style={{ background: "var(--color-surface-elevated)",
                border: "1px solid var(--color-border)" }} />

/* ✅ REQUIRED */
<input className="input font-mono" />
```

### 7a.3 Tiny-icon variant override

When a control genuinely needs to fit inline next to `text-xs` copy
(e.g. key "보기"/"숨기기" toggle, "복사" next to a hash hex), keep the
utility shell and override only `height / padding / fontSize`:

```tsx
<button
  className="btn btn--sm btn--outline"
  style={{
    height: "1.75rem",
    padding: "0 var(--space-3)",
    fontSize: "var(--text-xs)",
  }}
>
  클릭하여 보기
</button>
```

The border + shadow are preserved so the affordance stays readable.

### 7a.4 Chip — `.chip` utility

A `<span>` that looks like a pill but is **not interactive** is a chip,
not a button. Chips use the `.chip` utility with `min-height` + vertical
padding so long labels never clip. Banned: `px-2 py-0.5 rounded-full
text-xs` — the 2 px vertical padding clips Hangul descender strokes on
small viewports.

```tsx
/* ❌ BANNED — text can clip on 320 px viewport */
<span className="text-xs font-mono px-2 py-0.5 rounded-full"
      style={{ background: "...", color: "..." }}>
  PKI
</span>

/* ✅ REQUIRED */
<span className="chip chip--sm tag--violet"
      style={{ fontFamily: "var(--font-mono)",
               borderRadius: "var(--radius-pill)" }}>
  PKI
</span>
```

### 7a.5 Tab-group chips

A row of two-or-more "mode" pills that are clickable and mutually
exclusive (e.g. `no-pfs` vs `pfs`) uses the shared `.tabs__list /
.tabs__trigger` utilities, not ad-hoc chip styles:

```tsx
<div className="tabs__list w-full" role="tablist">
  {modes.map(m => (
    <button role="tab" aria-selected={m === active}
            className="tabs__trigger flex-1">{m}</button>
  ))}
</div>
```

### 7a.6 CI enforcement

These greps must return zero hits in `src/app/(learning)` and
`src/components/learning` / `src/components/demos`:

```bash
# Hand-rolled buttons
grep -rnE 'className="[^"]*px-[345]\s+py-[0-9.]+[^"]*rounded-(sm|md)[^"]*"' \
  src/app/\(learning\) src/components/learning src/components/demos | grep -i button

# Ad-hoc rounded-full pills outside the .chip utility
grep -rnE 'px-[23] py-(0\.5|1)\s+rounded-full' \
  src/app/\(learning\) src/components/learning src/components/demos
```

Rationale: every time the cryptography page ships a hand-rolled button
it regresses this rule. A flat fill without border/shadow reads as
"chip" and the user can't tell what's tappable. The utilities enforce
the visual affordance.

---

## 7.4 Nested-card anti-pattern (v3.2)

A `.card` already supplies `padding: var(--space-8)` (32 px). Wrapping a
second layer of `p-6` (24 px) children inside it creates double padding
(total 56 px inset) which crushes the usable column width — especially on
3-up flow cards observed on /auth.

**Rule**: when a `.card` hosts a grid of sub-panels divided by borders,
the outer card **must zero its padding** and the inner cells carry the
padding themselves.

```tsx
/* BAD */
<div className="card">
  <div className="grid grid-cols-3">
    <div className="p-6 space-y-3"> ... </div>
    ...
  </div>
</div>

/* GOOD */
<div className="card" style={{ padding: 0, overflow: "hidden" }}>
  <div className="grid grid-cols-3">
    <div
      style={{
        padding: "var(--space-6)",
        borderRight: "var(--border-hair) solid var(--color-line)",
      }}
    >
      ...
    </div>
    ...
  </div>
</div>
```

Same rule for `<details>` / `<summary>` cards: the summary row gets its own
padding, the outer `.card` drops to `padding: 0`.

---

## 7.5 Section rhythm (v3.2 — use `.stack`, not `space-y-*`)

Hand-rolled `space-y-6`, `space-y-16` on every page produces inconsistent
vertical rhythm. v3 ships three stack utilities — use them exclusively on
learning pages:

```css
/* v3.10 — lifted from v3.6 (20/8/24) so the 5 px navy shadow always
   breathes. At 8 px tight, the shadow of card N merged visually with
   the top border of card N+1 and read as a second border. */
.stack        { display: flex; flex-direction: column; gap: var(--space-6); }   /* 24 px */
.stack--tight { gap: var(--space-3); }                                           /* 12 px */
.stack--loose { gap: var(--space-8); }                                           /* 32 px */
```

**Shadow-breathing rule (v3.10 hard rule).** Every `.card` draws a 5 px
vertical navy shadow. To stay a silhouette and not a double border,
the gap below it must be **≥ 10 px**. In practice:

- Inside `.stack` (24 px), `.stack--tight` (12 px), `.stack--loose` (32 px) — always safe.
- Inside `grid gap-4` (16 px) or `grid gap-6` (24 px) — safe.
- Inside `grid gap-2` (8 px) — **banned** between stacked cards; drop to two-column max or lift to `gap-3`.
- Adjacent `.card` siblings without a parent flex/grid gap — must add `margin-top: var(--space-3)` explicitly, or wrap them in a `.stack`.

CI grep to catch regressions:

```bash
grep -rnE 'gap-(1|2)\b.*grid-cols' src/app/\(learning\) src/components/learning
```

Pattern on a page body:

```tsx
<div className="stack stack--loose">    {/* top-level: 32 px between sections (v3.10) */}
  <section className="stack stack--tight">  {/* header group: 12 px (v3.10) */}
    <span className="eyebrow">PART N · Title</span>
    <h1>...</h1>
    <p className="lede">...</p>
  </section>
  <section className="stack stack--tight"> ... </section>
  <section className="stack"> ... </section>        {/* default: 24 px (v3.10) */}
</div>
```

Anti-pattern — mixing `space-y-5` / `space-y-6` / `space-y-16` on siblings
of the same page produces a jagged vertical rhythm.

---

## 7.6 Dark-surface contrast cascade (v3.1)

When content sits on a navy surface (`.card--dark`, `.cta--dark`, nav hero)
the generic `color: inherit` reads as pure white for large text but must
drop to semi-transparent white for secondary copy. The neobrutalism CSS
ships this cascade automatically:

```css
.card--dark :where(.eyebrow)        { color: rgba(255,255,255,0.72); }  /* ≥14 px only */
.card--dark :where(p, li)           { color: rgba(255,255,255,0.88); }  /* AAA on navy */
.card--dark :where(h1, h2, h3, h4)  { color: #FFFFFF; }
.card--dark :where(a)               { color: var(--color-lime); text-decoration: underline; }
.card--dark :where(.chip)           { color: #FFFFFF; border-color: rgba(255,255,255,0.4); }
```

Tokens for explicit use when a `:where` selector cannot reach (for example
inside a portaled popover):

```css
--color-fg-inverse-muted:  rgba(255, 255, 255, 0.88);  /* paragraph on navy */
--color-fg-inverse-subtle: rgba(255, 255, 255, 0.72);  /* eyebrow on navy ≥ 14 px */
```

**Anti-pattern**: setting `color: rgba(255,255,255,0.65)` on an eyebrow —
fails AA at 12–13 px. Use the `--color-fg-inverse-subtle` token (0.72)
and ensure eyebrow font-size is ≥ 14 px.

---

## 8. Accessibility

```css
:focus-visible {
  outline: 2px solid var(--color-ring);
  outline-offset: 3px;
  border-radius: var(--radius-sm);
}
```

- Outline 2 px (navy) + 3 px offset — visible on white without
  overpowering the 1 px element border.
- `aria-label` required on icon-only buttons.
- Touch targets 44 × 44 primary / 36 × 36 dense.
- `prefers-reduced-motion` drops transforms; shadow removes via `:hover`
  still gives affordance.

---

## 9. Component recipes (v3)

All recipes are in [`src/styles/neobrutalism.css`](../src/styles/neobrutalism.css);
the names below are conceptual. The CSS file is the authoritative source.

### 9.1 Button

```css
.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-2);
  height: 2.75rem;
  /* Physical longhand (v3.10) — Tailwind v4 preflight emits `padding-*`
     per-side inside @layer base. Our logical shorthand (`padding: 0 X`)
     lost the cascade on Chromium even at higher specificity, so we
     match Tailwind's physical shape. See §9.4 for the full cascade
     note. */
  padding-top: 0;
  padding-bottom: 0;
  padding-left: var(--space-7);
  padding-right: var(--space-7);
  font-family: var(--font-sans);
  font-size: var(--text-base);
  font-weight: var(--font-weight-medium);   /* 500 */
  line-height: 1;
  color: var(--color-fg);
  background: var(--color-white);
  border: var(--border-hair) solid var(--color-ink-soft);
  border-radius: var(--radius-md);           /* 6 px */
  box-shadow: var(--shadow-base);            /* 0 5px 0 0 navy */
  cursor: pointer;
  transition: transform 120ms var(--ease-snap),
              box-shadow 120ms var(--ease-snap),
              background-color 80ms var(--ease-snap);
}
.btn:hover  { transform: translate(0, 2px); box-shadow: var(--shadow-sm); }
.btn:active { transform: translate(0, 5px); box-shadow: none; }

/* Variants */
.btn--main    { background: var(--color-main); color: var(--color-main-fg); border-color: var(--color-main); }
.btn--main:hover { background: var(--color-main-hover); }
.btn--outline { background: transparent; box-shadow: none; }
.btn--outline:hover { transform: none; background: var(--color-ash); }
.btn--ghost   { background: transparent; border-color: transparent; box-shadow: none; }
.btn--ghost:hover { transform: none; background: var(--color-ash); }
.btn--danger  { background: var(--color-coral); color: var(--color-ink); border-color: var(--color-ink-soft); }
.btn--sm      { height: 2.25rem; padding-left: var(--space-5); padding-right: var(--space-5); font-size: var(--text-sm); }
.btn--lg      { height: 3.25rem; padding-left: var(--space-8); padding-right: var(--space-8); font-size: var(--text-lg); }
```

### 9.2 Card

```css
.card {
  background: var(--color-white);
  color: var(--color-fg);
  padding: var(--space-8);
  border: var(--border-hair) solid var(--color-ink-soft);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-base);
}
.card--flat  { box-shadow: none; }
.card--dark  { background: var(--color-navy); color: var(--color-fg-inverse); border-color: var(--color-navy); box-shadow: var(--shadow-base-ink); }
.card--ghost { background: var(--color-ash); border-color: transparent; box-shadow: none; }
.card--feature { border-width: var(--border-strong); }    /* 2px for the 1 hero card */
.card--interactive { cursor: pointer; }
.card--interactive:hover  { transform: translate(0, 2px); box-shadow: var(--shadow-sm); }
.card--interactive:active { transform: translate(0, 5px); box-shadow: none; }
```

### 9.3 Input

```css
.input {
  display: block;
  width: 100%;
  height: 2.75rem;
  /* Physical longhand — same rationale as `.btn` / `.chip` (see §9.4). */
  padding-top: 0;
  padding-bottom: 0;
  padding-left: var(--space-3);
  padding-right: var(--space-3);
  font-size: var(--text-base);
  font-weight: var(--font-weight-regular);
  color: var(--color-fg);
  background: var(--color-white);
  border: var(--border-hair) solid var(--color-ink-soft);
  border-radius: var(--radius-sm);
  box-shadow: none;
  transition: box-shadow 120ms var(--ease-snap);
}
.input:focus-visible {
  outline: 2px solid var(--color-ring);
  outline-offset: 2px;
  box-shadow: var(--shadow-sm);
}
.input--invalid { border-color: var(--color-danger); }
```

### 9.4 Chip internal padding (v3.2 — MANDATORY, updated v3.10)

Problem observed on /auth: chips with `padding: 0` made the letter kiss the
border. Problem observed on /demos (v3.10): chips with fixed `height: 2rem`
+ `line-height: 1` clipped Hangul descender strokes at 320 px — the
glyph's bottom was cut by the border. Rule:

| Chip kind | `min-height` | Horizontal pad | Vertical pad | `line-height` | Gap (icon ↔ text) |
|---|---|---|---|---|---|
| `.chip` (default) | 2 rem (32 px) | `--space-4` (16 px) | `--space-1` (4 px) | 1.15 | `--space-2` (8 px) |
| `.chip--sm` | 1.625 rem (26 px) | `--space-3` (12 px) | 2 px | 1.15 | `--space-2` |
| `.tag` | auto | `--space-2` (8 px) | 3 px | 1.4 | `--space-1` (4 px) |
| `.badge` | ≥ 20 px | `--space-2` (8 px) | `--space-1` (4 px) | 1 | `--space-1` |

v3.10 changes (all four utilities):
1. Fixed `height` → `min-height` so long labels wrap cleanly instead of
   clipping.
2. Explicit vertical padding (no more "implicit via height – line-height")
   so descenders clear the border on every glyph set.
3. **Physical longhand `padding-top / bottom / left / right`** — Tailwind
   v4 preflight emits `*, ::before, ::after { padding: 0 }` inside
   `@layer base`, and on Chromium the per-side physical declaration beats
   our shorthand `padding:` even when our class specificity is higher.
   Matching Tailwind's physical shape guarantees we win the cascade.
4. `white-space: nowrap` on `.chip` and `.tag` so short pills stay one
   line; long labels fall back to wrapping only if the container is
   narrower than the chip's intrinsic width.

**Cascade infrastructure (v3.10 required)**: the whole
`neobrutalism.css` sheet is imported with `@import "./neobrutalism.css"
layer(components)` from `globals.css`, placing every `.chip / .btn /
.input / .tag` declaration in `@layer components` — above Tailwind's
`@layer base` preflight in the cascade. An unlayered universal reset
(`*, *::before, *::after { margin: 0; padding: 0 }`) previously sat in
`globals.css` and clobbered vertical padding even at higher specificity;
it was removed in v3.10 because Tailwind preflight already normalises
box-model margins/paddings.

**Minimum**: horizontal padding ≥ `--space-3` (12 px). Text never touches
the border — there is always at least 12 px of breathing on each side,
and at least 2 px of vertical space (4 px on `.chip` default).

### 9.4a Step marker (numeric)

For ordered flow indicators ("1 / 2 / 3") use `.step`:

```html
<span class="step" aria-hidden="true">1</span>
```

- Circle (`--radius-pill`), 32 × 32 px by default
- Navy background, white mono text, weight 500
- `.step--ghost` swaps to ash / navy text when placed on a dark surface
  would clash.

**Never** use `rounded-lg` / `rounded-md` on a numeric step — the square-ish
rounded shape looks incidental. A step marker is either a full circle or
not present at all.

### 9.4a-2 Tag (v3.4, tight-radius label)

`.tag` is the **sharp-cornered** rectangular label for dense dashboards —
tech-stack chips, quiz answers inside a bucket, inline code-like callouts.

```css
.tag {
  padding: 2px var(--space-2);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  font-weight: 500;
  border: 1px solid var(--color-line);
  border-radius: var(--radius-xs);   /* 2 px — near-sharp */
  background: var(--surface-cream);
  color: var(--color-fg);
}
.tag--navy / --lime / --amber / --coral / --violet   /* palette-tinted */
```

**When to use `.tag` vs `.chip` vs `.badge`**:

| Use case | Component | Radius |
|---|---|---|
| Nav item, selectable step | `.chip` | pill (`--radius-pill`) |
| Status label ("필수", "심화", "BETA") | `.badge` | `--radius-sm` (4 px) |
| Dense technology marker, quiz drop token, inline code accent | `.tag` | `--radius-xs` (2 px — sharpest) |

Rule: if the element sits inside a crowded grid of similar siblings
(e.g. 6 quiz answers, a tech-stack list), prefer `.tag` — the sharp
corner keeps the rhythm crisp. Fully round pills break the visual line.

### 9.4a-3 Status inks (v3.4, AAA on light)

The bright accent colours (`lime #B9FF66`, `saffron #FFDC58`, `coral #FF6B6B`,
`violet #C4A1FF`) **must not be used as text colour on light backgrounds**.
On light tints the contrast sinks below AA. Use the AAA-safe darks instead:

```css
--ink-success: #14532D;   /* deep green on lime/cream */
--ink-danger:  #7F1D1D;   /* deep red on coral/white */
--ink-warning: #78350F;   /* deep amber on amber/cream */
--ink-info:    #3730A3;   /* deep indigo on violet/cream */
--ink-accent:  var(--color-navy);   /* brand emphasis text */
```

Pair tokens (always use together, never mix):

| Surface | Text |
|---|---|
| `--surface-lime` | `--ink-success` |
| `--surface-coral` | `--ink-danger` |
| `--surface-amber` | `--ink-warning` |
| `--surface-violet` | `--ink-info` |
| `--surface-navy` | `--ink-accent` |

**Anti-pattern** (observed on SSHSimulator v3.3):
```tsx
style={{
  background: "color-mix(in oklch, var(--color-success) 8%, white)",
  color: "var(--color-success)",   /* lime #B9FF66 on pale lime cream → ~1.4 : 1 */
}}
```
**Fix**: use `.state state--success` utility (below) or the pair tokens above.

**v3.9 update — token rebind.** Since 2026-04-24 the semantic
`--color-success`, `--color-warning`, `--color-danger`, `--color-info`
(and the legacy alias `--color-cia-i`) no longer resolve to the bright
brand primitive. They now resolve to the AAA `--ink-*` variant above, so
existing inline code like `color: var(--color-success)` passes contrast
automatically. The bright primitives stay available through new
`*-fill` tokens when a saturated button, progress fill, or CTA pill is
the intended look:

```css
--color-success:      var(--ink-success);    /* default — text-safe       */
--color-success-fill: var(--color-lime);     /* opt-in — fill or border   */
/* same split for --color-warning / --color-danger / --color-info        */
```

Use `*-fill` only on elements whose text is white or ink-soft already:
`background: var(--color-success-fill); color: var(--color-ink);` —
black-on-lime scores 13 : 1 which is fine, but saturated fills still
read as "연두색 chip" so reach for them sparingly.

Badge variants follow the same split. `.badge--success/warning/danger/info`
now render as pale-tint + ink-text (quiet, uniform with `.tag--*` and
`.state--*`). Opt into a saturated pill via `.badge--success-fill`,
`.badge--warning-fill`, etc.

### 9.4a-4 `.state` wrapper (v3.4)

Any block carrying a status meaning — success toast, danger hint, warning
callout — uses `.state`:

```html
<div class="state state--success">
  <div class="state__title">✅ 인증 성공 — 세션 수립</div>
  <p>...body...</p>
</div>
```

Variants: `state--success / --danger / --warning / --info`. Each pairs the
tinted surface with the AAA ink automatically.

### 9.4b CIA disc — reserved 3-colour system (v3.2, mandatory)

`C` / `I` / `A` are never rendered as plain letters or bordered chips.
They **must** use `<CIABadge>` (from `@/components/learning/CIABadge`),
which renders a `.cia` disc. The three reserved colours are:

| Letter | Fill | Text colour | Meaning |
|---|---|---|---|
| **C** | `#1D4ED8` (blue-700, 6.1 : 1 with white) | white | Confidentiality |
| **I** | `#15803D` (green-700, 4.8 : 1 with white) | white | Integrity |
| **A** | `#F59E0B` (amber-500, 11 : 1 with ink) | ink (black) | Availability |

*v3.9 update:* C moved blue-600 → blue-700, I moved green-600 → green-700.
The 600 shades scored ≈ 3.2 : 1 with white (AA fail for body text), so
small discs (`.cia--sm`, 20 px) embedded inline with copy failed WCAG.
700 clears 4.5 : 1 at every size while still reading as "blue" and
"green" semantically.

Rules:
1. These three fills are **reserved** — never used for any other decorative
   purpose, anywhere in the app. The colour ↔ letter binding becomes
   learnable after one page.
2. Discs are always circular (`--radius-pill`). Size variants:
   `.cia--sm` (20 px), `.cia` default (24 px), `.cia--md` (28 px),
   `.cia--lg` (32 px).
3. Letter is mono-font, weight 500, centred.
4. Never place on a background of the same hue family (e.g. the blue `C`
   disc on a navy card washes out — use `.cia--md`+ for size contrast).
5. Icon-only usage carries `role="img"` + `aria-label` automatically via
   the React component.

### 9.4f Accordion (v3.8)

When a list of items has **expandable content** (extended notes, code
example, raw file view), use the native `<details>` / `<summary>` element
wrapped in `.card`. This preserves keyboard + screen-reader behaviour
without any JS.

Structural rule (same as "nested card" §7.4): the `.card` drops its own
padding to zero, and the `<summary>` + body each carry padding.

```tsx
<details className="card" style={{ padding: 0, overflow: "hidden" }}>
  <summary
    className="row"
    style={{
      justifyContent: "space-between",
      padding: "var(--space-3) var(--space-5)",
      cursor: "pointer",
      listStyle: "none",
    }}
  >
    <span style={{ fontWeight: 500 }}>id_ed25519</span>
    <div className="row" style={{ gap: "var(--space-2)" }}>
      <span className="tag tag--coral">chmod 600</span>
      <span aria-hidden>▾</span>
    </div>
  </summary>

  <div
    className="stack stack--tight"
    style={{
      padding: "var(--space-4) var(--space-5) var(--space-5)",
      borderTop: "var(--border-hair) solid var(--color-line)",
      background: "var(--surface-ivory)",
    }}
  >
    {/* body content */}
  </div>
</details>
```

Rules:
- Caret (`▾`) is **right-aligned** on the summary, `aria-hidden="true"`.
  Native `<details>` toggles state; don't re-implement with `useState`
  unless animated height is required.
- Body background is `--surface-ivory` (always) so the expanded region
  is visibly distinct from the closed summary header.
- Critical/dangerous items accept an optional left-accent 3-px border in
  `--color-coral`; otherwise no accent.
- Never nest `.card` inside a `<details>` body — use utilities
  (`.state`, `.tag`, plain `<pre>`) directly.

### 9.5 Chip / nav pill

```css
.chip {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  height: 2rem;
  padding: 0 var(--space-4);
  font-size: var(--text-sm);
  font-weight: var(--font-weight-medium);
  color: var(--color-fg);
  background: transparent;
  border: var(--border-hair) solid transparent;
  border-radius: var(--radius-pill);
  transition: background-color 80ms, border-color 80ms;
}
.chip:hover          { border-color: var(--color-ink-soft); }
.chip[aria-current="page"] {
  background: var(--color-navy);
  color: var(--color-fg-inverse);
  border-color: var(--color-navy);
}
```

### 9.5 Badge

```css
.badge {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  padding: var(--space-1) var(--space-2);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  font-weight: var(--font-weight-medium);
  letter-spacing: var(--tracking-caps);
  text-transform: uppercase;
  background: var(--color-ash);
  color: var(--color-fg);
  border: var(--border-hair) solid var(--color-line);
  border-radius: var(--radius-sm);
}
.badge--navy    { background: var(--color-navy); color: var(--color-fg-inverse); border-color: var(--color-navy); }

/* v3.9 — status badges render as pale-surface + ink-text (quiet,
   uniform with `.tag--*` and `.state--*`). A lime/coral/saffron/violet
   fill with black text still passes AA, but the whole surface read as
   "연두색/산호색 chip" and made pages look over-candied. */
.badge--success { background: var(--surface-lime);   color: var(--ink-success); border-color: #16A34A; }
.badge--warning { background: var(--surface-amber);  color: var(--ink-warning); border-color: #F59E0B; }
.badge--danger  { background: var(--surface-coral);  color: var(--ink-danger);  border-color: #DC2626; }
.badge--info    { background: var(--surface-violet); color: var(--ink-info);    border-color: #7C3AED; }

/* Full-saturation fill variants — opt-in for the occasional "hot" CTA
   pill. Ink text retains AA contrast on every bright fill. */
.badge--success-fill { background: var(--color-lime);    border-color: var(--color-ink-soft); color: var(--color-ink); }
.badge--warning-fill { background: var(--color-saffron); border-color: var(--color-ink-soft); color: var(--color-ink); }
.badge--danger-fill  { background: var(--color-coral);   border-color: var(--color-ink-soft); color: var(--color-ink); }
.badge--info-fill    { background: var(--color-violet);  border-color: var(--color-ink-soft); color: var(--color-ink); }
```

### 9.6 Alert, dialog, tabs, rail, codeblock

See `src/styles/neobrutalism.css` §6. Key change from v2: shadow + border
widths/colours migrated to the navy+hairline system.

---

## 10. Navigation (v3.6 — compact + three-state chips + brutalist dropdown)

The learning chrome has **two** rows of chrome total (v3.5 reduced from three):

### 10.1 Top nav — single category row

- **No wordmark row, no separate mobile menu toggle row.** The header is
  a single 44-px-tall row. Content starts 44 px from the viewport top.
- The row renders **one `.chip chip--sm` per PART category** (`P0 들어가며`,
  `P1 보안이란?`, …, `P7 용어사전`). **No CryptoStage chip** or external
  links in the nav (v3.6) — external resources go in page bodies.
- Chip font-size is `--text-xs` (13–14 px) and padding `0 var(--space-3)`
  (12 px) so all 8 PART chips fit the 1240-px container without
  horizontal scroll on desktop.
- Clicking a category chip toggles a **dropdown panel** directly below
  the bar. Dropdown closes on outside click, Escape, or navigation.
- Dropdown grid: `repeat(auto-fill, minmax(15rem, 1fr))` — responsive.

### 10.1a Three chip states (MANDATORY — user confusion fix, v3.6)

The same chip can carry three meanings. Each state has a distinct fill so
the user never confuses "where I currently am" with "what I'm peeking at".

| State | When | Background | Text | Border |
|---|---|---|---|---|
| **default** | idle | transparent | `--color-fg` | transparent (hover → `--color-ink-soft`) |
| **active** | the PART contains the current page | `--color-navy` | `--color-fg-inverse` | `--color-navy` |
| **open**   | the user just clicked it and the dropdown is visible, but hasn't navigated away yet | `--color-saffron` | `--color-ink` | `--color-ink-soft` |

If the currently-active PART is also the one the user is peeking at,
**open wins** (the amber signals "you're inspecting"). When the user
dismisses the dropdown (outside click / Escape), the chip falls back to
active.

Inside the dropdown the **currently-selected lesson** inherits the same
navy-fill treatment — the lesson where you are, everywhere, is navy.

### 10.1c Submenu link hover (v3.6)

Every row inside the dropdown must show a visible outline on hover so the
user sees the click target before committing. The `.submenu-link` utility
applies a 1-px ink outline + ivory fill on hover, matching the overall
brutalist language (outlines, no glow):

```css
.submenu-link:hover {
  border-color: var(--color-ink-soft);
  background: var(--surface-ivory);
}
.submenu-link:focus-visible { outline: 2px solid var(--color-ring); outline-offset: 1px; }

/* the current lesson row already carries a navy fill — its hover keeps
   the fill but shows an inverse white outline to still signal "hover" */
.submenu-link[data-current]:hover {
  border-color: var(--color-fg-inverse);
  background: var(--color-navy);
}
```

Anti-pattern: no-op hover (transparent → transparent). Every row in a
menu needs a visible target to avoid the "is it clickable?" hesitation.

### 10.1b Submenu panel — brutalist (v3.6)

The dropdown panel is the **one place** the otherwise-banned diagonal
shadow is used, because it clearly communicates "this is a floating
temporary surface, click-outside to dismiss":

```css
/* v3.10 — panel re-skinned to the navy surface. Content cards already
   use cream (`--surface-cream`), so a cream panel stacked above them
   read as a single visual plane. Neobrutalism calls for a clear
   "nav chrome ≠ content" separation — dark navy + white text gives
   that in one stroke. */
.submenu-panel {
  background: var(--color-navy);
  color: var(--color-fg-inverse);
  border: var(--border-strong) solid var(--color-navy-deep);   /* 2 px */
  border-radius: var(--radius-md);                              /* 6 px */
  /* Ink-soft drop shadow (not navy) — a navy-on-navy shadow is
     invisible. Diagonal 5/5 so right + bottom both register. */
  box-shadow: 5px 5px 0 0 var(--color-ink-soft);
  padding: var(--space-3) var(--space-4) var(--space-4);
}

.submenu-link--on-dark {
  /* Inverted hover + focus for the dark panel context; lime current
     fill with black ink text (AAA). See neobrutalism.css §7.3b. */
}
```

Rules:
- **Surface**: navy (`--color-navy`) — NOT `--surface-cream`. A cream
  panel that matches the content cards below fails the neobrutalism
  "chrome ≠ content" separation test.
- **Current-page fill**: lime (`--color-lime`) with `--color-ink` text,
  AAA on black.
- **Border width**: 2 px (`--border-strong`) — one step heavier than
  regular `.card` to emphasise the floating surface.
- **Shadow**: exactly `5px 5px 0 0 var(--color-ink-soft)` — navy shadow
  on a navy panel is invisible. Ink-soft reads against both the navy
  panel and the white canvas below.
- This is the **only** diagonal shadow allowed in the system. Do not
  port this recipe to cards, buttons, or other floating surfaces without
  an RFC through the designer agent.

```tsx
<header className="sticky top-0 z-40 ..." >
  <nav className="container relative">
    <ul className="flex items-center gap-1 overflow-x-auto" style={{ height: "2.75rem" }}>
      {PARTS.map(part => (
        <li key={part.id}>
          <button className="chip" aria-expanded={isOpen} ...>
            <span className="font-mono text-xs">P{part.id}</span>
            <span>{part.title}</span>
            <span aria-hidden>▾</span>
          </button>
        </li>
      ))}
      <li style={{ marginLeft: "auto" }}>
        <a className="chip" href="https://example.com/secondary-product">SecondaryProduct ↗</a>
      </li>
    </ul>

    {openPart && <DropdownPanel section={section} />}
  </nav>
</header>
```

**Anti-patterns** (rejected in review):
- Adding a second chrome row above the category row for a wordmark,
  logo, search, or CTA. The 44-px compact row is the whole header.
- A persistent visible lesson list (like v3.0–v3.4 had). Lessons live in
  the dropdown.
- Category labels longer than ~6 Korean characters — if a PART's title
  exceeds that, shorten it on the chip (full title stays in the dropdown
  eyebrow).

### 10.2 Footer prev/next

Unchanged from v3.0: ghost `.btn--outline btn--sm` for prev, solid
`.btn--main btn--sm` for next, centred `01 / 32` tabular count, 2 px navy
progress rail across the top edge of the footer.

---

## 10a. Page-root template (v3.8 canonical — MANDATORY)

Every learning page file under `src/app/(learning)/**/page.tsx` MUST start
with this exact shell. The learning layout already supplies the
`.container` wrapper + horizontal padding + max-width, so the page body
itself must not re-introduce any of those.

```tsx
export default function SomePage() {
  return (
    <div className="stack stack--loose">
      {/* Header block — always three lines: eyebrow / h1 / lede */}
      <section className="stack stack--tight">
        <span className="eyebrow">PART N · 섹션 이름</span>
        <h1>페이지 제목</h1>
        <p className="lede">한 문장 요약.</p>
      </section>

      {/* sub-sections */}
      <section className="stack stack--tight">...</section>

      {/* optional hero feature card — exactly one per page */}
      <section className="card card--dark">...</section>

      {/* optional closing CTA */}
      <section className="cta cta--dark">...</section>
    </div>
  );
}
```

### Banned page-root patterns (v3.8 audit findings)

The v3.8 polish pass found and removed these across 24 files. They must
never reappear:

```tsx
// ❌ 1. Legacy prose wrapper — the layout's container already does this
<article className="max-w-3xl py-8">...</article>

// ❌ 2. Hand-rolled max-width on page root
<div className="max-w-3xl mx-auto px-4">...</div>

// ❌ 3. mx-auto + w-full combined — w-full kills the auto-margin centring
<div className="mx-auto w-full max-w-3xl">...</div>

// ❌ 4. space-y-* on the page root or on sibling sections
<div className="space-y-16 pb-8">...</div>

// ❌ 5. Heading utility overrides — defaults already handle size/weight
<h1 className="text-3xl font-medium mt-2 mb-3" style={{ fontSize: "var(--text-hero)" }}>
  Page title
</h1>
```

Replace with:

```tsx
// ✅ 1 & 2 — let layout own the container
<div className="stack stack--loose">

// ✅ 3 — never combine mx-auto + w-full
<div className="container">                       /* standalone; NOT on page root */

// ✅ 4 — stack utilities only
<div className="stack stack--loose">

// ✅ 5 — bare element, no utility override
<h1>Page title</h1>
```

### Interior-padding exceptions (allowed)

`px-*` / `py-*` Tailwind utilities are fine INSIDE components when the
element isn't the page root:

- Table cells: `<th className="px-4 py-3">`, `<td className="px-4 py-3">`
- Button-like `<Link>` rows inside a card list: `<Link className="px-5 md:px-8 py-5">`
- Card-header dividers: `<div className="px-6 py-4 border-b">` inside `.card card--flat`

These are legitimate interior padding and are not caught by the root-level
grep below.

---

## 10b. Enforcement — CI greps (v3.8)

Every PR that touches `src/app/(learning)/` or `src/components/learning/`
must pass all 13 greps with **zero matches at page-root level** (interior
patterns from §10a above are the only exception). Run this exact block
before opening a PR:

```bash
# 1. Outer horizontal padding on page roots
grep -rnE 'className="[^"]*\bpx-(4|6|8|10|12)\b' src/app/\\(learning\\)/*/page.tsx \
  | grep -v '<th\|<td\|<button\|<Link\|card-header'

# 2. Legacy max-width on page roots
grep -rnE 'className="[^"]*max-w-(3xl|4xl|5xl|6xl|7xl)' src/app/\\(learning\\)/*/page.tsx

# 3. Outer py-* on page root wrapper
grep -rnE 'className="[^"]*\bpy-8\b' src/app/\\(learning\\)/*/page.tsx

# 4. mx-auto + w-full combination
grep -rn 'mx-auto.*w-full\|w-full.*mx-auto' src/app/\\(learning\\) src/components/learning

# 5. Hand-rolled eyebrow (use .eyebrow)
grep -rnE 'font-mono\s+text-xs\s+uppercase' src/app/\\(learning\\) src/components/learning \
  | grep -v 'eyebrow\|cia\|tag'

# 6. Bright accent as text colour on light bg
grep -rnE 'color:\s*["'\'']var\(--color-(success|danger|warning|error|info)\)' \
  src/app/\\(learning\\) src/components/learning

# 7. Disallowed radii
grep -rnE '\brounded-(xl|2xl|3xl)\b|borderRadius:\s*["'\'']1[2-9]px|borderRadius:\s*["'\'']2[0-9]px' \
  src/app/\\(learning\\) src/components/learning

# 8. Bold weight (max is 500)
grep -rnE '\bfont-(bold|semibold|extrabold|black)\b|fontWeight:\s*[6789]00' \
  src/app/\\(learning\\) src/components/learning

# 9. Hand-rolled card
grep -rnE 'className="[^"]*rounded-(md|lg)[^"]*\bborder\b' \
  src/app/\\(learning\\) src/components/learning \
  | grep -v 'submenu\|\.card\|\.chip\|\.btn\|\.tag\|\.badge'

# 10. Pure white card bg
grep -rnE 'background:\s*["'\'']var\(--color-white\)|background:\s*["'\'']#[fF]{3,6}["'\'']' \
  src/app/\\(learning\\) src/components/learning

# 11. Mixed space-y-* rhythm
grep -rnE '\bspace-y-(5|16|20)\b' src/app/\\(learning\\) src/components/learning

# 12. Raw Tailwind palette colours
grep -rnE '(bg|text|border)-(red|orange|amber|yellow|lime|green|emerald|teal|cyan|sky|blue|indigo|violet|purple|fuchsia|pink|rose|gray|zinc|slate|neutral|stone)-[0-9]+' \
  src/app/\\(learning\\) src/components/learning

# 13. Hand-rolled CIA letter square
grep -rnE '>[\s]*(C|I|A)[\s]*<' src/app/\\(learning\\) src/components/learning \
  | grep -iE 'bg.*cia|background.*cia' | grep -v CIABadge
```

A helper script lives at `scripts/design-audit.sh` (TODO for future CI).

---

## 11a. Curriculum map (v3.7)

The learning tree has **9 PARTs** / 30 pages (learning chrome page source of
truth at `src/lib/progress/navMap.ts`).

| PART | Title | Pages |
|---|---|---|
| 0 | 들어가며 | /intro |
| 1 | 보안이란? | /cia · /auth · /trust |
| 2 | 암호학 도구 | /hashing · /symmetric · /asymmetric · /hmac · /key-exchange · /pki · /password |
| 3 | SSH | /ssh-overview · /ssh-keygen · /ssh-auth · /ssh-server |
| 4 | HTTPS & 웹 기본 | /tls · /http-vs-https |
| 5 | **Web App 보안** (v3.7 신설) | /webapp-overview · /webapp-injection · /webapp-supply-chain · /webapp-auth · /webapp-devops |
| 6 | VPN (v3.7 재구성) | /vpn-overview · /ssh-tunnel · /wireguard · /vpn-scenarios |
| 7 | 개인정보보호법 | /pipa-overview · /pipa-rights · /pipa-security · /pipa-checklist |
| 8 | 용어사전 | /glossary |

### Removed in v3.7 (rationale)

- `/pfs` — 고급 주제. TLS 페이지의 '임시 키 이유' 1문단으로 흡수.
- `/ssh-debug` — 실무 trouble-shoot은 검색해서 해결. FAQ는 `/ssh-auth`·`/ssh-server` 내부 편입.
- `/ssh-files` — 키 생성과 폴더 구조를 분리할 이유 없음. `/ssh-keygen`에 흡수.
- `/web-nodes` — HTTP vs HTTPS 설명과 중복. `/http-vs-https` 내부로 흡수.
- `/vpn-protocols` — 사용자 요청: 알고리즘 성능 비교 불필요. PART 6 전면 재구성.
- `/pipa-breach` — 침해사고 대응은 실무 플레이북 영역. `/pipa-security` 한 섹션으로 흡수.

### 새 PART 5 "Web App 보안" 구성 원칙

실습 중심: 모든 페이지에 **취약 코드 → 공격 재현 → 패치된 코드** 3-pane 패턴.
개발자가 "이 실수가 왜 위험한지" 한 번에 보고 고칠 수 있도록.

### PART 6 VPN 구성 원칙

**개념 → 터널 → 실습 → 시나리오** 흐름. 알고리즘별 우위, 성능 벤치마크,
프로토콜 비교표는 의도적으로 배제 (현업에선 WireGuard가 사실상 기본이고
관리 레이어가 더 중요).

---

## 11. Page template (mandatory for all learning pages)

```tsx
<article className="stack">
  <header className="stack stack--tight">
    <span className="eyebrow">PART N · 섹션 이름</span>
    <h1>페이지 제목</h1>
    <p className="lede">한 문장 요약.</p>
  </header>

  <section className="stack">
    <h2>서브 섹션</h2>
    {/* content */}
  </section>

  {/* hero block (optional) */}
  <section className="card card--dark">
    ...
  </section>

  {/* call to action (end of page) */}
  <section className="cta">
    <h2>다음으로</h2>
    <div className="cta__row">
      <a className="btn btn--main" href="...">다음 페이지 →</a>
      <a className="btn btn--ghost" href="...">건너뛰기</a>
    </div>
  </section>
</article>
```

- `stack` utility applies `display: flex; flex-direction: column; gap: var(--space-6)` (24 px, v3.10).
- `stack--tight` swaps gap to `var(--space-3)` (12 px). `stack--loose` → `var(--space-8)` (32 px).
- Cards default to `--surface-cream`; accent tints via `.card--*-tint`. Pure white is opt-in only (`.card--plain`).
- CIA letters **always** via `<CIABadge>`. Status blocks **always** via `.state`.
- Interactive controls **always** via `.btn .btn--sm {--main|--outline|--danger|--ghost}` (v3.10 §7a). Hand-rolled `px-* py-* bg-* rounded-*` button is a lint violation.
- Navigation dropdown panel renders on navy (`--color-navy`), not cream — §10.1b (v3.10).

---

## 12. Frontend-agent checklist (v3.4, mandatory before PR)

> **Design reference (v3.11)** — this document is the single source of
> truth. `docs/design-guide.md` was merged in and deleted; any PR that
> introduces a new cheatsheet / summary / quick-ref file is rejected on
> sight. Questions about tokens, spacing, radius, typography, chips,
> buttons, menu → search this file first.

### Components (v3.3 HARD RULE)
- [ ] Page uses only components from `src/components/ui/*` + utilities from `neobrutalism.css` + `<CIABadge>`.
- [ ] No hand-rolled `<div className="rounded border p-4 bg-white shadow">` — convert to `.card`.
- [ ] No raw Tailwind colour utilities (`bg-red-500`, `text-orange-600`, `border-gray-300`, …).
- [ ] No bespoke `src/components/learning/<Name>.tsx` that re-implements an approved class.

### Canvas & colour (v3.3)
- [ ] Page background is `--color-bg` (white). Sections inside never set their own background; cards handle the tinted surface.
- [ ] Cards use `.card` (cream default) or one of the palette tints. Pure white on cards = forbidden unless `.card--plain`.
- [ ] Bright accents (`lime/saffron/coral/violet`) NEVER used as text colour on light backgrounds. Use `--ink-success/danger/warning/info` or `.state .state__title`.

### CIA + status signalling (v3.2 / v3.4)
- [ ] Any letter `C`, `I`, `A` in copy or cards is rendered through `<CIABadge type="C|I|A" size="..." />`.
- [ ] Success/danger/warning/info blocks wrap in `.state state--success|danger|warning|info` with a `.state__title` row. No ad-hoc `color: var(--color-success)` on text.

### Borders, radius, shadow (v3.2)
- [ ] Every bordered surface uses `--border-hair` (1 px). 2 px only on the single focus CTA of a view.
- [ ] Radius pulls from `--radius-xs (2) / --radius-sm (3) / --radius-md (6) / --radius-lg (10) / --radius-pill`. **Never anything between 10 and pill.**
- [ ] Dense rectangular labels (quiz answers, tech-stack chips) use `.tag` (2 px) — not `.badge`, not `.chip`.
- [ ] Shadow = one of `--shadow-xs/sm/base/lg/xl` — vertical-only, navy-tinted.
- [ ] Hover/press: `translate(0,2px)` + `--shadow-sm` / `translate(0,5px)` + no shadow.

### Typography
- [ ] Primary font resolves to **Pretendard Variable** (loaded via `<link>` in `app/layout.tsx`). Do not set `font-family` inline except via `var(--font-sans)` / `var(--font-display)` / `var(--font-mono)`.
- [ ] Weights only 400 or 500. Never 600+.
- [ ] Body ≥ 16 px. Captions ≥ 14 px.
- [ ] Eyebrow labels via `.eyebrow` (never inline `font-mono text-xs uppercase tracking-widest`).
- [ ] Headings rely on the default element style (`h1`–`h4`), no Tailwind `font-bold` overrides.

### Interactive controls (v3.10 HARD RULE — §7a)
- [ ] Every `<button>` / `<a role="button">` inside a learning page uses `.btn .btn--sm {--main|--outline|--danger|--ghost}`. Hand-rolled `px-* py-* bg-* rounded-* font-medium` buttons are a lint violation (they render as chips).
- [ ] Every `<input>` / `<textarea>` / `<select>` uses `.input` (or `.input font-mono` for hex/mono text). Hand-rolled padding+border combos are out.
- [ ] No `px-2 py-0.5 rounded-full` ad-hoc pills — use `.chip .chip--sm` with a `.tag--*` tone for colour.
- [ ] Small-control radius uses `rounded-md` (6 px) only. `rounded-lg` is hero-only (§5.2a).

### Accessibility (v3.9 contrast sweep)
- [ ] Bright accents (`--color-lime / --color-saffron / --color-coral / --color-violet`) NEVER used as text colour on light backgrounds. Use the AAA ink counterparts (`--ink-success` etc.) or the already-rebound semantic tokens (`--color-success` etc.).
- [ ] Any `color: var(--color-cia-i)` inline style resolves to `--ink-success` now — fine to keep as-is. But `color: var(--color-lime)` in a component file is still banned.
- [ ] CIA disc inline renderings use the React `<CIABadge>` component — never hand-roll the green-700 / blue-700 fill.

### Layout (v3.1 / v3.3)
- [ ] Page body wrapped in `<div className="stack stack--loose">` → `<section className="stack stack--tight">` hierarchy. Never `space-y-*` siblings with mixed values.
- [ ] Outer layout uses the learning `<div className="container">` — never `mx-auto w-full`, never own `px-*` on page root.
- [ ] 3-up grids use `gap-5` (20 px). Inner column breathes via the container's 32 px padding (so first/last column edge > inter-card gap).

### Nested components (v3.2)
- [ ] A `.card` hosting a sub-grid with borders zeroes its own padding; inner cells carry `padding`.
- [ ] `<details>` / `<summary>` use the same pattern — summary row padded, outer `.card` padding 0.

### Layout
- [ ] Container max = `--container-lg` (marketing) or `--container-md` (dashboard).
- [ ] Section gap = `--space-section`. Sub-sections = `--space-10`.
- [ ] Card padding = `--space-8` default.

### Forbidden
- [ ] No black shadow. Shadow colour is navy.
- [ ] No diagonal shadow (`Xpx Ypx`). Vertical only.
- [ ] No `bg-lime` / `bg-saffron` / `bg-violet` on cards — status-only.
- [ ] No weight ≥ 600 anywhere.
- [ ] No `border-2` / `border-4` Tailwind utilities — use explicit `border` + `var(--border-hair)`.
- [ ] No decorative emojis in page copy (status dots are fine).
- [ ] No 3-tone or 4-tone card grids. Cards are white; variation is via
      size/span, not colour.

---

## 13. On-demand CSS catalog

When the frontend agent needs more than the base recipes, they request one
of these codes from the designer agent:

| Code | Pattern |
|---|---|
| `NB-01` | Hero — headline left, illustration card (navy) right, 1 navy CTA + 1 ghost. |
| `NB-02` | Feature grid 3-up — white cards, 1 px border, navy shadow, numeric eyebrows. |
| `NB-03` | Process stepper — vertical list of numbered steps, white cards, chevron rails. |
| `NB-04` | Pricing table — single-pixel divider columns, navy shadow. |
| `NB-05` | Newsletter CTA — full-bleed navy block with white form. |
| `NB-06` | Codeblock — ink-soft bg, monospaced, 1 px navy border. |
| `NB-07` | Progress rail — 2 px navy fill on ash track. |
| `NB-08` | CIA triad or trust-model comparison — 3 white cards, one labelled with lime status badge. |
| `NB-09` | TOC / progression pill row. |
| `NB-10` | 404 / error page — navy hero with lime highlight. |

The canonical implementations live at `src/styles/neobrutalism.css`
(utility classes) and `src/components/patterns/` (React).

---

## 14. Migration — token changes across versions

### v2 → v3.0

| v2 token | v3.0 token | Notes |
|---|---|---|
| `--color-bg: paper` | `--color-bg: white` | ⚠ global canvas change |
| `--color-main: lime` | `--color-main: navy` | ⚠ primary accent flipped |
| `--border-base: 2px` | `--border-hair: 1px` | alias kept so old `border-base` resolves to 1 px |
| `--shadow-base: 4px 4px 0 ink` | `--shadow-base: 0 5px 0 navy` | ⚠ direction + colour flipped |
| `--radius-md: 4px` | `--radius-md: 8px` | ⚠ softened |
| hover `translate(2px,2px)` | hover `translate(0,2px)` | vertical |
| h1 weight 600 | h1 weight 500 | |
| body weight 500 | body weight 400 | captions still 500 |

### v3.0 → v3.1

| Surface | v3.0 | v3.1 |
|---|---|---|
| Container `padding-inline` @ ≥ md | 20 px | **32 px** (first/last column breathing) |
| Dark card muted text | inline `rgba(255,255,255,0.65)` | `.card--dark :where(...)` cascade (`--color-fg-inverse-muted` 0.88, `--color-fg-inverse-subtle` 0.72) |
| Shadcn neobrutalism tokens | unmapped | `bg-main`, `bg-secondary-background`, `shadow-shadow`, `rounded-base`, `font-base/heading` wired via `@theme` |
| html font-size | variable (rem feedback loop) | pinned 16 px |

### v3.1 → v3.2

| Surface | v3.1 | v3.2 |
|---|---|---|
| `--radius-md` | 8 px | **6 px** |
| `--radius-lg` | 12 px | **10 px** |
| Numeric step marker | `rounded-lg` square | `.step` (pill 32×32 navy) |
| CIA letter rendering | bordered text or hand-rolled square | `.cia` circular disc + `<CIABadge>` mandatory, colours C blue / I green / A amber |
| Section rhythm | mixed `space-y-5/6/16` | `.stack / --tight / --loose` only |
| Nested card padding | outer `p-6` + inner `p-6` | outer `.card { padding: 0 }` + inner cell carries padding |

### v3.2 → v3.3

| Surface | v3.2 | v3.3 |
|---|---|---|
| Default card bg | `#FFFFFF` | **`--surface-cream #F7F9F0`** |
| Card tint variants | — | `.card--navy-tint / --lime-tint / --amber-tint / --violet-tint / --coral-tint / --ivory / --plain` |
| Component policy | strong | **HARD RULE** — neobrutalism-only (§0.1); CI greps fail PRs |
| Raw Tailwind colours | allowed | forbidden outside `src/components/ui/*` |

### v3.3 → v3.4

| Surface | v3.3 | v3.4 |
|---|---|---|
| Dense rectangular labels | `.badge` (radius 3) or hand-rolled | **`.tag`** (radius 2 — sharpest) with palette variants |
| Status text on light | bright accent (`color: var(--color-success)`) — contrast FAILS | `--ink-success/danger/warning/info/accent` (AAA darks) |
| Status block | inline `color-mix` bg + bright text | **`.state state--success|danger|warning|info`** (surface + ink auto-paired) |

### v3.4–v3.8 → v3.9

| Surface | ≤ v3.8 | v3.9 |
|---|---|---|
| Primary font | `"Space Grotesk", "Inter", "Pretendard Variable"` (Pretendard as fallback) | **`"Pretendard Variable"` first**; Space Grotesk/Inter as Latin fallback |
| `--color-success/warning/danger/info` | `var(--color-lime/saffron/coral/violet)` (bright) — **text-colour uses FAIL AA** | `var(--ink-success/warning/danger/info)` (deep tones, AAA as text). Bright fills via new `--color-*-fill` aliases. |
| `--color-cia-i` legacy alias | `var(--color-lime)` (fails as text) | `var(--ink-success)` deep green |
| CIA disc `.cia--c` | `#2563EB` (blue-600, 3.2 : 1 white — AA fail at small sizes) | **`#1D4ED8`** (blue-700, 6.1 : 1) |
| CIA disc `.cia--i` | `#16A34A` (green-600, 3.2 : 1) | **`#15803D`** (green-700, 4.8 : 1) |
| `.badge--success/warning/danger/info` | saturated fill (lime/coral/saffron/violet) | **pale-surface + ink-text** (matches `.tag--*`/`.state--*`). Bright fill moved to `.badge--*-fill`. |
| `.eyebrow` colour | `var(--color-fg-subtle)` = `#6B7280` (4.66 : 1 — fails on tints) | `var(--color-fg-muted)` = `#3A3F4B` (10 : 1 AAA) |
| `--color-smoke` | `#6B7280` gray-500 | **`#4B5563`** gray-600 (7.0 : 1 on white, ≥ 5 : 1 on every pale tint) |
| Small-control radius | `rounded-lg` widely used (10 px) | **`rounded-md`** (6 px) mandated by §5.2a; `rounded-lg` hero-only |
| Inline `borderRadius: "0.5rem" / "0.625rem" / "0.75rem"` | allowed | **banned** — use `var(--radius-md)` |
| DH colour-mixing swatches (`/key-exchange`) | `#e76f51` / `#2a9d8f` / `#f4a261` etc. — text contrast ≤ 3 : 1 | darkened to `#B04A2F` / `#1F7A70` / `#8C2E17` / `#14564F` |

### v3.9 → v3.10

| Surface | v3.9 | v3.10 |
|---|---|---|
| Learning-page small buttons | hand-rolled `<button className="px-4 py-2 bg-* rounded-* font-medium">` — flat, no shadow | **`.btn .btn--sm {--main|--outline|--danger|--ghost}`** (border + 5 px navy shadow + translate-on-press) — §7a HARD RULE |
| Learning-page small inputs | hand-rolled `<input className="px-3 py-2 border border-border rounded-md">` | **`.input`** utility |
| `.chip` / `.btn` / `.input` / `.tag` padding | shorthand `padding: 0 var(...)` | **physical longhand** `padding-top/bottom/left/right` (beats Tailwind preflight cascade) |
| `.chip` sizing | fixed `height: 2rem` + `line-height: 1` (clipped Hangul descenders) | `min-height: 2rem` + `line-height: 1.15` + explicit vertical `padding-top/bottom` |
| `.chip--sm` vertical pad | 0 px | 2 px |
| `.tag` vertical pad | 2 px | 3 px |
| `.chip` / `.tag` text wrap | default (could line-break) | `white-space: nowrap` for short pills |
| neobrutalism.css import | unlayered | `@import "./neobrutalism.css" layer(components)` — beats Tailwind `@layer base` preflight |
| `globals.css` duplicate reset | `*, *::before, *::after { margin: 0; padding: 0 }` (clobbered chip/btn padding) | **deleted** — Tailwind preflight already normalises box-model |
| Submenu panel | cream bg + navy shadow — indistinguishable from content cards | **navy bg + ink-soft shadow + lime current-fill**. New `.submenu-link--on-dark` modifier. |
| `.stack` gap | 20 px / 8 px / 24 px | **24 px / 12 px / 32 px** so 5 px navy shadow always breathes |
| Ad-hoc `px-2 py-0.5 rounded-full` pills | allowed | **banned** — use `.chip .chip--sm` |

### v3.10 → v3.11

| Surface | v3.10 | v3.11 |
|---|---|---|
| Design reference docs | `design-system.md` + `design-guide.md` (cheat sheet) | **`design-system.md` only** — guide merged in + deleted to stop drift |
| Editing decision tree | in `design-guide.md §6` | `design-system.md §0.2` |
| Tuning workflows ("조금 더 크게/좁게") | in `design-guide.md §2.2 / §3.2 / §3.3` | `design-system.md §0.3` |
| Horizontal gap density table | in `design-guide.md §3.4` | `design-system.md §4a` |
| Global hygiene grep set | in `design-guide.md §8` | `design-system.md §0.4` |
| `CLAUDE.md` 디자인 섹션 | "스타일 가이드: docs/design-system.md" (soft reference) | **"단일 소스 오브 트루스: docs/design-system.md"** + explicit "추가 cheatsheet 문서 생성 금지" |
| Stale rules from v2 guide (font-weight 700, radius 4/6-only, no ink tokens) | lingered in `design-guide.md` | **removed** — not migrated; v3-correct rules already live in the sections above |

The alias layer in `src/styles/globals.css` keeps legacy token names
working while consumers migrate. Never consume an alias from new code —
always reach for the current canonical name.

The alias layer in `src/styles/globals.css` remaps old names so pages
that still reference v2 tokens render correctly; migrate consumers one
page at a time.

---

## 15. Reference evidence

- `ref-9thave-home.png` — the9thave.com home, 1440 viewport, 2026-04-13.
- `cur-intro.png`, `cur-cia.png` — SecureStage pre-v3 baseline.
- Observed `:root` palette on 9thave home:
  `--green:#b9ff66; --black:#000; --dark:#191a23; --gray:#f3f3f3; --white:#fff; --brand-navy:#001B5E; --brand-purple:#432C7A; --brand-accent:#80366B`.
- Observed shadow: `rgb(0,27,94) 0px 5px 0px 0px` — exactly navy, vertical,
  5 px.
- Observed border widths: `1px solid rgb(0,0,0)` and
  `1px solid rgb(24,24,27)`.
- Observed corner radii: 7, 8, 16, 44–45 (pill). Our halved set: 4, 8, 12,
  pill.
- Observed weights: 400 and 500 only. No 600+.
