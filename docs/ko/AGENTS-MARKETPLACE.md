> English: [../AGENTS-MARKETPLACE.md](../AGENTS-MARKETPLACE.md)

# 에이전트 마켓플레이스 가이드

Genasis는 최고의 커뮤니티 저장소(ECC, wshobson/agents, VoltAgent, dl-ezo)에서
선별한 20+ 에이전트를 제공합니다. 카테고리별 브라우징, 개별 설치, 프리셋 설치가 가능합니다.

## 에이전트 탐색 (CLI)

```bash
# interactive TUI — 카테고리 선택 → 에이전트 복수 선택 → 설치
genasis agents browse

# 카테고리 필터
genasis agents list --category mobile
genasis agents list --search "security"

# 현재 프로젝트에 설치된 에이전트 확인
genasis agents installed
```

## 에이전트 탐색 (Claude Code)

```
/install-agent frontend
```

Claude가 인덱스를 검색하고, 설명과 함께 매칭되는 에이전트를 보여주며,
선택하면 바로 설치됩니다.

## 이름으로 설치

```bash
genasis agents install frontend-developer
genasis agents install code-reviewer
genasis agents install ios-expert
```

## 프리셋 설치

```bash
# 웹 앱 팀 (9역할)
genasis agents install --preset web-app

# DevOps 포함 풀스택 (11역할)
genasis agents install --preset full-stack

# 모바일 앱 팀 (9역할)
genasis agents install --preset mobile
```

## 카테고리

| 카테고리 | 에이전트 |
|---|---|
| 핵심 개발 | frontend-developer, backend-developer, architect, designer, doc-updater |
| 리뷰 & 품질 | code-reviewer, qa-tester, security-reviewer, refactor-cleaner, silent-failure-hunter |
| 기획 & 관리 | pm, planner |
| 인프라 & DevOps | sre-engineer, devops-engineer |
| 모바일 | ios-expert, android-expert, react-native-expert, flutter-expert |
| 언어 전문가 | typescript-reviewer, python-reviewer, rust-reviewer, go-reviewer, nextjs-developer |

## 설치 후

`genasis attach`를 실행하면 각 설치된 에이전트에 Plane/Mattermost 오버레이
프로토콜이 자동 주입됩니다. 이슈 트래커와 팀 채팅에 자동으로 연결됩니다.

## 에이전트 제거

```bash
genasis agents remove frontend-developer
```

## 카탈로그 업데이트

```bash
genasis agents fetch              # 최신 카탈로그 다운로드
genasis agents fetch --version 1.1.0  # 특정 버전
```
