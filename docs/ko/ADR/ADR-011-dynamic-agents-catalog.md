> English: [../../ADR/ADR-011-dynamic-agents-catalog.md](../../ADR/ADR-011-dynamic-agents-catalog.md)

# ADR-011: Dynamic Agents Catalog — 바이너리 임베드 제거 + 런타임 fetch

## Status

Accepted (2026-05-05).

## Context

ADR-001 (marker fence) + M6 (10 overlay 템플릿) + M14 (bootstrap) 까지의
아키텍처는 **모든 agent 지침 파일을 `include_dir!()` 로 바이너리에
컴파일 타임 임베드**하는 전제 위에 서있다.

문제:
1. **업데이트 지연**: agent 지침을 수정하려면 genasis 바이너리 자체를
   새 버전으로 릴리즈해야 함. agent content의 라이프사이클이 도구의
   라이프사이클에 종속.
2. **커뮤니티 반영 불가**: `docs/famous-agents.md` 조사 결과, ECC /
   wshobson / VoltAgent / dl-ezo 등에서 best-of-breed agent를 큐레이션해
   빠르게 배포하고 싶으나 바이너리 재빌드 없이는 불가.
3. **큐레이션 분리 불가**: private repo(agents-pool)에서 crawl → verify →
   publish 하는 파이프라인이 필요하지만, 현재 구조에서는 template이
   코드와 결합되어 PR/CI가 무거움.
4. **Base agent 파일이 불필요하게 .tera**: 분석 결과 Tera 변수를 하나도
   사용하지 않는 정적 텍스트였음. plain .md로 충분.

## Alternatives

| 대안 | 채택 여부 | 사유 |
|---|---|---|
| (a) `include_dir!()` fallback 유지 (hybrid) | 거부 | 이중 관리 복잡. 내장본과 fetch본의 버전 불일치 risk. 바이너리 크기 불필요 유지. |
| (b) GitHub Releases tarball (agents-v1.x.tar.gz) | **채택** | 바이너리와 완전 독립 버전. 기존 release 인프라 활용. SHA-256 검증 가능. |
| (c) npm 패키지 (@genasis/agents) | 거부 | Node.js 의존성 발생. genasis의 "Rust 단일 바이너리" 원칙(ADR-002) 위배. |
| (d) GitHub raw URL fetch (브랜치 기반) | 거부 | rate limit, 버전 고정 불가, 캐시 무효화 어려움. |
| (e) Agent base를 .tera로 유지 | 거부 | 변수 0건 사용. 불필요한 렌더링 단계가 복잡성만 증가. |
| (f) Overlay도 plain .md로 전환 | 거부 | overlay는 `{{ project_name }}`, `{{ mm_url }}` 등 프로젝트별 변수 7~10개 사용. Tera 필수. |

## Decision

**(b) + (e-rejected) + (f-rejected)** 를 채택:

### 1. 구조 분리

| Layer | 형식 | 소스 | 배포 |
|---|---|---|---|
| Agent base 파일 | **plain .md** | agents-pool(private) → genasis/agents/base/ | GitHub Release tarball |
| Overlay patch | **.tera** (유지) | genasis/agents/overlays/{en,ko}/ | 같은 tarball에 포함 |
| Commands/Skills/Hooks | .tera (유지) | genasis/agents/{commands,skills,hooks}/ | 같은 tarball에 포함 |

### 2. 배포 흐름

```
agents-pool(private, submodule) → crawl → verify → publish.sh
  → genasis/agents/ 에 복사 → commit → tag agents-v1.x.0
  → CI(release-agents.yml)가 tarball 생성 → GitHub Release asset 업로드
```

### 3. CLI fetch 모델

- `genasis agents fetch [--version]`: 명시적 다운로드
- `genasis attach` / `upgrade`: auto-check (pinned version vs cache)
- Cache: `~/.cache/genasis/agents/v{version}/`
- Pin: `genasis.toml [agents].version = "1.0.0"`

### 4. `include_dir!()` 완전 제거

바이너리에 template/agent 내장 없음. 네트워크 접근 불가 + 캐시 없음 →
에러 + "run `genasis agents fetch` first" 안내.

### 5. 기본 9역할 (famous-agents.md 근거)

pm, architect, frontend-developer, backend-developer, code-reviewer,
qa-tester, security-reviewer, planner, designer — 커뮤니티
best-of-breed(ECC/wshobson/VoltAgent/dl-ezo) + genasis 자체(pm, designer).

### 6. 소통 규칙 주입 보장

모든 base agent에 overlay fence가 자동 주입되며, 그 안에 Plane/MM
lifecycle protocol + GENASIS.md 참조 링크가 포함. PM과 Designer는
소통 규칙의 핵심 허브 (이슈 배정 / 디자인 변경 알림).

## Consequences

**쉬워짐:**
- Agent content 업데이트가 바이너리 릴리즈와 독립. `agents-v1.1.0` 태그
  만으로 모든 사용자가 `genasis agents update` 한 줄로 반영.
- 커뮤니티 best-of-breed를 빠르게 큐레이션 → 배포.
- 바이너리 크기 감소 (template 임베드 제거).
- Agent base 관리가 plain .md로 단순화.

**어려워짐:**
- 첫 사용 시 네트워크 필수 (`install.sh`가 `genasis agents fetch` 포함).
- CI 환경에서 cache warm-up 필요 (CI cache 연동 권장).
- `genasis-templates` crate 대규모 리팩토링 (include_dir → HTTP client + cache).

**Foreclosed:**
- Offline-only 배포 (캐시 사전 준비 없이는 불가).
- 바이너리 단독으로 agent 설치 완결 (항상 catalog fetch 필요).

## References

- `docs/famous-agents.md` — 커뮤니티 agent 조사 (선정 근거)
- ADR-001 (marker fence) — overlay 주입 메커니즘 유지
- ADR-002 (Rust 단일 바이너리) — npm 의존성 거부 근거
- ADR-010 (default team bootstrap) — 이 ADR로 대체 (M14 → 이 구조로 흡수)
- `blueprint.ko.md` §20 → §21로 갱신 예정
