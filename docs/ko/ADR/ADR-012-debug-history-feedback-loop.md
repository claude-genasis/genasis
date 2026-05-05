> English: [`../../ADR/ADR-012-debug-history-feedback-loop.md`](../../ADR/ADR-012-debug-history-feedback-loop.md)

# ADR-012: Debug History — 필드 드리프트를 자기개선 피드백 루프로 활용

## 상태

제안됨 (2026-05-05).

## 맥락

Genasis는 **메타 도구**다: 실제 프로젝트 내부에 agentic team 설정을 생성하고
관리한다. 사용자는 필연적으로 생성된 파일을 수정하게 된다:
- overlay 템플릿의 버그 수정 (잘못된 lifecycle 명령, 누락된 환경변수)
- 프로젝트별 워크플로 적응 (커스텀 스프린트 주기, 비표준 Plane 라벨, 추가 MM 채널)
- Genasis 템플릿이 제공하지 않는 에이전트 지시사항 확장
- genasis 한계점이나 엣지 케이스 우회

이 수정사항은 **Genasis를 개선하기 위한 가장 가치 있는 신호**다 — 도구가 어디서
부족한지, 실제 팀이 무엇을 필요로 하는지 정확히 알려준다. 현재 이 신호는 소실된다:
사용자가 로컬에서 고치고 업스트림에 보고하지 않는다.

### 문제 정의

사용자 소스 코드 보안을 침해하지 않으면서, Genasis가 자체 생성물에 대한 필드
수정사항을 체계적으로 수집하고, 최소한의(이상적으로는 제로) 개발자 노력으로
Genasis 개발에 피드백할 수 있는 방법은?

### 설계 원칙

1. **상시 작동, 무설정 수집** — 드리프트 추적은 별도 설정 없이 기본 동작이어야 한다.
2. **보안 우선** — 사용자 소스 코드, 시크릿, 프로젝트 신원은 명시적 옵트인 없이
   로컬 머신을 떠나지 않아야 한다.
3. **최소 개발자 노력** — 피드백 제출은 명령 하나, genasis 개선에 활용하는 것은
   기존 Claude Code 워크플로로 충분해야 한다.
4. **Claude Code 자기완결** — 디버그 히스토리는 Claude Code가 genasis 작업 시
   외부 도구나 인간 해석 없이 직접 읽을 수 있는 구조여야 한다.

## 대안

| 대안 | 결정 | 이유 |
|---|---|---|
| (a) GitHub Issues 템플릿 (수동 버그 리포트) | 기각 | 개발자 노력 필요; 대부분의 드리프트는 보고되지 않음; Claude Code가 파싱할 구조화된 형식 없음 |
| (b) 텔레메트리 서비스 (phone-home) | 기각 | 프라이버시 우려; 호스팅 인프라 필요; 사용자 불신 |
| (c) **로컬 매니페스트 + diff + 옵트인 제출** | **채택** | 인프라 불필요, 기본 보안, 기계 소비용 구조 |
| (d) 사용자 프로젝트 git hook으로 자동 PR | 기각 | 너무 침습적; genasis를 사용자 git 워크플로에 결합 |
| (e) Overlay 체크섬 경고만 (수집 없음) | 기각 | 드리프트 감지하지만 내용 폐기 — 신호 낭비 |

## 결정

### 아키텍처: 매니페스트-드리프트-제출 파이프라인

```
┌─────────────────────────────────────────────────────────┐
│  사용자 프로젝트 (항상 로컬, 항상 자동)                 │
│                                                         │
│  genasis attach / init                                  │
│    └─► .claude/genasis/.manifest.json                   │
│         (설치 시점 모든 관리 파일의 SHA-256)             │
│                                                         │
│  genasis doctor / 모든 CLI 호출                         │
│    └─► 드리프트 감지 (라이브 파일 vs 매니페스트 비교)   │
│    └─► .claude/genasis/.drift-log/<timestamp>.jsonl     │
│         (구조화된 diff 레코드, append-only)              │
│                                                         │
│  genasis debug collect                                  │
│    └─► ~/.genasis/debug-history/<project-hash>/         │
│         └─► <timestamp>.patch.json                      │
│              (익명화, 소스 제거, overlay 범위만)         │
└─────────────────────────────────────────────────────────┘
          │
          │  genasis debug submit (옵트인, 인터랙티브 확인)
          ▼
┌─────────────────────────────────────────────────────────┐
│  GENASIS 리포                                           │
│                                                         │
│  debug-history/                                         │
│    ├── index.jsonl      (append-only 패치 레지스트리)   │
│    ├── patches/                                         │
│    │   ├── 2026-05-05_a1b2c3d4.patch.json              │
│    │   ├── 2026-05-06_e5f6g7h8.patch.json              │
│    │   └── ...                                          │
│    └── analysis/                                        │
│        ├── clusters.md  (자동 생성 패턴 그룹)           │
│        └── proposed-fixes.md (Claude Code 제안)         │
│                                                         │
│  .claude/skills/debug-review.md                         │
│    (debug-history/ 읽고 수정 제안하는 스킬)             │
└─────────────────────────────────────────────────────────┘
```

### 1. 매니페스트 생성 (`attach` / `init` 시점)

genasis가 overlay 파일을 생성하거나 주입할 때 매니페스트를 기록한다:

```json
{
  "genasis_version": "0.2.0",
  "agents_catalog_version": "1.3.0",
  "attached_at": "2026-05-05T14:30:00Z",
  "lang": "ko",
  "files": {
    ".claude/genasis/skills/plane-ops.md": {
      "sha256": "a1b2c3d4...",
      "template_source": "agents/skills/plane-ops.md.tera",
      "size_bytes": 2048
    },
    ".claude/agents/frontend.md": {
      "fence_sha256": "e5f6g7h8...",
      "fence_start_line": 5,
      "fence_end_line": 22
    }
  }
}
```

핵심:
- genasis가 관리하는 파일만 추적 (overlay 범위)
- agent 파일의 경우 marker-fenced 섹션만 추적 (사용자 콘텐츠 제외)
- `.claude/genasis/.manifest.json`에 저장

### 2. 드리프트 감지 (수동적, 매 CLI 호출 시)

모든 `genasis` 명령 실행 시(`doctor`, `monitor` 등 포함) CLI가 조용히 현재
파일 상태를 매니페스트와 비교한다. 드리프트 감지 시:

```jsonl
{"ts":"2026-05-06T09:15:00Z","file":".claude/genasis/skills/plane-ops.md","type":"content_modified","old_hash":"a1b2c3d4","new_hash":"f9e8d7c6","diff_lines":4}
{"ts":"2026-05-06T09:15:00Z","file":".claude/genasis/hooks/session-start.sh","type":"deleted","old_hash":"11223344"}
{"ts":"2026-05-07T11:00:00Z","file":".claude/genasis/skills/custom-deploy.md","type":"added","new_hash":"55667788"}
```

`.claude/genasis/.drift-log/current.jsonl`에 append (로컬 전용, 사용자가
커밋하지 않음). 비용: 관리 파일당 SHA-256 1회/CLI 호출 (일반적 20파일
overlay에서 < 1ms).

### 3. Debug Collect (명시적, 로컬 집계)

```bash
genasis debug collect
```

이 명령은:
1. `.drift-log/current.jsonl` 읽기
2. 수정된 각 파일에 대해 overlay 범위 콘텐츠의 **unified diff** 생성
3. 보안 제외 패턴에 매칭되는 콘텐츠 제거:
   - `TOKEN`, `SECRET`, `KEY`, `PASSWORD`, `CREDENTIAL` 포함 라인 (대소문자
     무관) → `[REDACTED]`로 대체
   - 절대 경로 → `<PROJECT_ROOT>/...`로 대체
   - `.env` 변수 값 → `[ENV_VALUE]`로 대체
4. `patch.json` 생성:

```json
{
  "schema_version": 1,
  "project_hash": "프로젝트-경로의-단방향-해시",
  "genasis_version": "0.2.0",
  "agents_catalog_version": "1.3.0",
  "os": "linux-x86_64",
  "lang": "ko",
  "drift_period_days": 2,
  "patches": [
    {
      "file": "skills/plane-ops.md",
      "template_source": "agents/skills/plane-ops.md.tera",
      "change_type": "content_modified",
      "diff": "@@ -15,3 +15,5 @@\n-기존 Plane 상태: Open → In Progress → In Review → Done\n+기존 Plane 상태: Open → In Progress → In Review → QA → Done\n+## QA 단계 추가\n+- QA 담당자가 확인 후 Done 전이 허용",
      "likely_reason": "workflow_extension"
    }
  ],
  "user_comment": null
}
```

5. `~/.genasis/debug-history/<project-hash>/<timestamp>.patch.json`에 저장

### 4. Debug Submit (옵트인, 명시적, 미리보기 제공)

```bash
genasis debug submit [--all | --latest | --file <path>]
```

흐름:
1. 제출될 JSON 페이로드 전체를 보여줌
2. 확인: "이 내용을 genasis 개선에 제출하시겠습니까? (y/N)"
3. 사용자가 선택적으로 변경 이유 코멘트 추가 가능
4. 제출 경로:
   - **GitHub Issue** (`debug-history` 라벨 자동 생성) — genasis repo 쓰기
     권한 없어도 동작
   - **`debug-history/patches/`에 PR** — fork/쓰기 권한 있는 경우

### 5. 자기개선 기계장치 (genasis 리포 내)

#### 5a. `/debug-review` 스킬

`.claude/skills/debug-review.md`에 위치하는 Claude Code 스킬:
- `debug-history/patches/` 내 모든 패치 읽기
- 영향받은 템플릿/파일별 클러스터링
- 반복 패턴 식별 (예: "12명의 사용자가 plane-ops lifecycle에 QA 단계 추가")
- 템플릿 변경 제안 및 PR 초안 작성
- `debug-history/analysis/clusters.md` 업데이트

#### 5b. `GENASIS.md` 자기참조

사용자 프로젝트에 주입되는 `GENASIS.md` 프로토콜 계약에 포함:

```markdown
## Debug History

이 overlay는 자동으로 변경 사항을 추적합니다.
- 수정된 내용은 로컬에만 저장됩니다 (외부 전송 없음)
- `genasis debug collect` — 변경 요약 생성
- `genasis debug submit` — genasis 개선에 기여 (선택)
```

#### 5c. 분석 자동화

genasis에서 Claude Code 작업 시 에이전트는:
1. 템플릿 수정 전 `debug-history/patches/`에서 관련 신호 확인
2. 변경 제안 시 특정 패치 ID 참조 ("이 수정은 패치 a1b2, c3d4, e5f6에서
   관찰된 드리프트 패턴을 해결합니다")
3. 템플릿 수정 후 관련 패치를 `debug-history/index.jsonl`에서 `resolved`로
   태그

### 6. 보안 모델

| 계층 | 보호 |
|---|---|
| 수집 범위 | `.claude/genasis/`와 marker-fenced 섹션만 — `src/`, `lib/`, `app/`, 테스트 등 절대 불포함 |
| 시크릿 제거 | 토큰/키/패스워드의 정규식 기반 교정 후 내보내기 |
| 경로 익명화 | 절대 경로 대체; 프로젝트는 비가역 해시로만 식별 |
| 옵트인 제출 | 명시적 `debug submit` + 확인 없이는 아무것도 머신 밖으로 나가지 않음 |
| 바이너리/blob 없음 | 마크다운/셸/toml 파일의 텍스트 diff만 — 컴파일된 아티팩트 불포함 |
| 페이로드 미리보기 | 제출 전 전체 JSON을 사용자에게 표시 — 숨겨진 필드 없음 |
| 속도 제한 | 프로젝트당 하루 최대 1회 제출 (실수 스팸 방지) |

### 7. 추가 도구 없이 Claude Code가 debug-history 활용하는 방법

핵심 통찰: **debug-history 패치는 리포 내 구조화된 JSON 파일일 뿐이다.**
Claude Code는 이미 파일을 읽을 줄 안다. 단순한 스킬 프롬프트 이상의 특별한
MCP 서버, 외부 API, 커스텀 스킬 인프라가 필요 없다.

`/debug-review` 스킬 프롬프트:

```markdown
debug-history/patches/와 debug-history/index.jsonl의 모든 파일을 읽으세요.
각 패치에 대해:
1. 변경이 어떤 템플릿/overlay를 대상으로 하는지 식별
2. 변경이 버그 수정인지, 워크플로 확장인지, 프로젝트별 적응인지 판단
3. ≥2개 패치에 나타나는 버그 수정 및 워크플로 확장에 대해: 템플릿 변경 제안
4. 관련 .tera 템플릿에 대한 Edit으로 변경 초안 작성
5. debug-history/analysis/clusters.md를 결과로 업데이트
```

이것은 **새 도구가 전혀 필요 없다** — Claude Code가 이미 하는 파일 읽기와
편집만으로 충분하다.

## CLI 명령 추가

```
genasis debug
├── status              현재 프로젝트의 드리프트 요약 (변경 파일 수, 마지막 collect 시점)
├── collect             드리프트 → 익명화된 patch.json 생성
├── submit              옵트인 제출 (GitHub Issue or PR)
├── log                 .drift-log 내용 열람
└── reset               매니페스트를 현재 상태로 갱신 (드리프트 히스토리 초기화)
```

## 결과

### 긍정적
- Genasis가 구조화된, 기계 판독 가능한 필드 피드백을 제로 개발자 노력으로 확보
  (최초 `debug submit` 이후)
- 템플릿 개선이 추측이 아닌 실제 사용 패턴에 기반
- Claude Code가 패치 파일을 읽어 자율적으로 개선 제안 가능
- 제출하는 사용자의 수정사항이 미래 genasis 버전에 반영됨
- 보안이 기본 안전 (명시적 옵트인까지 로컬 전용)

### 부정적
- 매니페스트가 프로젝트당 `.claude/genasis/`에 ~2KB 추가
- 매 CLI 호출 시 SHA 비교로 ~1ms 오버헤드 추가 (무시 가능)
- genasis 리포의 `debug-history/`가 시간에 따라 성장 — 주기적 아카이빙 필요
  (제안: 6개월 이상 패치를 `debug-history/archive/YYYY-MM/`로 아카이브)

### 위험
- 사용자가 비즈니스 컨텍스트를 드러낼 수 있는 프로젝트별 용어가 포함된 패치를
  제출할 수 있음 → 미리보기 + 교정으로 완화
- 패치 볼륨이 `/debug-review` 스킬을 압도할 수 있음 → 클러스터링과 빈도
  임계값(≥2회 관찰된 패턴에 대해서만 변경 제안)으로 완화

## 구현 계획

| 단계 | 마일스톤 | 범위 |
|---|---|---|
| P1 | M15 | attach/init 시 매니페스트 생성 + 매 CLI 호출 시 드리프트 감지 |
| P2 | M15 | `genasis debug status/collect/log/reset` 명령 |
| P3 | M16 | `genasis debug submit` + GitHub Issue 생성 |
| P4 | M16 | `/debug-review` 스킬 + `debug-history/` 리포 구조 |
| P5 | M17 | 분석 자동화 + `clusters.md` 생성 |
