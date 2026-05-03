# Impact of Multilingual Prompts on Claude Agents

> Research conducted 2026-05-04 to inform genasis M12 (i18n) architecture.
> Question: *What happens when a Claude-based agent's `.claude/agents/*.md`,
> `CLAUDE.md`, slash commands, and skills contain the same instructions in
> multiple natural languages simultaneously (e.g., Korean + English)?*
>
> TL;DR — **Don't ship both languages in active context at the same time.**
> Pick one at install time, treat the other as on-disk reference only.
> Empirical evidence below.

---

## 1. Anthropic's Official Guidance

Anthropic publishes performance numbers for Claude across 14 languages but
**no explicit guidance on mixing multiple languages in a single system
prompt**. The relevant sources:

### 1.1 Multilingual support page

The official [Multilingual support](https://platform.claude.com/docs/en/build-with-claude/multilingual-support)
docs confirm Claude is robust across languages but the entire "Best
practices" section is about *single-language* usage:

> **Provide clear language context**: While Claude can detect the target
> language automatically, explicitly stating the desired input/output
> language improves reliability. […]
> **Use native scripts**: Submit text in its native script rather than
> transliteration for optimal results.

The performance table (zero-shot CoT, % of English baseline) shows Korean
at 96.7% on Sonnet 4.5 and 93.3% on Haiku 4.5 — strong, but Anthropic
deliberately frames this as "test with the languages relevant to your
specific use cases", i.e. **per-deployment, not all-at-once**
([source](https://platform.claude.com/docs/en/build-with-claude/multilingual-support)).

### 1.2 Prompt caching

Prompt caching is **byte-for-byte prefix matching**
([source](https://docs.anthropic.com/en/docs/build-with-claude/prompt-caching)).
Implications for genasis:

- Korean and English versions of the same protocol document hash
  differently — they cannot share a cache prefix.
- If both languages live in the prefix, you pay the **cache write** cost
  (1.25× input) for both, and read both back on every turn → **double
  context bloat with no caching benefit**.
- The 5-minute and 1-hour TTLs make this worse over a long Sprint
  session: the model re-reads the whole multilingual blob on each cache
  miss.

### 1.3 Claude Code i18n status

Claude Code has **no built-in interface localization** as of May 2026.
Issue [#4866](https://github.com/anthropics/claude-code/issues/4866) is
the open feature request for `claude config set language ja` style CLI
locale; it has no maintainer engagement yet. Until then, the *only*
in-product mechanism for "stay in language X" is a sentence inside
`CLAUDE.md` — and as §2.1 below shows, that sentence is not always
respected.

**Implication for genasis**: We cannot rely on Claude Code itself to
route between language variants. Genasis has to make the choice at
install time and produce a single coherent set of files.

---

## 2. Empirical & Community Findings

### 2.1 Claude Code drifts language even with explicit instructions

[Issue #46846](https://github.com/anthropics/claude-code/issues/46846)
documents a **reproducible Claude Code regression** where a user's
`CLAUDE.md` says (in Traditional Chinese):

> 永遠使用台灣正體中文回覆，不得使用日文、韓文或簡體中文。
> *(Always respond in Traditional Chinese. Must not use Japanese,
> Korean, or Simplified Chinese.)*

Claude Code still responded **entirely in Japanese** after a `git push`,
then attempted to apologize — *also in Japanese*:

> commit と push 完了。lint-staged の index.lock エラーは…
> 申し訳ありません、日本語になっていました。再度、正體中文で：

The user's project contains **only Traditional Chinese and English** —
no Japanese in context at all. The drift is triggered by tool output
(English `git`/`gh` JSON), and a single negative instruction ("don't use
Japanese") is not strong enough to override the model's pull toward
training-distribution-dominant East Asian technical text
([source](https://github.com/anthropics/claude-code/issues/46846)).

[Issue #24941](https://github.com/anthropics/claude-code/issues/24941)
reports the symmetric Korean→Japanese drift mid-response, also
unresolved.

**Implication for genasis**: If we ship both Korean and English
*overlay fences* into the same agent file, the model will probabilistically
sample whichever language has more nearby tokens at any given point —
**effectively guaranteed code-switching** in long sessions.

### 2.2 Academic: line-level and word-level language confusion

The "Understanding and Mitigating Language Confusion in LLMs" paper
([arXiv 2406.20052](https://arxiv.org/html/2406.20052v1)) defines two
failure modes that matter for us:

| Failure | Definition | Trigger |
|---|---|---|
| **Line-level confusion** | Whole lines slip into the wrong language | Distribution over next tokens is flat (avg nucleus 3.56 vs 1.61 at non-confusion points) |
| **Word-level confusion** | Stray English words inside Korean/Japanese/Chinese output | Prevalent in *all* models tested, including Claude-class |

Korean and Japanese are explicitly named as **most vulnerable** — Llama
3 Instruct hit **0% line-level pass rate on Korean** in the monolingual
evaluation. Claude isn't tested in this paper but the mechanism
(confidence collapse at decode time) is model-agnostic.

The paper found **isolated language instructions (placed at the start or
end of the prompt) outperformed integrated instructions by 15–20%** —
strong evidence that *one* clearly-positioned instruction beats
*scattered* multilingual content.

The "Lost in the Mix" study
([arXiv 2506.14012](https://arxiv.org/html/2506.14012v1)) adds: *embedding
non-English tokens into an English matrix language **consistently
degrades** comprehension*. So even an English-default agent file with
"sprinkled" Korean comments measurably hurts the model.

**Implication for genasis**: A bilingual agent file isn't just visually
noisy — it actively reduces task accuracy.

### 2.3 Industry write-ups corroborate

A practitioner write-up
([dev.to: AI Language Drift](https://dev.to/stevengonsalvez/ai-language-drift-when-your-discord-bot-randomly-replies-in-mandarin-2i88))
puts it bluntly:

> The model's internal sense of "helpful assistant response" has a
> strong training language pull, and if the prompt in another language
> isn't assertive enough, it drifts back to what feels most natural.

The fix the author landed on: a **single, top-of-prompt language lock**,
no localized duplicates further down.

---

## 3. Real-World i18n Patterns in Agent Frameworks

### 3.1 Major Claude Code template repos: English-only

Surveying the leading Claude Code template ecosystems:

- **`hesreallyhim/awesome-claude-code`** — curated index of skills,
  hooks, slash-commands. **All entries English-only**. No locale
  dimension in the catalog
  ([source](https://github.com/hesreallyhim/awesome-claude-code)).
- **`aitmpl.com` (Claude Code Templates)** — 1000+ pre-built components.
  **English-only**. No `--lang` option in the CLI installer
  ([source](https://www.aitmpl.com/)).
- **`rohitg00/awesome-claude-code-toolkit`** — 135 agents, 35 skills,
  176+ plugins. **English-only**
  ([source](https://github.com/rohitg00/awesome-claude-code-toolkit)).
- **`Piebald-AI/claude-code-system-prompts`** — extracted Claude Code's
  *own* system prompts. **All English** even though Claude Code is used
  worldwide
  ([source](https://github.com/Piebald-AI/claude-code-system-prompts)).

The de facto OSS norm is "ship English, let the user instruct Claude in
their language at conversation time." Zero major template projects ship
localized agent prompt sets.

### 3.2 The translation-proxy pattern: `claude-ts` (claude-kr)

The one notable Korean-specific project,
[`kimi230/claude-ts`](https://github.com/kimi230/claude-ts), takes a
**translation proxy** approach instead of localizing prompts:

> You (any language) → Haiku/Ollama (→ EN) → Claude Code (EN context)
> → Haiku/Ollama (→ your language) → You

The README is explicit about *why*:

> Claude Code always works in English internally, so it reasons better
> and uses fewer tokens.

**`claude-ts` does not modify CLAUDE.md or install Korean prompt
templates** — it only translates I/O. This validates the "single
internal language, user-facing translation at the edges" pattern over
"localized templates".

### 3.3 Anthropic's own progressive disclosure

The [Claude API skill](https://platform.claude.com/docs/en/agents-and-tools/agent-skills/claude-api-skill)
uses **progressive disclosure**: it loads only the doc fragments
relevant to the user's language *(programming language, not natural
language)*, surface, and task — *not* everything at once. The
architectural lesson generalizes: **scope context to one variant, don't
union them**.

---

## 4. Specific Failure Modes if We Ship Both Languages

Mapping the evidence onto genasis's specific install model:

| # | Failure mode | Evidence | Severity for genasis |
|---|---|---|---|
| F1 | **Output language drift** — agent replies in the wrong language mid-response | Claude Code Issues [#46846](https://github.com/anthropics/claude-code/issues/46846), [#24941](https://github.com/anthropics/claude-code/issues/24941) | High — breaks the Mattermost protocol where threads are expected in one language |
| F2 | **Instruction divergence** — Korean and English fences slowly drift apart on edits | Common-sense + git history reality | High — silent contract bugs, e.g. one says "PR required for main", the other says "direct push allowed" |
| F3 | **Prompt cache invalidation** — both prefixes never hash-match, both pay write cost | [Prompt caching docs](https://docs.anthropic.com/en/docs/build-with-claude/prompt-caching) | Medium — visible in token bills, doubles cache write cost |
| F4 | **Context bloat** — every agent file ~2× larger | Direct measurement | Medium — 10 role overlays × 2 languages = 20 fences in CLAUDE.md prefix |
| F5 | **Tool-use parameter language confusion** — commit messages in mixed Korean/English, Plane state names mistranslated | [arXiv 2406.20052](https://arxiv.org/html/2406.20052v1) word-level confusion | High — Plane state IDs are ASCII, but `plane.state.name` text leaks into prompts; Mattermost messages are user-facing |
| F6 | **Agent code-switching with no clear cause** — `git push` triggers Japanese drift | [#46846](https://github.com/anthropics/claude-code/issues/46846) shows even *removing* the wrong language from context isn't a guarantee; *adding* it makes it worse | High — debuggability collapses |

**F2 deserves special attention.** Genasis fences are protocol contracts
("only your assignee owns lifecycle transitions", "merge with `gh pr
merge --squash --delete-branch`"). A bilingual fence where the Korean
version drifts to say `--rebase` while the English version still says
`--squash` becomes a **silent ownership/merge bug** that shows up only
when the agent picks the wrong language at decode time. There is no
existing tooling to detect protocol-level translation drift between two
prose documents.

---

## 5. Recommended Architecture for Genasis

### 5.1 Default: **install one language, keep the other on disk only**

```
┌─────────────────────────────────────────────────────────────┐
│ genasis init --lang ko    OR    genasis init --lang en      │
│            (default: --lang $(detect from $LANG))           │
└─────────────────────────────────────────────────────────────┘
                              │
                ┌─────────────┴─────────────┐
                ▼                           ▼
    ACTIVE (in agent context)      REFERENCE (on disk only)
    ─────────────────────────      ───────────────────────────
    .claude/agents/*.md            docs/i18n/<other-lang>/...
    .claude/genasis/skills/        (NOT @import'd, NOT in
    .claude/genasis/commands/       any fence, NOT in CLAUDE.md)
    GENASIS.md  ← @import'd
    Marker fences inside agents
```

**Rule**: Only the chosen language ever appears inside `@import` chains,
`.claude/agents/*.md` fences, `CLAUDE.md`, or any file Claude Code
loads. The other language ships as **passive reference docs** under
`docs/<lang>/` for the human maintainer.

This matches both Anthropic's progressive disclosure principle (§3.3)
and the empirical observation that **single-language context is
strictly safer** (§2.2, §4).

### 5.2 If the user wants both at runtime — refuse, with a recovery path

When `genasis init --lang both` is invoked, the CLI should **reject
with an explanation** and offer two alternatives:

1. **`genasis init --lang en` then `genasis lang switch ko`** — install
   English first, swap atomically when needed (`switch` swaps the entire
   set: agent overlays, skills, commands, GENASIS.md, fence bodies).
2. **`genasis init --lang en --reference-docs ko`** — install English
   active; place Korean copies in `docs/ko/` for human reading. *Not*
   loaded by Claude.

The CLI message should cite this document
(`docs/impact-of-multilang-prompts.md`) so the user understands the
reasoning rather than feeling artificially limited.

### 5.3 `genasis lang switch <lang>` — atomic locale swap

A first-class command for the team-level transition (e.g., a Korean
team adds an English-speaking contributor). It must be **atomic**:

```
genasis lang switch en
  ↓
1. Snapshot all current files containing GENASIS fences
2. For each fence: regenerate body from English template + new hash
3. Replace GENASIS.md, .claude/genasis/{skills,commands,hooks}/ wholesale
4. Update genasis.toml: [i18n] active = "en", previous = "ko"
5. Single git commit so detach/upgrade can roll back cleanly
6. Print: "✅ Switched ko → en. Restart Claude Code so the new context loads."
```

Critical detail from §1.2: **the cache prefix changes wholesale** — that
is fine, because *one* cache write happens and then the team benefits
from caching again. Versus the bilingual approach which **never
caches**.

### 5.4 Frontmatter contract — one of, not list of

Each genasis-owned file (overlay templates, skill SKILL.md, slash
command, GENASIS.md) carries a single-language declaration:

```yaml
---
name: scrum-protocol
description: Lifecycle, mention, DoD rules
genasis_lang: en          # ← scalar, not list
genasis_fence_version: 1.0
---
```

Not `genasis_lang: [en, ko]`. The schema enforces scalar to make F2
(divergence) impossible by construction.

### 5.5 Doctor & lint-i18n CI checks

`genasis doctor` adds an `[i18n]` section that flags **any file with
content in a language other than `genasis.toml [i18n] active`**. This
catches:

- Hand-edits where a Korean operator added Korean to an English file
- Forgotten Korean strings in English templates
- Mixed-language fences if a future bug ever produces them

The check is a regex sweep (`[\u{AC00}-\u{D7AF}]` for Hangul,
`[\u{3040}-\u{30FF}]` for Kana, etc.) gated to **Genasis-owned files
only** — user code is untouched.

CI mirrors this as `lint-i18n`, hard-fail on release pipelines, warn on
PRs (matches the existing tolerance philosophy in genasis CI).

### 5.6 Migration guidance for mixed-preference teams

The realistic case: a Korean-speaking team running genasis wants to onboard
a Japanese OSS contributor who reviews PRs.

- **Don't switch the agent context language per contributor.** That is
  a footgun and produces F2 drift.
- **The team's `genasis.toml [i18n] active` is the source of truth** for
  the agent runtime. Change it once, team-wide, with a `genasis lang
  switch` PR.
- Individual contributors read `docs/<their-lang>/` for human-facing
  documentation. The agent always speaks the team's chosen language in
  Mattermost / Plane.
- A team unwilling to commit to one language for the agent runtime
  should **stay on English** (lowest-drift, best-cached) and let humans
  translate at the edges (claude-ts pattern, §3.2).

---

## Recommendation

**Default architecture**: `genasis init --lang en|ko` (auto-detect from
`$LANG`, fall back to `en`). The choice produces a **single-language
overlay set** in `.claude/`. The other language exists only as
**non-`@import`ed reference docs** under `docs/<lang>/`.

`--lang both` is **rejected** with a citation to this document. Locale
switches happen via `genasis lang switch <lang>` — atomic, single
commit, full prefix rotation so prompt cache works.

**Justification, in order of weight**:

1. **F2 (instruction divergence)** is unbounded and silent. Two prose
   contracts maintained by hand will drift; protocol drift produces
   wrong agent behavior with no detection mechanism. Architecture must
   make this impossible, not improbable.
2. **Empirical F1 evidence**: Claude Code itself drifts language even
   with explicit single-language instructions
   ([#46846](https://github.com/anthropics/claude-code/issues/46846));
   adding the wrong language to context guarantees worse outcomes.
3. **Prompt cache cost (F3)**: bilingual prefix doubles cache writes
   and never matches — direct cash impact.
4. **OSS norm convergence**: every major Claude Code template ecosystem
   (§3.1) and the only Korean wrapper (§3.2) chose single-language
   internal context. This is not a contrarian choice.
5. **Reversibility**: `genasis lang switch` provides a clean upgrade
   path if a team's preference changes, without ever requiring both to
   coexist in active context.

Anthropic's strong multilingual benchmarks (§1.1) **support**, not
contradict, this choice — Claude is excellent in Korean *or* English
*independently*; it is **mixing** that breaks down.

---

## Sources

- [Anthropic — Multilingual support](https://platform.claude.com/docs/en/build-with-claude/multilingual-support)
- [Anthropic — Prompt caching](https://docs.anthropic.com/en/docs/build-with-claude/prompt-caching)
- [Anthropic — Claude API skill (progressive disclosure)](https://platform.claude.com/docs/en/agents-and-tools/agent-skills/claude-api-skill)
- [Claude Code Issue #46846 — Traditional Chinese requested, Japanese delivered](https://github.com/anthropics/claude-code/issues/46846)
- [Claude Code Issue #24941 — Korean → Japanese mid-response drift](https://github.com/anthropics/claude-code/issues/24941)
- [Claude Code Issue #4866 — i18n/l10n CLI feature request](https://github.com/anthropics/claude-code/issues/4866)
- [arXiv 2406.20052 — Understanding and Mitigating Language Confusion in LLMs](https://arxiv.org/html/2406.20052v1)
- [arXiv 2506.14012 — Lost in the Mix: Code-Switched Text Comprehension](https://arxiv.org/html/2506.14012v1)
- [kimi230/claude-ts — Korean ↔ English translation proxy for Claude Code](https://github.com/kimi230/claude-ts)
- [hesreallyhim/awesome-claude-code](https://github.com/hesreallyhim/awesome-claude-code)
- [aitmpl.com — Claude Code Templates](https://www.aitmpl.com/)
- [rohitg00/awesome-claude-code-toolkit](https://github.com/rohitg00/awesome-claude-code-toolkit)
- [Piebald-AI/claude-code-system-prompts](https://github.com/Piebald-AI/claude-code-system-prompts)
- [dev.to — AI Language Drift case study](https://dev.to/stevengonsalvez/ai-language-drift-when-your-discord-bot-randomly-replies-in-mandarin-2i88)
