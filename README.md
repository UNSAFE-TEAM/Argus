<table>
  <tr>
    <td width="280">
      <img src="assets/argus.png" width="260" alt="Argus">
    </td>
    <td>
      <h1>Argus</h1>
      <p>
        <i>Argus Panoptes</i>, the many-eyed watcher from Greek mythology.
      </p>
      <p>
        Dynamic behavior observation tool based on Frida.
      </p>
      <p>
        <img alt="Rust" src="https://img.shields.io/badge/rust-f04041?style=for-the-badge&labelColor=c0282d&logo=rust">
        <img alt="Frida" src="https://img.shields.io/badge/Frida-111111?style=for-the-badge&logo=target&logoColor=white">
        <img alt="Version" src="https://img.shields.io/badge/version-v0.1.0-green?style=for-the-badge">
        <img alt="License" src="https://img.shields.io/github/license/UNSAFE-TEAM/Argus?label=license&style=for-the-badge">
      </p>
    </td>
  </tr>
</table>

## Overview

Argus is a Windows dynamic analysis tool built on top of [Frida](https://frida.re/).
It launches or attaches to a target process, injects a JavaScript runtime, loads
rule scripts, and streams structured events back to Rust for console or JSONL
output.

The current `v0.1.0` release is a preview focused on behavior observation and
anti-analysis research. It is intended for local malware-analysis labs, especially
VMware-based analysis environments.

## Features

- Spawn a target process and attach before resume.
- Attach to an existing process by PID.
- Optionally follow child processes through the Python Frida bindings.
- Load embedded scripts or a local scripts directory.
- Emit structured events with the `argus.frida.v1` schema.
- Prefer main-module call sites for `collect` and `triggered` event addresses.
- Output colorized console logs or machine-readable JSONL.
- Save logs to disk with per-event flush for crash-resistant output.
- Apply VMware-oriented anti-sandbox profiles.
- Detect common behavior across process, file, registry, service, network, sync object, and injection activity.

## Current Coverage

### Anti-Analysis

- `IsDebuggerPresent` return normalization.
- Module enumeration hiding for Frida-related modules.
- VMware-oriented CPU, memory, disk, WMI, firmware, MAC, registry, file, and D3D profile handling.
- User activity handling through cursor and last-input APIs.
- Window probing handling for common debugger, analysis, sandbox, and VM window names.

### Behavior

- Process creation and process exit.
- Shell execution.
- File create/write/delete/move/copy/attribute changes.
- Registry open/query/set/delete/rename activity.
- Service create/start/control/delete/config changes.
- Mutex, event, and semaphore activity.
- Network target identification through WinHTTP, WinINet, URLMon, DNS, Winsock TCP, UDP, and ConnectEx paths.
- Injection behavior detection for remote process memory and thread activity.

## Usage

Run a program:

```powershell
Argus.exe --exec ".\target.exe --flag value"
```

Short form:

```powershell
Argus.exe -e ".\target.exe"
```

Attach to a PID:

```powershell
Argus.exe --pid 1234
```

Use the VMware preset:

```powershell
Argus.exe -e ".\target.exe" -p vmware
```

Enable the behavior module:

```powershell
Argus.exe -e ".\target.exe" -p vmware -m behavior
```

Use JSONL output:

```powershell
Argus.exe -e ".\\target.exe" --output jsonl
```

Follow child processes through the Python Frida helper:

```powershell
Argus.exe -e ".\\target.exe" --follow-children
```

Save output to a file:

```powershell
Argus.exe -e ".\target.exe" --save out.jsonl
```

Save output without printing Argus events to the console:

```powershell
Argus.exe -e ".\target.exe" --quiet --save out.jsonl
```

Load scripts from a local directory instead of embedded scripts:

```powershell
Argus.exe -e ".\target.exe" --scripts-dir .\scripts
```

## Event Format

Rules send events through the bootstrap runtime. The Rust side receives and
formats events like:

```json
{
  "schema": "argus.frida.v1",
  "time": "2026-06-04T09:49:48.498Z",
  "event": "triggered",
  "tag": "behavior",
  "subject": {
    "name": "winhttp.dll!WinHttpConnect",
    "address": "0x7ff7744a15ca"
  },
  "data": {
    "action": "http_request",
    "network": {
      "host": "xxx.xxx.xxx.xxx",
      "port": "443",
      "method": "",
      "path": ""
    }
  }
}
```

Common event types:

| Event | Purpose |
| --- | --- |
| `init` | Runtime or module initialization. |
| `register` | A hook has been installed. |
| `collect` | API call input was collected. |
| `triggered` | A rule handled, normalized, or reported behavior. |
| `skip` | A module or API is unavailable. |
| `error` | Script/runtime error reporting. |
| `other` | Auxiliary telemetry. |

For `collect` and `triggered`, `subject.address` prefers the target program's
main-module call site. If Argus cannot recover a main-module caller, it keeps the
direct caller address. This keeps output useful for IDA, x64dbg, and runtime
triage while preserving system-library-originated behavior.

## Output Modes

Console output is intended for live reading:

```text
[triggered] [behavior] winhttp.dll!WinHttpConnect @ 0x7ff7744a15ca {"action":"http_request","network":{"host":"xxx.xxx.xxx.xxx","port":"443"}}
```

JSONL output is intended for storage, tooling, and later UI rendering:

```text
{"schema":"argus.frida.v1","time":"...","event":"triggered","tag":"behavior","subject":{"name":"winhttp.dll!WinHttpConnect","address":"0x..."},"data":{"action":"http_request","network":{"host":"xxx.xxx.xxx.xxx","port":"443"}}}
```

## Scripts

Rule scripts live in the `scripts` submodule:

```text
scripts -> https://github.com/UNSAFE-TEAM/Argus-Rules.git
```

Script layout:

```text
runtime/                  Argus bootstrap runtime loaded before rules
scripts/sensors/          Low-level API sensors
scripts/anti_debug/       Anti-debugging rules
scripts/anti_injection/   Frida/module hiding and injection-resistance rules
scripts/anti_sandbox/     VM/sandbox profile normalization rules
scripts/modules/behavior/ Behavior aggregation modules
scripts/presets/vmware/   VMware-specific profile rules
```

Clone with submodules:

```powershell
git clone --recurse-submodules https://github.com/UNSAFE-TEAM/Argus.git
```

If the repository was already cloned:

```powershell
git submodule update --init --recursive
```

Update rules:

```powershell
cd scripts
git pull origin main
cd ..
git add scripts
git commit -m "chore: update rules"
```

## Build

Build in debug mode:

```powershell
cargo build
```

Build in release mode:

```powershell
cargo build --release
```

The Frida Rust bindings may download the matching Frida core devkit during build.
If the download fails behind a proxy, configure Cargo/Git proxy settings or place
the devkit where `frida-sys` can find it.

If you use `--follow-children`, Argus also needs a Python interpreter with the
`frida` package available. Override the interpreter path with `ARGUS_PYTHON`
when `python` is not on `PATH`.

## Repository Layout

```text
runtime/          Frida bootstrap runtime
scripts/          External rule scripts submodule
src/cli/          Command-line parsing
src/frida/        Frida spawn, attach, script loading, and message handling
src/output/       Event parsing and output formatting
assets/           Project images
build.rs          Embedded script generation
```

## Roadmap

- Expand anti-debugging rules.
- Improve injection detection and process-hollowing coverage.
- Add shellcode dump workflows based on executable memory and execution triggers.
- Add x64dbg integration for automatic comments and navigation.
- Add offline Web UI for JSONL report exploration.
- Add unpacking-oriented dump and analysis helpers.

## License

Argus is licensed under the MIT License. See [LICENSE](LICENSE).

## Status

Argus is an early-stage research tool. The event schema, rule layout, and
behavior taxonomy may change while the runtime and rules stabilize.
