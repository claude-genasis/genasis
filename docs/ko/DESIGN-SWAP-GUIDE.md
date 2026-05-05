> English: [../DESIGN-SWAP-GUIDE.md](../DESIGN-SWAP-GUIDE.md)

# 디자인 교체 가이드

프로젝트의 디자인 시스템은 `docs/design-system.md`에 있습니다. 새로운 디자인
언어를 도입하거나, 컴포넌트 라이브러리를 변경하거나, 처음부터 새로 시작하고
싶을 때 genasis가 안전하게 전환을 처리합니다.

## design-system.md 동작 방식

- **단일 진실 원천** — 토큰(컬러, 타이포, 간격, 모션)과 컴포넌트 사양을 담고 있음.
- **프론트엔드 에이전트가 참조** — UI 코드 작성 전 반드시 읽음.
- **디자이너 에이전트가 관리** — 변경을 오케스트레이션.
- **두 가지 모드**: pristine (본문이 진실) / external (외부 DESIGN.md를 가리키는 포인터).

## 갤러리에서 교체

```bash
# 사용 가능한 디자인 시스템 확인
genasis design status

# 슬러그로 새 디자인 시스템 적용
genasis design swap tailwind-minimal
#   → 현재 design-system.md를 pristine.bak으로 백업
#   → npx getdesign으로 새 사양 가져오기
#   → 영향받는 UI 영역에 Plane 이슈 생성
#   → Mattermost에 🚨 DESIGN CHANGE 알림
```

교체 후, 프론트엔드 에이전트는 업데이트된 사양을 읽을 때까지 작업을 중단합니다.

## 로컬 파일로 교체

자체 디자인 사양이 있는 경우 (예: Figma 내보내기):

```bash
genasis design swap --from ./my-design-spec.md
#   → 동일 흐름이지만 갤러리 대신 로컬 파일에서 읽음
```

## 이전 디자인으로 복원

```bash
genasis design restore
#   → 외부 DESIGN.md를 아카이브로 이동
#   → pristine.bak을 design-system.md로 복원
#   → §B 사용자 오버라이드 초기화
```

## 사용자 오버라이드

개발자가 디자인 시스템에서 벗어나야 할 때:

```bash
genasis design override add "모바일 카드 그리드에서 12px 대신 8px 간격 사용"
#   → design-system.md의 §B에 추가
#   → 디자이너 에이전트가 인지하고 추적
```

```bash
genasis design override list    # 누적된 모든 오버라이드 표시
genasis design override remove <id>
```

## EPIC 모드 (4개 이상 영역 영향 시 자동)

디자인 교체가 4개 이상 영역(color-tokens, typography, spacing, layout,
components, motion)에 영향을 미치면 genasis가 자동으로:

1. Plane에 **EPIC** 이슈 생성
2. 영향 영역별 하위 이슈 생성
3. 각 하위 이슈에 프론트엔드 에이전트 할당
4. Mattermost에 요약 공지

`--per-area`로 4개 미만이어도 영역별 이슈를 강제하거나,
`--full-rewrite`로 항상 EPIC 구조를 사용할 수 있습니다.

## 무결성 검증

```bash
genasis design verify
#   → DESIGN.md의 SHA-256과 .design-state.toml의 해시 비교
#   → 무단 수정 감지
```

## 설정

`genasis.toml`:

```toml
[design]
gallery_index_url = "https://getdesign.md/"
add_command = "npx getdesign@latest add {slug} --force --out {out}"
disable_telemetry = true    # getdesign에 분석 데이터 미전송
external_dir = "docs/design-system"
```

## 참고

- [ADR-009: 디자인 카탈로그 위임](ADR/ADR-009-design-catalog-delegation.md)
- [`docs/example-design-system.md`](../example-design-system.md) — 디자인 사양 예시
