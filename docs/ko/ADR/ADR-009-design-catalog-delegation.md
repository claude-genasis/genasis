# ADR-009 — 디자인 카탈로그 위임 (vendor 안 함)

> English: [../../ADR/ADR-009-design-catalog-delegation.md](../../ADR/ADR-009-design-catalog-delegation.md)

- **Status**: Accepted (2026-05-04)
- **Phase**: D (post-M12)
- **Supersedes**: 없음
- **Related**: ADR-002 (단일 정적 바이너리), blueprint §7 (M7 design hot-swap)

## 배경

M7 에서 우리는 5-phase `design swap <url> --body <path>` orchestrator 를
가지고 있었다 — 미리 만들어진 `design-system.md` body 를 받아 persist /
diff / per-area improvement 이슈 emit 까지 자동화. Phase D 는 두 목표를
추가한다:

1. 외부 디자인 시스템(Apple, Linear, PostHog, Vercel 등)을 1차 swap
   대상으로 만든다 — `genasis design swap posthog`.
2. 에이전트가 외부 디자인 명세를 투명하게 참조하되, 사용자 오버라이드
   정책을 명확히 한다.

첫 번째 목표가 갈림길을 만들었다 — **awesome-design-md 의 71개 브랜드
DESIGN.md 를 genasis 리포에 vendor 할 것인가, 아니면 upstream `getdesign`
npm CLI 에 위임할 것인가?**

## 결정

우리는 **`getdesign` (npm 패키지, MIT, VoltAgent) 에 위임하고**,
**카탈로그 콘텐츠를 vendor 하지 않는다**. `genasis design swap <slug>` 는
설정 가능한 shell 명령 템플릿(기본:
`npx getdesign@latest add {slug} --force --out {out}`)을 호출하여 선택된
브랜드의 `DESIGN.md` 를 가져와 `docs/design-system/DESIGN.md` 에
떨어뜨린다. 이 템플릿이 갤러리 URL 의 교체 가능성을 보장한다 —
운영자는 코드 변경 없이 사내 fork 나 자체 호스팅 갤러리로 전환할 수 있다.

`docs/design-system.md` 는 두 모드로 작동한다:

- **`mode: pristine`** — 파일 본문이 진실. 외부 위임 없음.
- **`mode: external`** — 파일은 §A/§B/§C 3섹션 포인터:
  §A 는 `docs/design-system/DESIGN.md` 를 1차 참조로 가리키고,
  §B 는 엄격한 충돌 해결 정책 하에 사용자 오버라이드를 누적,
  §C 는 운영자 명령을 문서화한다.

`docs/.design-state.toml` 은 mode / slug / source / sha256 / applied_at /
previous_slug / override_count / gallery URLs 를 단일 소스로 기록한다.

## 결과

**긍정**

- MIT 라이선스 콘텐츠를 재호스팅하지 않으므로 라이선스 컴플라이언스는
  upstream 소관 — 리포가 가벼움.
- 71개 브랜드 (및 향후 추가)가 무료 — getdesign 이 이미 vendor 함.
- `add_command` 템플릿 한 줄이 갤러리 교체 지점. genasis 는 특정 브랜드
  리스트에 종속되지 않음.
- pristine vs external 의 깔끔한 2-모드 — 부분 상태 없음. `restore` 는
  external 디렉터리를 archive 로 옮긴 뒤 pristine 본문을 복원하는 단일
  비파괴 작업.

**부정 / 리스크**

- `genasis design swap <slug>` 는 런타임에 Node ≥18 + npx 필요. 이를
  `genasis doctor` 와 `install.sh` 패키지 안내에서 surface (pristine 일
  땐 경고, external 모드면서 npx 가 없으면 에러).
- `getdesign` upstream 이 사라지면 slug swap 이 깨짐. 완화: `add_command`
  를 사내 fork 로 repoint 가능. 기존 설치는 각 프로젝트가 DESIGN.md 를
  로컬에 캐싱하고 있으므로 정상 동작.
- 텔레메트리: getdesign 은 install 이벤트를 `https://getdesign.md/api/cli/downloads`
  로 POST. 우리는 npx 호출 전에 `GETDESIGN_DISABLE_TELEMETRY=1` 을
  자동 export 하여 **기본 OFF**. 사용자가 `[design].disable_telemetry = false`
  또는 `--telemetry` 로 opt-in.

## 대안 검토

1. **카탈로그 vendor** — 71개 DESIGN.md (와 라이선스) 를
   `crates/genasis-design-catalog/THIRD_PARTY/` 로 복사. 거부: 무한
   유지보수(카탈로그 갱신), 위임 대비 이점 없음 — getdesign 이 이미
   hash-pinned manifest 로 vendor 함.
2. **getdesign.md 직접 REST 호출** — npx 레이어를 건너뛰고
   `https://getdesign.md/<slug>/design-md.txt` 같은 엔드포인트 직접 호출.
   거부: JSON/MD endpoint 는 getdesign 공개 계약이 아님; npm CLI 가 공개
   계약. npx 경유는 upstream 이 지원하는 것에 정렬.
3. **포맷 어댑터 (Stitch DESIGN.md ↔ genasis design-system.md)** — 각
   브랜드를 기존 genasis §0~§n 포맷으로 파싱·재작성. 거부: 포인터 모델이
   더 단순하고 attribution 을 보존; 외부 본문은 read-only 이므로 재구성
   불필요.

## 사용자 오버라이드 정책 (이 ADR 와 결합)

포인터 본문의 §B 는 활성 DESIGN.md 위에 사람의 오버라이드를 기록하는
유일한 장소. `design-aware` 스킬이 강제:

1. §A 항목과 사용자 요청을 나란히 인용.
2. 요청이 §A 와 일치 → 그대로 진행, 기록 없음.
3. 요청이 §A 와 상충 → 명시적 `[y/N]` 확인.
4. `y` → `genasis design override add "<text>"` 호출. CLI 가 §B.2 에
   append 하고 `override_count` 증가.
5. `n` → §A 그대로 따름.

충돌 해결은 의도적으로 *human-in-the-loop* — 에이전트는 진실 소스를
조용히 무시할 수 없다.

## 향후 마일스톤 미결 항목

- swap 으로 인한 오버라이드 재검토 자동화? 현재는 새 slug 로 swap 시
  포인터 본문이 재생성되며 §B.2 가 초기화됨. 사용자가 새 디자인 하에서
  오버라이드를 재적용. 향후 `genasis design override migrate` 가 충돌
  플래깅과 함께 §B 를 옮길 수 있으나 — 실 사용 패턴을 본 후 결정.
- `--from <path>` 가 디렉터리(여러 디자인 문서 시드)를 받게 할까? 보류.
