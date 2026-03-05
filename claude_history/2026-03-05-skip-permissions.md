# skip_permissions 설정 옵션 추가 + README 문서 업데이트

## 작업 요약
- `skip_permissions` 설정 옵션 추가: resume 시 permission bypass 플래그 자동 추가 (기본: true)
  - Claude Code: `--dangerously-skip-permissions`
  - Codex: `--dangerously-bypass-approvals-and-sandbox`
- README.md에 config 옵션 테이블, 누락 키바인딩(`S`, `?`) 추가

## 변경 파일
- `src/config.rs`: `skip_permissions: Option<bool>` 필드 추가
- `src/app.rs`: `skip_permissions` 필드 저장 및 resume 명령어에 bypass 플래그 추가
- `README.md`: Configuration 섹션, 키바인딩 추가
- `CLAUDE.md`: `skip_permissions` 설정 문서화

## 검증
- `cargo build` 성공
