> English: [../../ADR/ADR-010-default-team-bootstrap.md](../../ADR/ADR-010-default-team-bootstrap.md)

# ADR-010: Default agentic team bootstrap (M14) — base + patch 2-layer

## Status

Proposed (2026-05-05). 사용자 ratify 게이트 — 승인 후 Accepted 로 전환.

## Context

ADR-001 (marker fence) + M2 (overlay merger) + M6 (10 patch overlay
템플릿) 까지 진행한 결과, genasis 의 동작 모델은 다음 전제 위에 서있다:

> 사용자의 `.claude/agents/<role>.md` 파일이 **이미 존재한다**.

`attach` 는 기존 파일에 marker fence 만 주입하고, `detach` 는 fence 만
제거하며, `upgrade` 는 fence 본문만 갱신한다. ECC /
knowledge-work-plugins / 자체 작성 팀을 가진 사용자에게는 정확히 맞다.

하지만 **agent 팀이 전혀 없는 빈 프로젝트** — 즉 `genasis init` 의 첫
대상 — 에서는 scaffold 경로가 비어있다. `cmd_init` 은 Plane/Mattermost
provisioning 만 수행하고, `.claude/agents/*.md` 파일은 사용자가 직접
작성해야 한다. 결과:

- blueprint §15 (1차 릴리즈 범위) 가 ECC 를 사실상 reference 사용자로
  가정해 "agent 파일 이미 있음" 이 암묵적 전제였음.
- `tests/golden/blank/` 픽스처가 M0 부터 README 만 있고 input/expected 가
  비어있는 stub 상태로 방치됨.
- README Comparison 표가 "Non-destructive overlay" 만 강조하고 "Bootstrap"
  차원이 없어 ECC `claude-code-templates` 와의 차별점이 시각적으로
  설명되지 않음.

2026-05-05 사용자 제기로 이 갭을 닫는 결정.

## Alternatives

| 대안 | 채택 여부 | 사유 |
|---|---|---|
| (a) `attach` 가 빈 `.claude/agents/` 감지 시 자동 scaffold (default ON) | 거부 | 기존 사용자가 처음 attach 할 때 silent file 생성을 당하는 risk. ADR-001 의 비파괴 invariant 정신과 충돌. |
| (b) opt-in `--bootstrap` flag (default OFF) | **채택** | 사용자 보호 + 명시적 의도 + green-field 팀에는 한 줄 명령으로 진입 가능. |
| (c) `init --bootstrap` (init 하위) | 부분 채택 | init 은 이미 Plane/MM provisioning 으로 무거움. 혼동 risk. |
| (d) 별도 `genasis bootstrap` 서브커맨드 | **채택** | 진입점이 명확. `init --bootstrap` 도 alias 로 유지 가능. |
| (e) ECC `claude-code-templates` 콘텐츠 vendor | 거부 | 라이선스 obligation + 유지보수 부담. base 템플릿은 5~10줄 stub 으로 의도적으로 얇게 작성, patch fence 가 protocol 본문을 채움. |
| (f) sidecar `.claude/agents/<role>.genasis.md` 별도 파일 | 거부 | Claude Code sub-agent 가 sibling 파일을 읽지 않음 (ADR-001 §Alternatives 참조). |

## Decision

**(b) + (d) + (e-rejected)** 를 채택:

1. **default OFF**: `genasis attach` 는 빈 `.claude/agents/` 를 만나도
   silent scaffold 하지 않는다. 대신 stderr 안내:
   > no agents detected — run `genasis bootstrap` (or `genasis init
   > --bootstrap`) to scaffold the default team.

2. **2-layer 구조**:

   | Layer | 위치 | 소유권 | 갱신 트리거 |
   |---|---|---|---|
   | **Base** | `.claude/agents/<role>.md` 의 marker fence **밖** (frontmatter + 역할 헤더 5~10줄) | 사용자 | `bootstrap` 1회 emit, 이후 자유 편집 |
   | **Patch** | 같은 파일의 marker fence **안** (Plane/MM 프로토콜 본문) | genasis | `attach` / `upgrade` 가 hash diff 로 갱신 |

3. **진입점**: `genasis bootstrap [--lang en|ko] [--roles <list>]` 신규
   서브커맨드 + `genasis init --bootstrap` alias. Bootstrap 후 자동으로
   `attach` 가 이어 호출되어 fence 까지 주입됨 (사용자가
   `--no-attach-after` 로 분리 가능).

4. **Base 템플릿 contract**: `templates/{en,ko}/agents/<role>.md.tera`
   는 5 키 frontmatter (`name/description/tools/model/color`) + 5~10줄
   역할 헤더만 포함. ECC content 를 vendor 하지 않는다 — protocol 살은
   patch overlay 가 fence 안에서 채운다.

5. **role set**: `Role::ALL` (M2) 의 10 역할 — pm / planner / architect
   / frontend / backend / qa / designer / security / devops /
   code-reviewer.

6. **i18n**: `templates/en/agents/` + `templates/ko/agents/` 2 트리.
   기존 `lang switch` 의 `templates/<lang>/` swap 메커니즘을 그대로
   재사용. 사용자가 base (fence 밖) 를 편집했다면 swap 시 보존 — 기존
   fence-internal-only 갱신 정책 그대로.

## Consequences

**쉬워짐**:
- 빈 프로젝트에서 한 줄로 ECC 호환 팀 scaffold (`genasis bootstrap`).
- `tests/golden/blank/` 픽스처 활성화 → green-field round-trip 회귀
  검증.
- README Comparison 표에 "Bootstrap" 차원 추가 가능 — ECC
  `claude-code-templates` 와의 시각적 차별화.
- ADR-001 의 marker fence invariant 그대로 유지 — bootstrap 은 단지
  "fence 가 들어갈 file 자체가 없을 때 빈 base 를 떨어뜨린다" 는 얇은
  추가 stage.

**어려워짐**:
- Base 템플릿 20개 (10 role × 2 lang) 의 frontmatter contract 일관성을
  단위 테스트로 강제해야 함 (M14.1).
- `genasis bootstrap` 이 conflict 를 만났을 때 (예: 일부 역할만 존재)
  의 부분 scaffold 시멘틱 정의 — 부재 역할만 `Create`, 존재 역할은
  `Skip("exists")`. `--overwrite` 는 의도적으로 미제공 (사용자가
  `detach` 후 `bootstrap` 로 명시적 의사 표현).

**Foreclosed**:
- Auto-scaffold (default ON) 경로 — 기존 사용자 보호와 충돌.
- ECC content vendor — 라이선스 / 유지보수 부담.

## References

- Implementation: `crates/genasis-overlay/src/bootstrap.rs` (M14.2)
- Templates: `crates/genasis-templates/templates/{en,ko}/agents/` (M14.1)
- CLI: `crates/genasis-cli/src/cmd_bootstrap.rs` 또는 `cmd_init.rs --bootstrap` (M14.3)
- Golden fixture: `tests/golden/blank/` (M14.4)
- Blueprint: `blueprint.ko.md` §20
- Progress: `progress.ko.md` §M14
- Predecessor ADRs: ADR-001 (marker fence — invariant 보호), ADR-008
  (i18n install-time selector — `--lang` 우선순위 재사용)
