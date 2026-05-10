> English: [`../MIGRATE-PG-CONSOLIDATION.md`](../MIGRATE-PG-CONSOLIDATION.md)

# PostgreSQL 통합 마이그레이션 — Plane + Mattermost → 단일 PG

ADR-015 결정에 따라 `servers/docker-compose.yml`이 Plane용 `plane-db`와
Mattermost용 `mm-postgres` 두 인스턴스를 단일 `pg-shared`(postgres:15.7-alpine)로
통합했습니다. 본 문서는 **이미 운영 중인 배포**를 새 구조로 옮기는 절차입니다.
신규 배포는 그냥 `docker compose up -d`만 하면 되므로 본 문서는 무관합니다.

## 변경 사항 요약

| 항목 | 이전 | 이후 |
|---|---|---|
| Plane Postgres | `plane-db` 서비스, `postgres:15.7-alpine` | `pg-shared` 단일, `postgres:15.7-alpine` |
| MM Postgres | `mm-postgres` 서비스, `postgres:18-alpine` | `pg-shared` 단일, `postgres:15.7-alpine` |
| Plane DB volume | `plane-pgdata` | `pg-shared-data` |
| MM DB volume | `mm-pgdata` | `pg-shared-data` (공용) |
| Plane DATABASE_URL | `…@plane-db/plane` | `…@pg-shared/plane` |
| MM 데이터소스 | `…@mm-postgres:5432/mattermost…` | `…@pg-shared:5432/mattermost…` |
| max_connections | Plane 1000 / MM 기본 | 통합 1500 |
| 트라이얼 앱 데이터 디렉토리 | `./data` 바인드 마운트 | 명명 볼륨 `trial-app-data` |

## 사전 준비

- 새 compose가 들어 있는 git revision 으로 체크아웃 (단, **아직 `up`은 금지**)
- 기존 운영 스택 정상 작동 확인 (`docker compose ps` 모두 healthy)
- Postgres dump 시 일시 잠금 발생 가능 — 사용자 통보 후 점검 시간대 진행 권장
- 디스크 여유 공간: 두 DB 합산 크기의 ≥1.5배

## 마이그레이션 절차

### 1. 양쪽 DB 덤프

```bash
cd servers/

# 두 DB 동시 dump (compose 내부에서 실행해 password 노출 방지)
docker compose exec -T plane-db \
  pg_dump -U "$PLANE_POSTGRES_USER" -d plane -Fc -Z9 \
  > /tmp/plane-$(date +%Y%m%d).dump

docker compose exec -T mm-postgres \
  pg_dump -U "$MM_POSTGRES_USER" -d mattermost -Fc -Z9 \
  > /tmp/mattermost-$(date +%Y%m%d).dump

ls -lh /tmp/{plane,mattermost}-*.dump
```

> ⚠️ Plane 1.x 는 `worker` / `beat` 가 mid-write 일 수 있습니다. 안전한 dump를
> 위해 직전에 worker/beat만 잠깐 멈추면 깔끔합니다:
> ```bash
> docker compose stop plane-worker plane-beat
> # ↑ dump 후
> docker compose start plane-worker plane-beat
> ```

### 2. 기존 스택 정지 (볼륨은 보존)

```bash
docker compose down
# 주의: `down -v` 절대 금지 — 볼륨이 날아가면 dump가 유일한 소스
```

### 3. 신규 compose 적용

git pull 등으로 새 `docker-compose.yml` + `init/init-databases.sh`를 받아옵니다.
`servers/.env`에 다음이 있는지 확인 (없으면 `setup-user-env.sh` 재실행 또는
수동 추가):

```env
COMPOSE_PROJECT_NAME=genasis-...
PLANE_POSTGRES_USER=plane
PLANE_POSTGRES_PASSWORD=<기존과 동일>
MM_POSTGRES_USER=mmuser
MM_POSTGRES_PASSWORD=<기존과 동일>
```

> 비밀번호는 dump 안의 user 권한 정보와 일치해야 합니다 — 새로 생성하면
> restore 시 권한 매핑에서 깨질 수 있어 **기존 비밀번호 그대로 유지**가 안전.

### 4. pg-shared만 먼저 기동

```bash
docker compose up -d pg-shared

# init script 가 mattermost role/DB 생성하기까지 대기 (~5초)
docker compose logs -f pg-shared | grep -m1 "init-databases.sh"

# 두 DB가 모두 보이는지 확인
docker compose exec pg-shared \
  psql -U plane -d postgres -c '\l'
# → plane, mattermost 두 DB 가 보여야 함
```

### 5. dump 복원

```bash
# Plane
docker compose exec -T pg-shared \
  pg_restore -U plane -d plane --clean --if-exists \
  < /tmp/plane-$(date +%Y%m%d).dump

# Mattermost
docker compose exec -T pg-shared \
  pg_restore -U mmuser -d mattermost --clean --if-exists \
  < /tmp/mattermost-$(date +%Y%m%d).dump
```

> Plane 의 일부 owner GRANT 가 `plane` 외 role을 참조해 warning 이 떨어질 수
> 있으나 데이터는 정상 복구됩니다. critical error 만 주시.

### 6. 나머지 서비스 기동 + 헬스 체크

```bash
docker compose up -d

# Plane
curl -fsSL "http://localhost:${PLANE_PORT}/api/v1/health/" | jq .
# Mattermost
curl -fsSL "http://localhost:${MM_PORT}/api/v4/system/ping" | jq .

# 양쪽 마이그레이션이 멱등하게 한 번 더 돌고 끝남
docker compose logs plane-migrator | tail -20
docker compose logs mattermost     | tail -20
```

### 7. 트라이얼 앱 데이터 이전 (해당 시)

기존 `trial-app/docker-compose.yml`이 `./data` 바인드 마운트였다면 이제
명명 볼륨 `trial-app-data`로 변경됐습니다.

```bash
cd ../trial-app
# 새 컨테이너 한 번 띄워서 빈 볼륨 만들기
docker compose up -d
docker compose stop

# 호스트의 ./data 안 sqlite를 명명 볼륨에 복사
docker run --rm \
  -v "$(pwd)/data:/from:ro" \
  -v "${COMPOSE_PROJECT_NAME}_trial-app-data:/to" \
  alpine sh -c "cp -av /from/. /to/"

docker compose up -d
```

### 8. 정상 작동 확인 후 정리

- 24시간 이상 운영하며 이슈 생성/메시지 작성/마이그레이션 정상 동작 확인
- 정상 작동 확인되면 구 `plane-pgdata`, `mm-pgdata` 볼륨 삭제:
  ```bash
  docker volume rm "${COMPOSE_PROJECT_NAME}_plane-pgdata"
  docker volume rm "${COMPOSE_PROJECT_NAME}_mm-pgdata"
  ```
- `/tmp/*.dump` 파일도 안전한 곳으로 옮기거나 삭제

## 롤백 절차

문제가 생겼다면 새 스택을 정지하고 기존 compose 로 되돌립니다.

```bash
docker compose down
git checkout <이전-commit> -- servers/docker-compose.yml
docker compose up -d
```

`plane-pgdata`, `mm-pgdata` 볼륨이 8단계 정리 전이라면 자동으로 다시
인식되어 데이터 손실 없이 롤백됩니다.

## 마이그레이션 후 변경되는 운영 절차

| 작업 | 이전 | 이후 |
|---|---|---|
| Postgres 백업 | `pg_dump` 두 번 (각 DB) | `pg_dumpall` 한 번 또는 DB별 두 번 |
| PG 업그레이드 | Plane/MM 각각 독립 일정 | 양쪽 동시 일정 — 통합 PG 한 번에 업그레이드 |
| 디스크 사용량 모니터링 | 두 볼륨 따로 | 단일 볼륨 `pg-shared-data` |
| 장애 격리 | 한쪽 PG 죽어도 반대편 정상 | 둘 다 영향. **HA 가 필요한 운영은 통합 비추천** |

## 자주 묻는 질문

**Q. PG 18에서 PG 15로 다운그레이드인데 Mattermost 데이터가 안 깨지나요?**
A. Mattermost 10.11 은 Postgres 13–17 모두 공식 지원합니다. PG 18 이 비공식
지원이었던 것이고, dump 형식은 모든 메이저 버전 호환이라 restore 가능.

**Q. max_connections=1500 이 너무 큰가요?**
A. Plane gunicorn workers + worker + beat + live + redis pool 합산이 800–1000
정도 / MM 기본 풀 200–400. 여유 두고 1500 — 32GB 호스트에서 RAM 영향 무시 가능
(`shared_buffers` 가 따로 있으니 connection 한도 자체는 메모리 부담 작음).

**Q. 한 PG 가 죽으면 양쪽 다운인데 위험하지 않나요?**
A. 맞음. **HA 가 SLA 인 운영**은 통합 비추천. trial / 데모 / 개발 환경에서
RAM 절약과 관리 단순화 가치가 큰 케이스에 적합. 운영 환경은 ADR-015 §
Trade-offs 다시 검토.

## 관련 문서

- ADR-015 — Postgres 통합 결정 (예정)
- [`servers/README.md`](../../servers/README.md) — 신규 구조 전체 가이드
- [`docs/ko/ADR/ADR-013-trial-bridge-config-wiring.md`](ADR/ADR-013-trial-bridge-config-wiring.md) — trial 라우팅 SSOT
