<p align="center">
  <strong>English</strong> | <a href="README_ko.md">한국어</a>
</p>

<h3 align="center">Screenshots</h3>

<p align="center">
  <a href="docs/screenshots.md">View screenshots</a>
</p>

# Better-Skillinfo

**A Forge overlay plugin for MHGU Recomp or Switch Emulator that displays detailed skill information for equipment and decorations.**

It shows skill activation requirements and effects that may not be clear from item descriptions alone. It also lets you compare how your total skill points would change when replacing equipment or decorations.

> Designed for MHGU Global v1.4.0. Japanese is supported as a display language, but the Japanese version of MHXX is not supported.

## Features

- Displays activation thresholds, activated skill names, and effect descriptions for each skill category.
- Compares skill points before and after equipment or decoration changes, combining duplicate skill entries.
- Indicates when armor is incompatible with the current Blademaster or Gunner type.
- Supports 9 display languages.

Rather than replacing in-game descriptions, the plugin displays a separate panel through Forge. You can preview the expected changes by moving the cursor without actually changing your equipped gear or decorations.

## Compatibility

| Item | Supported environment |
| --- | --- |
| Runtime | MHGU Recomp PC version with built-in Forge support or a switch emulator with Forge installed |
| Game data | MHGU Global v1.4.0 |
| Display languages | English, 한국어, 日本語, 简体中文, 繁體中文, Deutsch, Français, Italiano, Español |

## Installation

1. Close the game.
2. Extract the release ZIP archive.
3. Place the `Better_Skillinfo` folder in the game's `mods` directory.
4. Enable `Better_Skillinfo` in the in-game mod list.

When the content exceeds the maximum panel height, it is split into multiple pages instead of using scrolling. The plugin tries to keep each skill group on the same page. If a group is too long to fit, its skill heading is repeated on the following page.

| Input | Action |
| --- | --- |
| Hold L3 + move the right stick left / right | Previous / next page |
| Ctrl + ← / → | Previous / next page |

If Insert does not open Forge, check that keyboard passthrough is enabled in the emulator.

## Building from source

The following tools are required to build the plugin. They are not needed if you only want to install a prebuilt release.

- Rust nightly and `rust-src` — The required Rust toolchain and components are specified in `rust-toolchain.toml`.
- devkitARM — Set `DEVKITARM` or `DEVKITPRO`, or add `arm-none-eabi-gcc` to your PATH.
- Forge CLI — Use the SDK revision specified below.
- Python 3 — Required only for optional release packaging. No third-party Python packages are needed.

```powershell
cargo install --git https://github.com/Lawrence-al/forge-rs.git --rev f2af0622a2a0aa291ffbd3d2f7628c0e2d96d220 cargo-mhgu-forge --locked

# Build and package from the project directory
./Build.ps1

# Build the NRO only
./Build.ps1 -SkipPackage
```

### Project structure

| Path | Description |
| --- | --- |
| `src/` | Game integration, input handling, information panels, pagination, translations, and skill data |
| `conf/better_skillinfo.ini` | Embedded default configuration |
| `assets/` | Multilingual fonts and font licenses |
| `.cargo/`, `link/`, `armv7a-none-eabi.json` | ARM32 build and linker configuration |
| `tools/` | Release packaging and installation scripts |

The skill data required for building is included in `src/generated.rs`. Original game dumps, save files, and local analysis tools are not required.

Build artifacts are excluded through `.gitignore`, while `Cargo.lock` is retained to ensure reproducible dependency versions.

## Reporting issues

If you encounter incorrect information or display issues, please include the following details:

- MHGU Recomp or Switch Emulator name and Better-Skillinfo versions
- The menu where the issue occurs and the specific page number
- Selected equipment or decorations, currently equipped gear, and Blademaster/Gunner type
- Expected behavior versus actual behavior, preferably with screenshots
- If a crash occurs, the relevant `log.txt`

Not every possible equipment combination has been individually tested in-game. Providing reproducible screenshots and step-by-step instructions will help identify and resolve issues.