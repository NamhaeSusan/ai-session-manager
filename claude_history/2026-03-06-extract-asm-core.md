# Extract asm-core shared library

## 작업 요약
- 단일 패키지 → Cargo workspace 전환 (asm-core + asm)
- session.rs 로직을 asm-core 라이브러리로 추출
- ScanMode::Fast/Full 지원, tre-file-manager와 공유
- Codex 파일 매칭 강화 (ends_with → exact/delimiter 매칭)

## 변경 파일
- `Cargo.toml` → workspace 정의
- `asm-core/Cargo.toml` + `asm-core/src/lib.rs` → 새 라이브러리
- `asm/Cargo.toml` + `asm/src/*.rs` → 기존 파일 이동, import 변경
- `src/session.rs` → 삭제
- `CLAUDE.md`, `README.md` → 워크스페이스 구조 반영

## 검증
- `cargo build` 성공
- `cargo install --path asm` 확인
