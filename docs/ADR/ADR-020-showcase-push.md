# ADR-020 — Showcase Push (운영자 hosting 만 유지, 사용자가 빌드 결과를 push)

> 한국어: [docs/ko/ADR/ADR-020-showcase-push.md](../ko/ADR/ADR-020-showcase-push.md)

Status: **Proposed (alpha.39 scaffold target)**
Date: 2026-05-16

## Context

지금까지 사용자가 만든 example app 이 `mmplane-trial.realstory.blog` 의
trial-app 안의 iframe 에 보이는 흐름은 **reverse-proxy** 모델:

1. devops agent 가 사용자 머신에서 `vite dev` 띄움 (예: `localhost:5173`)
2. agent 가 `announce_dev_server_url(http://localhost:5173)` 호출
3. trial-app 의 `/dev/<token>/*` 경로가 사용자의 vite 로 forward proxy
4. 사용자 브라우저 iframe 이 `/dev/<token>/` 열어 자기 머신의 dev server 콘텐츠 봄

**문제**:
- 사용자 머신 종료 / vite 죽음 → 운영자 trial-app 의 iframe 도 빈 화면
- 사용자가 데모를 친구에게 공유하려면 자기 머신을 계속 켜야 함
- 다른 디바이스 (스마트폰, 다른 PC) 에서 같은 URL 접속 시 reverse-proxy 가
  뚫린 사용자 머신 IP 에 의존 — 사실상 같은 LAN / 같은 머신에서만 동작

대안 검토:
- **GHCR 에 trial-app docker image publish** → 사용자가 자체 호스트 가능.
  단점: image publish CI 관리 부담 + 사용자가 docker 띄워야 함.
- **운영자 host 만 유지 + 사용자가 정적 자산을 push** → 자기 머신 꺼져도
  데모가 살아있음. 운영자 입장에서 storage + 보안 부담은 있지만 단일 운영점.

## Decision

**reverse-proxy 모델을 폐기하고 push-to-operator 모델로 일원화.**

### 새 흐름

```
사용자 머신                               운영자 (mmplane-trial.realstory.blog)
─────────────                              ──────────────────────────────
1. agent 가 vite 앱 빌드 (`npm run build`)
   → ./dist/ 정적 자산 생성

2. agent (또는 사용자) 가
   `genasis push` 호출
   ────── multipart POST ──────────────►   /api/trial/showcase-push
                                            ?team=<token>
                                            body: tarball(dist/)

3. trial-app 이 받은 자산을
                                            $TRIAL_STORAGE/<team_token>/ 에 저장
                                            
4. 브라우저가 https://mmplane-trial.realstory.blog/dev/<token>/ 접속
   ────── GET ────────────────────────►    `/dev/<token>/*` 가 위 storage 의
                                            정적 파일을 직접 serve
   ◄───── HTML/JS/CSS ──────────────────
```

사용자 머신 꺼져도 데모 live. 다른 디바이스에서 같은 URL 접속 가능.

### 새 subcommand: `genasis push`

```bash
cd <project-with-built-app>     # 예: marketing-squad/app/dist 이 있는 곳
genasis push                     # 자동 감지: dist/, build/, out/ 중 첫 매치
# 또는 명시
genasis push --dir ./app/dist
# 또는 archive 직접 지정
genasis push --tarball ./build.tar.gz
```

내부 동작:
1. `genasis.toml` 에서 trial team token 읽기
2. 자산 디렉터리를 tar+gzip 으로 패키지
3. `POST /api/trial/showcase-push?team=<token>` 으로 업로드
4. 업로드 끝나면 운영자가 unpack → `$TRIAL_STORAGE/<token>/` 저장
5. ShowcasePanel 이 `dev_server_url` 컬럼 보고 그게 비어있으면 새 column
   `static_deploy_at` 보고 static 자산이 있는지 판단

### Agent integration

devops agent overlay 에 한 단계 추가:

```
... (기존 vite scaffold + dev server 검증) ...
5. `npm run build` 호출 (dist/ 생성).
6. `genasis push --dir ./dist` 호출 — 운영자 trial-app 에 자산 전송.
7. `post_message(actor="devops", text="✅ 배포 완료 ...")`.
```

기존 `announce_dev_server_url` 은 제거 (reverse-proxy 모델 폐기로 불필요).

### 운영자 측 storage

- 위치: `$TRIAL_STORAGE/<team_token>/`
- 기본값: `<trial-app-cwd>/data/showcase/` (NFS, RAM 디스크 등 운영자 선택)
- per-team quota: **50 MB** (대부분의 Vite SPA 가 1-3 MB)
- TTL: **30일 미사용 시 자동 삭제** — `sim_teams.last_push_at` 기준 cron
- 보안: token-as-capability — `team_token` 가진 사용자만 그 팀 자산 read/write

### `/dev/<token>/*` route 변경

```ts
// app/dev/[token]/[[...path]]/route.ts (변경 후)
export async function GET(req, { params }) {
  const { token, path = [] } = params;
  const team = getTeam(token);
  if (!team) return new Response("not registered", { status: 404 });

  // D-068 reverse-proxy 분기 제거. 정적 파일만 serve.
  const requested = path.join("/") || "index.html";
  const filePath = path_join(STORAGE_ROOT, token, requested);
  if (!isInside(STORAGE_ROOT, filePath)) {
    return new Response("path escape", { status: 403 });
  }
  try {
    const buf = await fs.readFile(filePath);
    return new Response(buf, { headers: contentTypeFor(requested) });
  } catch {
    return new Response("no showcase deployed yet — run `genasis push`", {
      status: 404,
    });
  }
}
```

## Consequences

### Trade-offs

- **단일 운영점 유지** — image publish 자동화 불필요, 단일 deploy boundary.
- **사용자 ergonomics 큰 개선** — 머신 꺼도 데모 살아있음.
- **dev loop 다소 길어짐** — vite HMR 대신 `build + push` 매번. 수 초 추가.
- **운영자 storage 부담** — quota + TTL 로 mitigate.

### 기존 사용자 영향

- `announce_dev_server_url` 호출하는 옛 agent overlay 는 1 cycle deprecate
  (warning 출력, 동작 안 함). devops agent 가 그 step 호출하면 ShowcasePanel
  에 "이 흐름은 폐기됐습니다. `genasis push` 로 대체하세요" 표시.
- `dev_server_url` DB 컬럼 keep — `static_deploy_at` 컬럼 추가, 둘 다 nullable.

### 보안

- 정적 자산이 사용자 브라우저에서 실행되므로 XSS 가능 — 같은 token 의 자산은
  같은 사용자 소유라 self-XSS 만 가능, 다른 팀 자산 접근 불가 (token isolation).
- 운영자 측에서 자산 검증 / 스캐닝 안 함 — token-as-capability 만으로 충분
  (multi-tenant 격리는 token 단위).

### 마이그레이션

- alpha.39: `genasis push` scaffold + trial-app `/api/trial/showcase-push`
  endpoint + new `static_deploy_at` column.
- alpha.40: devops overlay 에 `npm run build && genasis push` 단계 추가.
  기존 `announce_dev_server_url` deprecation warning.
- alpha.41: `announce_dev_server_url` 완전 제거. reverse-proxy route 제거.

## Implementation status

- 본 ADR 작성: alpha.38 (현재).
- 다음: alpha.39 scaffold (CLI + endpoint + storage).
