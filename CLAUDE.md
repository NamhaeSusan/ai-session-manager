# ai-session-manager

## 컨셉

Claude Code와 Codex의 세션을 터미널에서 탐색, 미리보기, 재개, 삭제할 수 있는 Rust TUI 애플리케이션.

---

### 기능 구현 후 필수 체크리스트 (CRITICAL — 절대 빠뜨리지 말 것)

기능을 추가하거나 변경한 뒤에는 **반드시** 아래 문서들을 모두 업데이트해야 한다.
CLAUDE.md와 README.md 는 필수로 업데이트 하도록 한다.

| 변경 유형 | 업데이트할 문서 |
|-----------|----------------|
| 새 모듈/파일 추가 | `CLAUDE.md` 핵심 설계 원칙 (해당 시) |
| 새 의존성 추가 | `README.md` Dependencies |
| 새 Feature 추가 | `README.md` Features |
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

### 릴리스 체크리스트 (CRITICAL)

태그를 따기 **전에** 반드시:
1. `asm/Cargo.toml`의 `version`을 태그 버전과 일치시킬 것 (예: `v0.2.2` → `version = "0.2.2"`)
2. 버전 범프 커밋 후 해당 커밋에 태그를 달 것

---

### 핵심 설계 원칙

1. **공유 라이브러리** — 세션 로직을 `asm-core`로 분리하여 tre-file-manager와 공유
2. **외부 런타임 의존성 없음** — 순수 Rust, 시스템 라이브러리만 사용. 날짜 계산도 직접 구현
3. **exec로 세션 재개** — TUI 종료 후 `exec`로 프로세스를 대체하여 깔끔하게 세션 전환
4. **안전한 삭제** — 경로 정규화(canonicalize)로 디렉토리 traversal 방지, 빈 디렉토리만 정리
5. **ScanMode** — `Fast` (200줄 스캔, line count) 웹 서버용 / `Full` (전체 파일, 실제 메시지 카운트) TUI용
6. **asm-core 공개 API 안정성** — `asm-core`는 tre-file-manager가 `branch = "main"`으로 의존한다. 가능한 기존 응답 타입(struct 필드, enum variant)과 함수 시그니처를 유지하고, 새 함수/타입을 추가하는 방식으로 확장할 것
