# agent-lock

lock your screen with a PIN while keeping AI agents and background processes running

**the problem**: macOS/Windows screen locks stop background processes, kill SSH connections, and interrupt long-running tasks

**the solution**: fullscreen GUI overlay that blocks interaction but keeps everything running in the background

## features

- fullscreen black overlay with PIN unlock
- global hotkey (Cmd+Shift+L on macOS)
- prevents system sleep while locked
- keeps all apps running (AI agents, downloads, compilations, servers)
- works on macOS (Windows coming soon)
- lightweight daemon mode

## installation

```bash
cargo install --path .
```

or build from source:

```bash
git clone https://github.com/louis030195/agent-lock.git
cd agent-lock
cargo build --release
```

binary will be at `target/release/agent-lock`

## usage

### setup PIN

first time setup:

```bash
agent-lock setup
```

enter 4-8 digit PIN

### lock immediately

```bash
agent-lock lock
```

shows fullscreen black overlay with PIN input - enter your PIN to unlock

### daemon mode (recommended)

```bash
agent-lock daemon
```

runs in background - press **Cmd+Shift+L** to lock screen anytime

tip: add to startup items for always-available locking

### check status

```bash
agent-lock status
```

shows PIN config status and usage instructions

## how it works

### fullscreen overlay

creates a native GUI window that:
- covers entire screen at highest window level
- captures all keyboard/mouse input
- shows only PIN entry field
- unlocks when correct PIN entered

### sleep prevention

- **macOS**: uses `caffeinate` command
- **Windows**: uses `SetThreadExecutionState` API

### security

- PIN hashed with SHA-256
- config stored at `~/.config/screen-locker/auth.json`
- unlimited unlock attempts (personal use - for stricter security, modify source)

## use cases

- **AI agents**: autonomous agents that run for hours/days
- **long compilations**: rust, c++, large codebases
- **data processing**: ETL pipelines, ML training
- **downloads/uploads**: large files, backups
- **server processes**: local dev servers, databases
- **SSH sessions**: keep connections alive
- **screen sharing**: lock screen without disconnecting remote viewers

## why not use built-in screen lock?

macOS/Windows screen locks:
- interrupt background processes
- disconnect SSH sessions
- pause some applications
- can't customize unlock mechanism
- no global hotkey support

agent-lock:
- guarantees all processes keep running
- maintains all network connections
- simple PIN unlock
- customizable hotkey
- designed for developers running background tasks

## configuration

config file: `~/.config/screen-locker/auth.json` (macOS)

contains SHA-256 hash of your PIN

## development

```bash
cargo test
cargo build --release
```

## roadmap

- [ ] Windows implementation
- [ ] Linux support
- [ ] custom hotkey configuration
- [ ] rate limiting for failed attempts
- [ ] automatic lock on idle
- [ ] multiple monitor support

## license

MIT

## author

[Louis Beaumont](https://github.com/louis030195)
