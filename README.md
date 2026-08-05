# scry

Capture a screenshot and ask an AI about it.

`scry` grabs a monitor, encodes it as a PNG in memory, sends it to a vision-capable model, and prints the answer to stdout — nothing is written to disk. The default prompt asks for the answer only, with no reasoning or filler, so it composes well with pipes and scripts.

It is provider-agnostic: any model that accepts an image and returns text can be plugged in, whether that's a hosted API or something running locally.

Runs on Linux and Windows.

## Supported providers

| Provider | Default model | Select with | Status |
| --- | --- | --- | --- |
| Google (Gemini) | `gemini-2.5-flash` | `--provider gemini` | **working** |
| Anthropic (Claude) | `claude-opus-5` | `--provider anthropic` | **working** |
| OpenAI (ChatGPT) | — | — | planned |
| Local (Ollama, llama.cpp) | — | — | planned |

Each backend is one file under `src/providers/` — a request builder and a response parser — paired with a JSON template holding the model and request body. Everything upstream (capture, PNG encoding, prompt, output) is provider-neutral, so adding OpenAI or a local endpoint means adding one module and one template, then a variant to the `Provider` enum in `src/providers/mod.rs`.

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
  -l, --list                 List available monitors and exit
  -m, --monitor <MONITOR>    Index of the monitor to capture (defaults to the first)
  -p, --provider <PROVIDER>  AI provider to ask (defaults to the one set in the config file) [possible values: anthropic, gemini]
  -h, --help                 Print help
  -V, --version              Print version
```

```sh
scry                        # capture the first monitor, ask the default provider
scry --provider gemini      # ask Gemini for this run
scry --list                 # see available monitors, with index and resolution
scry -m 1 -p gemini         # capture the second monitor, ask Gemini
```

`--provider` overrides the default for a single run. To make a provider the permanent default, set `provider` in the config file (below).

## Configuration

Keys are resolved in this order, first hit wins:

1. **Environment variable** — `GEMINI_API_KEY` or `ANTHROPIC_API_KEY`
2. **Config file** — the matching entry under `[keys]`
3. **Prompt** — asks on stdin and saves the answer to the config file

### Environment variable

Nothing to add to `PATH` — an API key is an ordinary environment variable, not an executable.

```sh
export GEMINI_API_KEY="your-key-here"
scry --provider gemini
```

That lasts for the current shell only. To make it permanent for your user, append the same `export` line to `~/.bashrc` and reload it:

```sh
echo 'export GEMINI_API_KEY="your-key-here"' >> ~/.bashrc
source ~/.bashrc
```

For one-off use without exporting anything, prefix the command:

```sh
GEMINI_API_KEY="your-key-here" scry --provider gemini
```

An exported key is visible to every process you launch from that shell, and `~/.bashrc` is a plaintext file. If you'd rather not have it in your shell environment, skip this and let `scry` store it in the config file instead.

### Config file

| OS | Config file |
| --- | --- |
| Linux | `~/.config/scry/default-config.toml` (honours `$XDG_CONFIG_HOME`) |
| Windows | `%APPDATA%\scry\config\default-config.toml` |

```toml
provider = "gemini"

[keys]
anthropic = "sk-ant-..."
gemini = "AIza..."
```

`provider` sets the default backend when `--provider` isn't passed; omit it and Anthropic is used. Keys are stored per provider, so both can live side by side and you switch with the flag alone.

Keys are stored in plaintext TOML, so treat that file the way you'd treat any other credential on disk. Clearing a key's entry triggers the setup prompt again on the next run. A config from an older version with a bare `api_key = "..."` is migrated to `keys.anthropic` automatically.

### Putting `scry` itself on `PATH`

That's a separate thing from the key. `cargo install --path .` drops the binary in `~/.cargo/bin`, which rustup normally adds to your `PATH` already. Check with `command -v scry`; if it comes back empty:

```sh
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

## Notes

- **Wayland**: depending on your compositor, capture goes through the wlr-screencopy protocol or the xdg-desktop-portal. On GNOME and KDE, expect a screen-sharing permission dialog the first time.
- **Model and token limit** live in the per-provider templates, `src/providers/gemini.json` and `src/providers/anthropic.json`. Each is the request body for that API, with the model at the top. Gemini takes its model in the URL, so `scry` lifts that field out of the template when building the request. Templates are embedded at compile time via `include_str!`, so edits require a rebuild.
- **The prompt** is in `src/prompt.rs`, shared by every provider. Change it there if you want something other than terse answer-only output.
- **Capture and output** (`src/capture.rs`, `src/output.rs`) have no provider-specific code — a new backend only has to build a request and parse a reply.
