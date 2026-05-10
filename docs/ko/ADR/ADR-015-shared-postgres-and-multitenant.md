> English: [`../../ADR/ADR-015-shared-postgres-and-multitenant.md`](../../ADR/ADR-015-shared-postgres-and-multitenant.md)

# ADR-015: 공유 PostgreSQL + 동일 호스트 다중 운영자 격리

## Status

Accepted (2026-05-10).

## Context

`servers/docker-compose.yml`은 Plane용 `plane-db`와 Mattermost용 `mm-postgres`
두 개의 PostgreSQL 인스턴스를 띄우고 있었다. 두 앱이 실제로는 같은 RDBMS
프로토콜을 쓰는데도 분리된 이유는 단순 관성. 이로 인해:

- RAM ~600MB 추가 사용 (PG ×2)
- 운영자가 백업 두 번, 업그레이드 두 번 실행
- `mm-postgres`가 `postgres:18-alpine` (Mattermost 10.11 비공식 지원선)을 사용
  중이라 잠재적 호환성 이슈

또한 본 스택을 **여러 운영자가 한 호스트에서 각자 계정으로 띄우려는** 시나리오
(사내 실습, 데모 환경)에서:

- `COMPOSE_PROJECT_NAME` 미지정 시 컨테이너/볼륨 충돌
- 호스트 포트 (`PLANE_PORT`, `MM_PORT`, trial-app `3000`) 충돌
- TLS 도메인/시크릿 공유로 인한 운영자 간 간섭

## Decision

### 1. PostgreSQL 한 인스턴스로 통합 — `pg-shared`

- 이미지: `postgres:15.7-alpine` — Plane 공식 권장 + Mattermost 10.11 공식 지원
  교집합
- 단일 볼륨 `pg-shared-data`
- 부트 시 `init/init-databases.sh`가 두 번째 role+DB(mattermost) 생성
  (Plane DB는 `POSTGRES_DB` 환경변수로 자동 생성)
- `max_connections=1500` (Plane gunicorn pool + MM 합산 + 여유)
- `mm-postgres`에 있던 hardening (`security_opt: no-new-privileges`,
  `read_only: true`, `tmpfs`) 그대로 유지

### 2. 모든 외부 노출 자원을 사용자별 격리

- `COMPOSE_PROJECT_NAME=genasis-${USER}` — 헬퍼 스크립트가 강제. 이 한 변수가
  컨테이너 이름·네트워크·**볼륨**까지 자동 namespace 처리.
- 호스트 포트는 UID 기반 offset 으로 자동 계산:
  - `PLANE_PORT = 38400 + (uid % 50)`
  - `MM_PORT    = 38500 + (uid % 50)`
  - `TRIAL_APP_PORT = 3100 + (uid % 50)`
- `ss`/`lsof` 로 점유 여부 사전 확인, 충돌 시 다음 빈 슬롯 자동 탐색
- 모든 시크릿(`*_PASSWORD`, `*_SECRET_KEY`, `*_SHARED_SECRET`)을
  `openssl rand -hex 30` 로 자동 생성

### 3. trial-app 도 동일 격리에 포함

- `trial-app/docker-compose.yml` 의 `./data` 바인드 마운트 → 명명 볼륨
  `trial-app-data` 변경 (`COMPOSE_PROJECT_NAME` namespacing 적용)
- `trial-app/.env` 에도 `COMPOSE_PROJECT_NAME` + `TRIAL_APP_PORT` +
  `TRIAL_SHARED_SECRET` 도입
- `setup-user-env.sh` 가 `servers/.env` 와 `trial-app/.env` 를 한 번에 작성하면서
  `TRIAL_SHARED_SECRET` 을 양쪽 동일하게 맞춤 → Rust trial provider가 곧바로
  본인 trial-app 으로 라우팅

### 4. Caddy 분리 패턴 문서화

`/etc/caddy/Caddyfile` 한 곳에서 `import /etc/caddy/sites/genasis-*.caddy`
한 줄로 운영자별 sub-도메인 라우팅을 흡수. 운영자는 본인 파일만 추가/수정하면
되고 root는 `systemctl reload caddy` 만.

## Consequences

**Easier**:
- 32GB 호스트 기준 동시 4명 데모 가능 (이전엔 2–3명에서 RAM 한계)
- 백업/업그레이드 절차 절반으로 축소
- 신규 실습자 합류 = 본인 계정에서 `setup-user-env.sh` + `docker compose up`
  단 두 줄

**Harder**:
- 통합 PG가 단일 장애 지점 — 한 번 죽으면 양쪽 다 영향
- HA-SLA 운영 환경에는 비추천 (분리 layout 유지하라는 가이드를 ADR + 마이그레이션
  가이드에 명시)
- 기존 운영 중인 배포는 `pg_dump`/`pg_restore` 수동 절차 필요 (마이그레이션
  가이드 작성)

**Foreclosed**:
- Plane PG 와 MM PG 의 **독립 메이저 버전 업그레이드** — 통합 후엔 둘이 같은
  메이저 버전을 공유. (한쪽이 PG 16 요구 / 다른 쪽이 PG 15 강제 같은 시나리오는
  layout 분리로 회귀 필요)

## Verification

- `docker compose config` 가 양쪽 compose 모두 valid (compose v2 spec)
- 신규 호스트에서 `setup-user-env.sh` → `docker compose up -d` → Plane API
  health + MM ping 200 응답
- 두 명 이상 동시 기동 시 컨테이너/포트/볼륨 충돌 없음 (`docker ps`,
  `ss -tln` 으로 검증)
- trial-app `/api/events/stream` 이 본인 PG-shared 와 무관하게 SSE 정상 흐름

## References

- 구현: `servers/docker-compose.yml`, `servers/init/init-databases.sh`,
  `servers/scripts/setup-user-env.sh`, `trial-app/docker-compose.yml`
- 마이그레이션 가이드: [`../MIGRATE-PG-CONSOLIDATION.md`](../MIGRATE-PG-CONSOLIDATION.md)
- 관련 ADR: ADR-013 (trial bridge config SSOT) — `[trial]` 섹션과 trial-app 사이
  라우팅이 본 ADR 의 `TRIAL_SHARED_SECRET` 매개로 확장됨
