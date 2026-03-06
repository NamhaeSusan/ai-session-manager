# ai-session-manager

## 컨셉

Claude Code와 Codex의 세션을 터미널에서 탐색, 미리보기, 재개, 삭제할 수 있는 Rust TUI 애플리케이션.

---

## 아키텍처

```
ai-session-manager/
├── Cargo.toml              # [workspace] members = ["asm-core", "asm"]
├── asm-core/               # 공유 라이브러리 (tre-file-manager에서도 사용)
│   ├── Cargo.toml           # deps: serde, serde_json
│   └── src/lib.rs           # 세션 스캔/파싱/삭제/대화 읽기 (ScanMode::Fast/Full)
├── asm/                    # TUI 바이너리
│   ├── Cargo.toml           # deps: asm-core, ratatui, crossterm, serde, serde_json, toml
│   └── src/
│       ├── main.rs          # 엔트리포인트, 터미널 초기화/정리, 이벤트 루프
│       ├── app.rs           # App 상태 관리, 키 입력 핸들링
│       ├── config.rs        # 설정 파일 로드 (~/.config/asm/config.toml)
│       ├── tree.rs          # 3단 트리 구조 (Tool -> Project -> Session), 필터링
│       └── ui.rs            # ratatui 기반 UI 렌더링 (트리, 프리뷰, 상태바, 팝업)
├── README.md
└── CLAUDE.md
```

---

### 작업 기록 (CRITICAL)

모든 작업이 끝나면 `claude_history/` 폴더에 기록을 남긴다.

- **파일명**: `yyyy-mm-dd-{work}.md` (예: `2026-03-05-add-search.md`)
- **내용**: 작업 요약, 변경된 파일, 검증 결과를 간단하게 기록
- 같은 날 여러 작업 시 work 부분으로 구분

---

### 기능 구현 후 필수 체크리스트 (CRITICAL — 절대 빠뜨리지 말 것)

기능을 추가하거나 변경한 뒤에는 **반드시** 아래 문서들을 모두 업데이트해야 한다.
CLAUDE.md와 README.md 는 필수로 업데이트 하도록 한다.

| 변경 유형 | 업데이트할 문서 |
|-----------|----------------|
| 새 모듈/파일 추가 | `CLAUDE.md` 아키텍처 트리 |
| 새 의존성 추가 | `CLAUDE.md` 기술 스택, `README.md` Dependencies |
| 새 Feature 추가 | `CLAUDE.md` 기능 상세, `README.md` Features |
| 키바인딩 변경 | `README.md` Keybindings |

---

### 작업 방식 (CRITICAL)

**간단한 작업**(단일 파일 수정, 오타 수정, 한 줄짜리 버그 픽스)은 직접 처리해도 된다.

**그 외 모든 작업**은 반드시 **TeamCreate로 에이전트 팀을 구성**해서 병렬로 진행한다:
- 기능 구현 → 구현 에이전트
- 테스트/검증 → 검증 에이전트
- 문서 업데이트 → 문서 에이전트
- 코드 리뷰 → `code-reviewer` 에이전트

---

### 핵심 설계 원칙

1. **공유 라이브러리** — 세션 로직을 `asm-core`로 분리하여 tre-file-manager와 공유
2. **외부 런타임 의존성 없음** — 순수 Rust, 시스템 라이브러리만 사용. 날짜 계산도 직접 구현
3. **exec로 세션 재개** — TUI 종료 후 `exec`로 프로세스를 대체하여 깔끔하게 세션 전환
4. **안전한 삭제** — 경로 정규화(canonicalize)로 디렉토리 traversal 방지, 빈 디렉토리만 정리
5. **ScanMode** — `Fast` (200줄 스캔, line count) 웹 서버용 / `Full` (전체 파일, 실제 메시지 카운트) TUI용
6. **asm-core 공개 API 안정성** — `asm-core`는 tre-file-manager가 `branch = "main"`으로 의존한다. 가능한 기존 응답 타입(struct 필드, enum variant)과 함수 시그니처를 유지하고, 새 함수/타입을 추가하는 방식으로 확장할 것

---

## 기술 스택

| 영역 | 라이브러리 | 용도 |
|------|-----------|------|
| 세션 스캐닝 | `asm-core` (워크스페이스 내) | 세션 감지·파싱·삭제·대화 읽기 공유 라이브러리 |
| TUI 렌더링 | `ratatui` 0.29 | 위젯 기반 터미널 UI |
| 터미널 I/O | `crossterm` 0.28 | 키 입력, raw mode, alternate screen |
| 직렬화 | `serde` 1 + `serde_json` 1 | JSONL 세션 파일 파싱 |
| 설정 파일 | `toml` 0.8 | TOML 설정 파일 파싱 |

---

## 핵심 기능 상세

### 세션 스캔
- `~/.claude/projects/` 에서 Claude Code 세션 (.jsonl) 스캔
- `~/.codex/sessions/` 에서 Codex 세션 (.jsonl) 재귀 스캔
- sidechain/child 세션 필터링 (parentUuid, isSidechain)

### 트리 뷰
- Tool -> Project -> Session 3단계 계층 구조
- 접기/펼치기, 커서 이동
- 텍스트 검색 필터링 (프롬프트, 프로젝트명, 세션 ID)

### 프리뷰
- 세션 메타데이터 (프로젝트, 경로, 브랜치, 생성일, 메시지 수)
- 대화 내역 미리보기 (최대 50줄)
- Ctrl+d/u로 스크롤

### 세션 관리
- Enter로 세션 재개 (`claude --resume` / `codex --resume`)
- d -> y로 세션 삭제 (확인 팝업)
- r로 세션 목록 새로고침

### 설정 파일
- `~/.config/asm/config.toml` 또는 `~/.asm.toml` 에서 설정 로드
- `default_sort`: 기본 정렬 모드 ("date" / "project" / "messages")
- `default_expanded`: 트리 기본 펼침 여부 (bool)
- `claude_projects_dir`: Claude Code 프로젝트 디렉토리 경로
- `codex_sessions_dir`: Codex 세션 디렉토리 경로
- `skip_permissions`: resume 시 permission bypass 플래그 자동 추가 (기본: true). Claude Code: `--dangerously-skip-permissions`, Codex: `--dangerously-bypass-approvals-and-sandbox`

### 정렬 옵션
- s 키로 정렬 모드 순환 (date → project → messages)
- 트리 타이틀에 현재 정렬 모드 표시

### 세션 통계
- i 키로 통계 팝업 표시
- 총 세션 수, 도구별 세션 수, 상위 10개 프로젝트
- Esc/i/q로 팝업 닫기

---

## 로드맵

### Phase 1 — MVP
- [x] Claude Code 세션 스캔 및 표시
- [x] Codex 세션 스캔 및 표시
- [x] 3단 트리 뷰
- [x] 세션 프리뷰 패널
- [x] 세션 재개 (exec)
- [x] 세션 삭제
- [x] 검색/필터링

### Phase 2 — 개선
- [x] 세션 정렬 옵션 (날짜, 프로젝트, 메시지 수)
- [x] 세션 통계 (총 세션 수, 프로젝트별 통계)
- [x] 설정 파일 지원
