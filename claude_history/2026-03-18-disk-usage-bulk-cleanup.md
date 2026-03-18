# 디스크 사용량 표시 & 일괄 정리

## 작업 요약
세션 JSONL 파일의 디스크 사용량을 표시하고 오래된 세션을 일괄 삭제하는 기능 추가.

## 변경된 파일
- `asm-core/src/lib.rs`: `SessionEntry`에 `file_size: u64` 필드 추가, `format_size()` 유틸 함수 추가
- `asm/src/tree.rs`: `SessionStats`에 `total_size` 추가, `by_tool`에 크기 정보 추가, `all_sessions()` 접근자 추가
- `asm/src/ui.rs`: 트리 뷰/프리뷰/Stats 팝업에 크기 표시, BulkCleanup/BulkCleanupConfirm 팝업 추가, `parse_iso_timestamp` pub(crate) 변경
- `asm/src/app.rs`: `BulkCleanup`/`BulkCleanupConfirm` 모드 추가, D 키바인딩, 일수 입력/대상 계산/삭제 실행 로직
- `CLAUDE.md`: 기능 상세 업데이트
- `README.md`: Features, Keybindings 업데이트

## 검증
- `cargo build` 성공
