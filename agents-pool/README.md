# agents-pool (private)

> This directory is a git submodule pointing to
> `git@github.com:claude-genasis/agents-pool.git` (private).
> General users will not see this content after `git clone`.

## Purpose

Curation pipeline for the genasis agents catalog:
1. **Crawl** — fetch latest agent .md files from community repos (ECC, wshobson, VoltAgent, dl-ezo, 0xfurai)
2. **Verify** — validate frontmatter, check for conflicts with genasis overlay injection
3. **Publish** — copy verified files to `../agents/base/` in the genasis public repo

## Usage (developer only)

```bash
cd agents-pool/
./scripts/crawl.sh      # shallow-clone all source repos
./scripts/verify.sh     # validate + copy to verified/
./scripts/publish.sh    # copy to ../agents/base/

# Then in genasis root:
cd ..
git add agents/
git commit -m "feat(agents): update catalog from pool"
git tag agents-v1.1.0
git push --tags         # CI creates release + tarball
```

## Configuration

Edit `config.toml` to add/remove source repositories.

---

## Secrets Management & Contribution Governance (GitHub Setup)

The genasis public repo uses a **Data-Only PR Model** (ADR-012) for debug
history contributions. This section documents how to configure the GitHub
repository to enforce the separation between data contributors and
automated code development.

### 1. Branch Protection Rules

```
Settings → Branches → Branch protection rules → Add rule
```

**Rule: `main`**
- ✅ Require a pull request before merging
- ✅ Require approvals: 1 (maintainer self-approve for auto-PRs)
- ✅ Require status checks to pass before merging:
  - `ci` (main CI workflow)
  - `lint-i18n` (mirror drift check)
  - `debug-history-validate` (JSON schema check for debug PRs)
- ✅ Require branches to be up to date before merging
- ✅ Restrict who can push to matching branches → Maintainer only
- ❌ Do NOT require signed commits (blocks auto-development flow)

### 2. CODEOWNERS File

Create `.github/CODEOWNERS`:

```
# Everything: maintainer approval required
* @<maintainer-username>

# Exception: debug-history patches can be submitted by anyone
# (but CI validates schema, and maintainer still approves PR)
# No CODEOWNERS override here — maintainer reviews all PRs anyway
```

### 3. GitHub Actions Workflow for Debug History PRs

Create `.github/workflows/debug-history-pr.yml`:

```yaml
name: Validate debug-history PR

on:
  pull_request:
    paths:
      - 'debug-history/patches/**'

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Verify only patch.json files changed
        run: |
          # Get list of changed files
          CHANGED=$(gh pr diff ${{ github.event.pull_request.number }} --name-only)

          # Every changed file must be under debug-history/patches/ and end with .patch.json
          echo "$CHANGED" | while read -r file; do
            if [[ ! "$file" =~ ^debug-history/patches/.*\.patch\.json$ ]]; then
              echo "❌ BLOCKED: $file is not a valid debug-history patch path"
              echo "   Only debug-history/patches/*.patch.json files are allowed in debug-history PRs"
              exit 1
            fi
          done
          echo "✅ All changed files are valid patch paths"
        env:
          GH_TOKEN: ${{ github.token }}

      - name: Validate JSON schema
        run: |
          for f in $(git diff --name-only origin/main -- 'debug-history/patches/*.patch.json'); do
            # Validate required fields
            jq -e '.schema_version and .project_hash and .genasis_version and .patches' "$f" || {
              echo "❌ $f: missing required fields (schema_version, project_hash, genasis_version, patches)"
              exit 1
            }

            # Reject executable content
            if grep -qP '^\s*#!' "$f" || grep -qP '`[^`]*\b(rm|curl|wget|eval|exec)\b' "$f"; then
              echo "❌ $f: contains suspicious executable content"
              exit 1
            fi
          done
          echo "✅ All patches pass schema validation"

      - name: Auto-label
        uses: actions/github-script@v7
        with:
          script: |
            github.rest.issues.addLabels({
              owner: context.repo.owner,
              repo: context.repo.repo,
              issue_number: context.issue.number,
              labels: ['debug-history']
            })

      - name: Auto-assign maintainer
        uses: actions/github-script@v7
        with:
          script: |
            github.rest.issues.addAssignees({
              owner: context.repo.owner,
              repo: context.repo.repo,
              issue_number: context.issue.number,
              assignees: ['<maintainer-username>']
            })
```

### 4. Repository Secrets (for automated development)

```
Settings → Secrets and variables → Actions
```

| Secret | Purpose | Used by |
|---|---|---|
| `ANTHROPIC_API_KEY` | Claude Code automation for `/debug-review` | `debug-review.yml` (maintainer-only workflow) |
| `GH_PAT_AUTOPR` | Create PRs from automated analysis | `debug-review.yml` |

These secrets are ONLY used by maintainer-triggered workflows, never by
contributor PRs.

### 5. Rulesets (GitHub Rulesets — alternative to branch protection)

If using GitHub Rulesets (newer, more granular):

```
Settings → Rules → Rulesets → New ruleset
```

**Ruleset: "debug-history-data-only"**
- Target: branches matching `main`
- Bypass list: Maintainer (for auto-development merges)
- Rules:
  - Restrict file path patterns that can be modified:
    - Contributors can ONLY modify: `debug-history/patches/**/*.patch.json`
  - Require pull request
  - Require status checks: `debug-history-validate`

> ⚠️ Note: GitHub Rulesets file-path restrictions are available on
> GitHub Enterprise and Team plans. On Free/Pro plans, use the
> workflow-based validation above instead.

### 6. Issue Labels Setup

```bash
# Create required labels
gh label create "debug-history" --color "0E8A16" --description "Debug history patch submission"
gh label create "auto-development" --color "5319E7" --description "PR auto-generated by /debug-review"
gh label create "needs-review" --color "FBCA04" --description "Auto-PR awaiting maintainer review"
```

### 7. Issue Template for Debug History

Create `.github/ISSUE_TEMPLATE/debug-history.yml`:

```yaml
name: Debug History Submission
description: Submit a debug-history patch from genasis debug submit
labels: ["debug-history"]
assignees:
  - <maintainer-username>
body:
  - type: textarea
    id: patch-json
    attributes:
      label: Patch JSON
      description: Paste the patch.json content generated by `genasis debug collect`
      render: json
    validations:
      required: true
  - type: textarea
    id: context
    attributes:
      label: Context (optional)
      description: What were you trying to do when you made this change?
    validations:
      required: false
  - type: checkboxes
    id: security-confirm
    attributes:
      label: Security checklist
      options:
        - label: I have verified the patch contains no secrets, tokens, or source code
          required: true
        - label: I have verified absolute paths are anonymised
          required: true
```

### 8. Maintainer Auto-Development Workflow

Create `.github/workflows/debug-review.yml` (maintainer-only, manual trigger):

```yaml
name: Debug Review (auto-development)

on:
  workflow_dispatch:
    inputs:
      mode:
        description: 'Analysis mode'
        required: true
        default: 'propose'
        type: choice
        options:
          - propose  # Generate analysis only
          - draft-pr # Generate analysis + create draft PR with fixes

permissions:
  contents: write
  pull-requests: write

jobs:
  review:
    runs-on: ubuntu-latest
    if: github.actor == '<maintainer-username>'
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Count unresolved patches
        id: count
        run: |
          COUNT=$(find debug-history/patches/ -name "*.patch.json" | wc -l)
          echo "patch_count=$COUNT" >> $GITHUB_OUTPUT
          if [ "$COUNT" -eq 0 ]; then
            echo "No unresolved patches. Exiting."
            exit 0
          fi

      - name: Run /debug-review via Claude Code
        if: steps.count.outputs.patch_count > 0
        env:
          ANTHROPIC_API_KEY: ${{ secrets.ANTHROPIC_API_KEY }}
        run: |
          # Install Claude Code CLI
          npm install -g @anthropic-ai/claude-code

          # Run the debug-review skill
          claude --skill debug-review --non-interactive \
            --output debug-history/analysis/

      - name: Create draft PR (if draft-pr mode)
        if: inputs.mode == 'draft-pr' && steps.count.outputs.patch_count > 0
        env:
          GH_TOKEN: ${{ secrets.GH_PAT_AUTOPR }}
        run: |
          BRANCH="auto/debug-review-$(date +%Y%m%d)"
          git checkout -b "$BRANCH"
          git add -A
          git commit -m "feat(templates): auto-fix from debug-history analysis

          Patches analysed: $(find debug-history/patches/ -name '*.patch.json' | wc -l)
          Generated by /debug-review skill.

          Co-Authored-By: Claude Code <noreply@anthropic.com>"
          git push -u origin "$BRANCH"
          gh pr create --title "Auto: template fixes from debug-history" \
            --body "Generated by \`/debug-review\` workflow. Review carefully." \
            --label "auto-development,needs-review" \
            --draft
```

### 9. Complete Setup Checklist

```markdown
- [ ] Branch protection on `main` with required status checks
- [ ] `.github/CODEOWNERS` file committed
- [ ] `.github/workflows/debug-history-pr.yml` committed
- [ ] `.github/workflows/debug-review.yml` committed (maintainer trigger only)
- [ ] `.github/ISSUE_TEMPLATE/debug-history.yml` committed
- [ ] `debug-history/` directory created (index.jsonl + patches/ + analysis/ + schema.json)
- [ ] GitHub labels created (debug-history, auto-development, needs-review)
- [ ] Repository secrets set (ANTHROPIC_API_KEY, GH_PAT_AUTOPR)
- [ ] `.claude/skills/debug-review.md` skill created in genasis repo
- [ ] Test: contributor submits a PR to debug-history/patches/ → CI validates → auto-label works
- [ ] Test: maintainer triggers debug-review workflow → analysis generated
```

### 10. Security Notes

- **Contributors NEVER get write access to templates, overlays, or
  genasis code.** They can only add files to `debug-history/patches/`.
- **Auto-development PRs always land as DRAFT.** Maintainer must
  explicitly review and approve.
- **`ANTHROPIC_API_KEY`** is only exposed to the maintainer-only
  `debug-review.yml` workflow (protected by `if: github.actor ==`).
- **No `pull_request_target`** trigger is used (avoids the common
  GitHub Actions security pitfall of running untrusted code with write
  permissions).
