<p align="center">
  <a href="README.md">English</a> | <strong>한국어</strong>
</p>

<p align="center">
### 스크린샷

[스크린샷 보기](docs/screenshots.md)
</p>

# Better-Skillinfo

**MHGU Recomp에서 장비와 장식주의 스킬 정보를 확인하는 Forge 오버레이 플러그인입니다.**

아이템 설명만으로 알기 어려운 스킬 발동 조건과 효과를 표시하고, 장비나 장식주를 교체했을 때 전체 스킬 포인트가 어떻게 달라지는지 비교할 수 있습니다.

> MHGU 글로벌판 1.4.0을 대상으로 합니다. 일본어 표시를 지원하지만, 일본판 MHXX 게임은 지원하지 않습니다.

## 주요 기능

- 스킬 계통별 발동 포인트, 발동 스킬 이름, 효과 설명 표시
- 장비·장식주 교체 전후 수치 비교 및 중복 스킬 통합
- 검사·거너 타입이 맞지 않는 방어구의 호환 불가 안내
- 9개 표시 언어

게임 내 설명을 덮어쓰는 방식이 아니라 Forge를 통해 별도 패널을 표시합니다. 커서를 옮겨 예상값을 확인하는 동안 실제 장비나 장식주를 변경하지 않습니다.

## 지원 환경

| 항목 | 지원 범위 |
| --- | --- |
| 실행 환경 | Forge가 내장된 MHGU Recomp PC판 |
| 게임 데이터 | MHGU 글로벌판 1.4.0 |
| 표시 언어 | English, 한국어, 日本語, 简体中文, 繁體中文, Deutsch, Français, Italiano, Español |

## 설치

1. 게임을 종료합니다.
2. 배포 ZIP을 압축 해제합니다.
3. `Better_Skillinfo` 폴더를 게임의 `mods` 폴더에 넣습니다.
4. 게임의 모드 목록에서 `Better_Skillinfo`를 활성화합니다.

내용이 최대 높이를 넘으면 스크롤 대신 여러 페이지로 나눕니다. 가능한 한 스킬 한 묶음을 같은 페이지에 배치하고, 묶음 자체가 너무 길면 다음 페이지에도 스킬 제목을 반복합니다.

| 입력 | 동작 |
| --- | --- |
| L3를 누른 채 R 스틱 왼쪽 / 오른쪽 | 이전 / 다음 페이지 |
| Ctrl + ← / → | 이전 / 다음 페이지 |

## 소스에서 빌드

빌드에는 다음 도구가 필요합니다. 배포된 모드만 설치하는 사용자는 준비할 필요가 없습니다.

- Rust nightly 및 `rust-src` — 프로젝트의 `rust-toolchain.toml` 파일에 필요한 Rust 개발 환경이 지정되어 있습니다.
- devkitARM — `DEVKITARM` 또는 `DEVKITPRO` 설정, 또는 `arm-none-eabi-gcc`를 PATH에 등록
- Forge CLI — 아래 SDK 리비전 사용
- Python 3 — 선택 사항인 배포 패키징에 사용하며 외부 Python 패키지는 필요 없음

```powershell
cargo install --git https://github.com/Lawrence-al/forge-rs.git --rev f2af0622a2a0aa291ffbd3d2f7628c0e2d96d220 cargo-mhgu-forge --locked

# 프로젝트 폴더에서 빌드 및 패키징
./Build.ps1

# NRO만 빌드
./Build.ps1 -SkipPackage
```

### 소스 구성

| 경로 | 내용 |
| --- | --- |
| `src/` | 게임 연동, 입력 처리, 정보 패널, 페이지 분할, 번역, 스킬 데이터 |
| `conf/better_skillinfo.ini` | 내장 기본 설정 |
| `assets/` | 다국어 글꼴과 글꼴 라이선스 |
| `.cargo/`, `link/`, `armv7a-none-eabi.json` | ARM32 빌드·링크 설정 |
| `tools/` | 배포 패키징·설치 스크립트 |

빌드에 필요한 스킬 데이터는 `src/generated.rs`에 포함되어 있습니다. 원본 게임 덤프, 세이브, 로컬 분석 도구는 빌드에 필요하지 않습니다. 빌드 산출물은 `.gitignore`로 제외하며 의존성 고정을 위해 `Cargo.lock`은 유지합니다.

## 문제 제보

표시 오류를 발견하면 다음 내용을 함께 남겨 주세요.

- MHGU Recomp 및 Better-Skillinfo 버전
- 문제가 발생한 메뉴와 상세 페이지 번호
- 선택 장비·장식주, 현재 착용 상태, 검사·거너 타입
- 기대한 표시와 실제 표시, 가능하면 스크린샷
- 크래시가 발생했다면 해당 시점의 `log.txt`

모든 장비 조합을 개별 실행 검증한 것은 아니므로, 재현 가능한 화면과 조작 순서가 문제를 확인하는 데 도움이 됩니다.
