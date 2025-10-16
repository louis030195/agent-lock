# screen-locker

cross-platform CLI tool to lock your screen with a PIN while keeping apps running and preventing sleep

perfect for AI agents, long-running processes, and background tasks that need to continue while you're away

## features

- PIN-based authentication (4-8 digits)
- keeps all applications running in background
- prevents system sleep/screen saver
- works on macOS and Windows
- terminal-based locking
- zero dependencies on system GUI

## installation

```bash
cargo install --path .
```

or build from source:

```bash
git clone https://github.com/yourusername/screen-locker.git
cd screen-locker
cargo build --release
```

binary will be at `target/release/screen-locker`

## usage

### setup PIN

first time setup:

```bash
screen-locker setup
```

follow prompts to set 4-8 digit PIN

### lock screen

```bash
screen-locker lock
```

this will:
- lock your terminal with PIN prompt
- keep all apps running
- prevent system sleep
- prevent screen saver

enter your PIN to unlock

### check status

```bash
screen-locker status
```

shows whether PIN is configured and ready to lock

## how it works

### sleep prevention

- **macOS**: uses `caffeinate` to prevent system sleep
- **Windows**: uses `SetThreadExecutionState` API

### locking mechanism

uses terminal-based locking that:
- blocks terminal input/output until correct PIN entered
- runs in foreground keeping all background processes active
- no GUI dependencies - works over SSH/remote sessions

### security

- PIN hashed with SHA-256
- config stored in user config directory (`~/.config/screen-locker/auth.json` on macOS)
- unlimited unlock attempts (for personal use - modify if needed for stricter security)

## use cases

- running AI agents that need uninterrupted execution
- long compilations or data processing
- downloads or uploads
- server processes on workstation
- any task that can't be interrupted by sleep/screen lock

## configuration

config file location:
- **macOS**: `~/.config/screen-locker/auth.json`
- **Windows**: `%APPDATA%\screen-locker\auth.json`

## development

run tests:

```bash
cargo test
```

build for release:

```bash
cargo build --release
```

## platform support

- ✅ macOS (tested on 10.15+)
- ✅ Windows (10+)
- ⚠️  Linux (untested - should work with modifications to sleep prevention)

## license

MIT

## author

Louis Beaumont

## contributing

PRs welcome for:
- Linux support
- additional security features
- GUI overlay option
- automatic lock on idle
