# Homebrew Tap 배포 설정

## 작업 요약
- `asm --version` 플래그 추가 (`env!("CARGO_PKG_VERSION")` 사용)
- GitHub Actions release workflow 생성 (3 target: aarch64-apple-darwin, x86_64-apple-darwin, x86_64-unknown-linux-gnu)
- Homebrew Formula 생성 (`~/homebrew-tap/Formula/asm.rb` — 리포 생성 후 push 필요)

## 변경 파일
- `asm/src/main.rs` — `--version` / `-V` 플래그 처리 추가
- `.github/workflows/release.yml` — 새 파일 (release workflow)
- `README.md` — Homebrew 설치 방법 추가
- `CLAUDE.md` — 아키텍처 트리에 `.github/workflows/` 추가

## 외부 파일 (별도 리포)
- `~/homebrew-tap/Formula/asm.rb` — SHA256 placeholder, 첫 릴리스 후 업데이트 필요

## 검증
- `cargo run -p asm -- --version` → `asm 0.1.0` 출력 확인
