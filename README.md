# scry

Capture a screenshot and ask an AI about it.

`scry` grabs a monitor, encodes it as a PNG in memory, sends it to a vision-capable model, and prints the answer to stdout — nothing is written to disk. The default prompt asks for the answer only, with no reasoning or filler, so it composes well with pipes and scripts.

The goal is to stay provider-agnostic: any model that accepts an image and returns text should be usable, whether that's a hosted API or something running locally.

Runs on Linux and Windows.

## Supported providers

| Provider | Models | Status |
| --- | --- | --- |
| Anthropic (Claude) | `claude-opus-5`, other vision models | **working** |
| OpenAI (ChatGPT) | GPT vision models | planned |
| Google (Gemini) | Gemini vision models | planned |
| Local (Ollama, llama.cpp) | any vision model you host | planned |

Only the Anthropic backend is implemented today — `src/ai.rs` speaks the Messages API directly. The other rows are the intended direction, not shipped behaviour. Adding a provider means a request builder and a response parser for that API; everything upstream of it (capture, PNG encoding, prompt, output) is already provider-neutral.

## Prerequisites

Rust (edition 2024, so a reasonably recent toolchain) plus a few system libraries. The Rust crates are handled by Cargo; the system libraries are not, and their absence shows up as cryptic build-script or linker errors.

### Linux

`xcap` depends on `pipewire` unconditionally on Linux, and links against gbm/EGL/Wayland/xcb. You need the **development** packages, not just the runtime ones — the runtime ships `libgbm.so.1`, but the linker wants the bare `libgbm.so` symlink.

| What needs it | Why | Fedora | Debian / Ubuntu |
| --- | --- | --- | --- |
| `libspa-sys` build script | finds `libpipewire-0.3.pc` via pkg-config | `pipewire-devel` | `libpipewire-0.3-dev` |
| final link | `-lgbm` | `mesa-libgbm-devel` | `libgbm-dev` |
| final link | `-lEGL`, `-lwayland-client`, `-lwayland-server`, `-lxcb` | `mesa-libEGL-devel wayland-devel libxcb-devel` | `libegl1-mesa-dev libwayland-dev libxcb1-dev` |
| `libspa-sys` bindgen | needs libclang | `clang` | `clang` |
| `aws-lc-sys` (TLS, via `reqwest`) | builds with CMake and a C compiler | `cmake gcc perl pkgconf` | `cmake build-essential perl pkg-config` |

Fedora:

```sh
sudo dnf install pipewire-devel mesa-libgbm-devel mesa-libEGL-devel wayland-devel \
                 libxcb-devel clang cmake gcc perl pkgconf
```

Debian / Ubuntu:

```sh
sudo apt install libpipewire-0.3-dev libgbm-dev libegl1-mesa-dev libwayland-dev \
                 libxcb1-dev clang cmake build-essential perl pkg-config
```

Most desktop installs already have everything except `pipewire-devel` and `mesa-libgbm-devel`; the full list is here for a clean machine.

### Windows

No pipewire or gbm involved — those dependencies are gated on `cfg(target_os = "linux")`, and screen capture goes through the Win32 APIs. The build requirement comes from `aws-lc-rs`, the TLS backend `reqwest` 0.13 uses by default:

- **Visual Studio Build Tools** with the "Desktop development with C++" workload
- **CMake** — `winget install Kitware.CMake`
- **NASM** — `winget install NASM`

On x86_64 you can skip the NASM install: `aws-lc-sys` ships prebuilt assembly objects, but only uses them if you opt in explicitly.

```powershell
$env:AWS_LC_SYS_PREBUILT_NASM=1
cargo build --release
```

Without either NASM on `PATH` or that variable set, the build fails with `NASM command not found! Build cannot continue.`

> The Windows requirements are derived from the dependency sources (`aws-lc-sys` 0.43 `use_prebuilt_nasm()`), not from an executed Windows build — only the Linux build has been verified here.

## Build

```sh
cargo build --release
```

The binary lands at `target/release/scry`. To put it on your `PATH`:

```sh
cargo install --path .
```

## Usage

```
Capture a screenshot and ask an AI about it

Usage: scry [OPTIONS]

Options:
  -l, --list               List available monitors and exit
  -m, --monitor <MONITOR>  Index of the monitor to capture (defaults to the first)
  -h, --help               Print help
  -V, --version            Print version
```

```sh
scry              # capture the first monitor and ask about it
scry --list       # see which monitors are available, with index and resolution
scry -m 1         # capture the second monitor
```

A `--provider` flag for choosing between backends is planned, alongside the providers above.

## Configuration

On first run, `scry` prompts for an API key on stdin and saves it. Later runs read it back silently.

| OS | Config file |
| --- | --- |
| Linux | `~/.config/scry/default-config.toml` (honours `$XDG_CONFIG_HOME`) |
| Windows | `%APPDATA%\scry\config\default-config.toml` |

```toml
api_key = "sk-ant-..."
```

The config currently holds one key, used for Anthropic. Multi-provider support will extend this to a provider selection plus per-provider keys; the single-key file will keep working as the Anthropic default.

The key is stored in plaintext TOML, so treat that file the way you'd treat any other credential on disk. Deleting it triggers the setup prompt again.

## Notes

- **Wayland**: depending on your compositor, capture goes through the wlr-screencopy protocol or the xdg-desktop-portal. On GNOME and KDE, expect a screen-sharing permission dialog the first time.
- **Model and token limit** live in `src/payload_template.json`, which is the request body sent to the API. It's embedded at compile time via `include_str!`, so edits require a rebuild. This file is Anthropic-shaped today; per-provider templates are part of the multi-provider work.
- **The prompt** is in `src/prompt.rs`. Change it there if you want something other than terse answer-only output.
- **Capture and output** (`src/capture.rs`, `src/output.rs`) have no provider-specific code — a new backend only has to build a request and parse a reply.
