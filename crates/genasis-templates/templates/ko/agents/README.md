# Base agent 템플릿 (M14)

각 역할마다 `<role>.md.tera` 한 개. `genasis bootstrap` (또는 `genasis
init --bootstrap`) 의 bootstrap 단계가 이 파일들을 **`.claude/agents/<role>.md`
가 부재할 때만** 렌더링합니다. emit 이후 파일 전체는 사용자 소유 — genasis
는 다시 덮어쓰지 않습니다.

이후 `genasis attach` 가 주입하는 marker fence (`agent-overlays/<role>.patch.md.tera`
형제 파일) 는 같은 파일의 **안쪽**, 이 base 컨텐츠의 **바깥쪽** 에 위치합니다.
ADR-001 (marker fence) + ADR-010 (base + patch 소유권 경계) 참조.

역할: `pm`, `planner`, `architect`, `frontend`, `backend`, `qa`,
`designer`, `security`, `devops`, `code-reviewer`.

템플릿 contract — 모든 base 파일이 반드시 갖춰야 할 것:

- YAML frontmatter 5 키: `name`, `description`, `tools`, `model`, `color`.
- `name` 은 파일명 stem 과 일치 (role-inference 가 즉시 `Known(_)` 으로
  classify).
- 5~10 줄 역할 헤더 (markdown) — 역할이 소유하는 영역 간단 설명. 짧게
  유지하세요. 프로토콜 본문은 patch overlay 가 채우므로 여기 들어가지
  않습니다.
