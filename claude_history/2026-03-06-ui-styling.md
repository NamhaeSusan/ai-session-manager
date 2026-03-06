# UI 개선: 프리뷰 메타데이터 보강 + 트리뷰 스타일링

## 변경 파일
- `src/ui.rs`

## 변경 내용

### 프리뷰 패널
- Tool, Session ID, File 메타데이터 추가
- 라벨(DarkGray) / 값(White) 색상 분리
- Tool 이름 색상 구분 (Claude Code → Cyan, Codex → Green)
- 섹션 구분선 DarkGray, Last Prompt 텍스트 Yellow 강조

### 트리뷰
- `●` 마커 tool별 색상 구분 (Claude Code → Cyan, Codex → Green)
- prompt truncate 길이 확대 (flat: 30→40, nested: 40→50)

### 삭제 확인 팝업
- 팝업 크기 확장 (40x5 → 50x8)
- 삭제 대상 프로젝트명, ID(앞 8자), 첫 프롬프트 표시
- `draw_confirm_popup` 시그니처에 `app` 파라미터 추가

## 검증
- `cargo build` 성공
