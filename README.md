# rscan

Fast CLI port scanner written in Rust. Scans TCP ports on a single host or across an entire subnet. Async, concurrent, cross-platform.

## Features

- Scan a single host or CIDR subnet (e.g. `192.168.1.0/24`)
- Flexible port specification: single (`80`), list (`22,80,443`), range (`1-1024`), or mixed (`22,80,100-200`)
- Async concurrent scanning with configurable parallelism
- Configurable connection timeout
- Text table and JSON output formats
- Verbose mode with scan progress
- Cross-platform: builds for Linux and Windows from a single Docker command

## Requirements

- [Docker](https://docs.docker.com/get-docker/) — that's it. No Rust toolchain needed on the host.

## Build

```bash
docker build -t rscan-builder .
docker run --rm -v ./dist:/dist rscan-builder
```

Rebuild from scratch (no cache):

```bash
docker build --no-cache -t rscan-builder .
docker run --rm -v ./dist:/dist rscan-builder
```

Binaries appear in `./dist/`:

```
dist/
  linux/rscan        # ELF x86_64
  windows/rscan.exe  # PE x86_64
```

## Usage

```bash
rscan <TARGET> [OPTIONS]
```

### Arguments

| Argument | Description |
|----------|-------------|
| `TARGET` | IP address or CIDR subnet |

### Options

| Option | Default | Description |
|--------|---------|-------------|
| `-p, --ports <PORTS>` | `1-1024` | Ports: `80`, `22,80,443`, `1-1024`, or `22,80,100-200` |
| `-t, --timeout <MS>` | `1000` | Connection timeout in milliseconds |
| `-j, --threads <NUM>` | `100` | Max concurrent connections |
| `--json` | — | Output as JSON |
| `-v, --verbose` | — | Show scan progress |
| `-h, --help` | — | Show help |
| `-V, --version` | — | Show version |

### Examples

Scan common ports on a single host:

```bash
rscan 192.168.1.1 -p 22,80,443,8080
```

```
Scan complete: 4 ports scanned, 2 open

HOST                 PORT       STATE
----------------------------------------
192.168.1.1          22         open
192.168.1.1          80         open
```

Scan an entire subnet for port 80:

```bash
rscan 192.168.1.0/24 -p 80
```

Scan with JSON output and custom timeout:

```bash
rscan 10.0.0.1 -p 1-1024 --json -t 500
```

```json
[
  {
    "host": "10.0.0.1",
    "port": 22,
    "state": "open"
  },
  {
    "host": "10.0.0.1",
    "port": 80,
    "state": "open"
  }
]
```

Verbose scan with 200 concurrent threads:

```bash
rscan 10.0.0.0/24 -p 22,80,443 -j 200 -v
```

## How it works

1. Parses target into a list of IP addresses (single IP or CIDR expansion)
2. Parses port specification into a list of ports
3. For each IP × port combination, performs an async TCP connect with timeout
4. Concurrency is limited via a semaphore (configurable with `--threads`)
5. Collects open ports and outputs results as a table or JSON

## Tech stack

- **Rust** — compiled, zero-cost abstractions, no runtime overhead
- **tokio** — async runtime for concurrent scanning
- **clap** — CLI argument parsing with derive macros
- **serde** — JSON serialization
- **Docker** — reproducible build environment, cross-compilation via mingw-w64

## License

MIT
