# ADR-020 — Showcase Push (운영자 호스팅만 유지, 사용자가 빌드 결과 push)

> English: [docs/ADR/ADR-020-showcase-push.md](../../ADR/ADR-020-showcase-push.md)

상태: **Proposed (alpha.39 scaffold 타겟)**
일자: 2026-05-16

## 배경

지금까지 사용자 example app 이 `mmplane-trial.realstory.blog` 의
trial-app iframe 에 보이는 흐름은 **reverse-proxy** 모델:

1. devops agent 가 사용자 머신에서 `vite dev` (`localhost:5173`) 띄움
2. `announce_dev_server_url(http://localhost:5173)` 호출
3. trial-app 의 `/dev/<token>/*` 경로 → 사용자 vite 로 forward
4. iframe 이 `/dev/<token>/` 열면 사용자 머신의 dev server 콘텐츠 봄

**문제**:
- 사용자 머신 끄거나 vite 죽으면 iframe 빈 화면
- 데모 공유 시 사용자 머신 항상 켜둬야 함
- 다른 디바이스에서 같은 URL 접속 시 reverse-proxy 가 뚫린 사용자 IP 의존

대안:
- GHCR 에 trial-app docker image publish → image 관리 부담
- **운영자 host 만 유지 + 사용자가 정적 자산 push** → 머신 꺼도 데모 살아있음.
  단일 운영점.

## 결정

**reverse-proxy 모델 폐기, push-to-operator 모델로 통일.**

### 새 흐름

```
사용자 머신                              운영자 (mmplane-trial.realstory.blog)
─────────────                             ──────────────────────────────
1. agent 가 vite 앱 빌드 (`npm run build`)
   → ./dist/ 정적 자산 생성

2. agent (또는 사용자) `genasis push`
   ─── multipart POST ──────────────►    /api/trial/showcase-push
                                          ?team=<token>
                                          body: tarball(dist/)

3. trial-app 이 받은 자산을
                                          $TRIAL_STORAGE/<team_token>/ 에 저장

4. 브라우저가 https://.../dev/<token>/
   ─── GET ─────────────────────────►    `/dev/<token>/*` 가 위 storage 의
                                          정적 파일 직접 serve
   ◄── HTML/JS/CSS ──────────────────
```

사용자 머신 꺼도 데모 live. 다른 디바이스 접속 가능.

### 새 서브명령: `genasis push`

```bash
cd <project-with-built-app>
genasis push                     # 자동 감지: dist/, build/, out/
genasis push --dir ./app/dist    # 명시
genasis push --tarball ./build.tar.gz   # archive 직접
```

내부 동작:
1. `genasis.toml` 에서 trial team token
2. 자산 디렉터리 tar+gzip
3. `POST /api/trial/showcase-push?team=<token>`
4. 운영자가 unpack → `$TRIAL_STORAGE/<token>/` 저장
5. ShowcasePanel 이 `dev_server_url` 비어있으면 `static_deploy_at` column
   으로 static 자산 확인

### Agent 통합

devops agent overlay 에 단계 추가:

```
... (기존 vite scaffold) ...
5. `npm run build` (dist/ 생성)
6. `genasis push --dir ./dist`
7. `post_message(actor="devops", text="✅ 배포 완료 ...")`
```

기존 `announce_dev_server_url` 제거 (reverse-proxy 폐기).

### 운영자 측 storage

- 위치: `$TRIAL_STORAGE/<team_token>/`
- 기본: `<trial-app-cwd>/data/showcase/`
- per-team quota: **50 MB** (Vite SPA 일반적으로 1~3 MB)
- TTL: **30일 미사용 시 자동 삭제** — `sim_teams.last_push_at` cron
- 보안: token-as-capability — `team_token` 가진 사용자만 자기 팀 자산 R/W

### `/dev/<token>/*` route 변경

```ts
export async function GET(req, { params }) {
  const { token, path = [] } = params;
  const team = getTeam(token);
  if (!team) return new Response("not registered", { status: 404 });

  // reverse-proxy 분기 제거. 정적 파일만 serve.
  const requested = path.join("/") || "index.html";
  const filePath = path_join(STORAGE_ROOT, token, requested);
  if (!isInside(STORAGE_ROOT, filePath)) {
    return new Response("path escape", { status: 403 });
  }
  try {
    const buf = await fs.readFile(filePath);
    return new Response(buf, { headers: contentTypeFor(requested) });
  } catch {
    return new Response("no showcase deployed yet — run `genasis push`",
      { status: 404 });
  }
}
```

## 결과

### Trade-offs

- **단일 운영점 유지** — image publish 자동화 불필요
- **사용자 ergonomics 큰 개선** — 머신 꺼도 데모 살아있음
- **dev loop 다소 길어짐** — vite HMR 대신 `build + push`
- **운영자 storage 부담** — quota + TTL 로 mitigate

### 기존 사용자 영향

- `announce_dev_server_url` 호출 옛 overlay 는 1 cycle deprecate
- `dev_server_url` DB 컬럼 keep, `static_deploy_at` 컬럼 추가
- showcase 가 dev_server_url 있으면 옛 reverse-proxy (deprecated 경로) 표시

### 보안

- 정적 자산 XSS 가능 — 같은 token 의 self-XSS 만, 다른 팀 자산 접근 불가
- 운영자 측 검증 / 스캐닝 안 함 — token isolation 만으로 충분

### 마이그레이션

- **alpha.39**: `genasis push` scaffold + `/api/trial/showcase-push` endpoint +
  `static_deploy_at` column
- **alpha.40**: devops overlay 에 `npm run build && genasis push` 단계.
  `announce_dev_server_url` deprecation warning
- **alpha.41**: `announce_dev_server_url` 완전 제거. reverse-proxy 제거

## 구현 상태

- ADR: alpha.38 (현재)
- 다음: alpha.39 scaffold (CLI + endpoint + storage)
