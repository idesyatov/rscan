# rscan

Fast CLI port scanner written in Rust. Scans TCP ports on hosts, hostnames, or across entire subnets. Async, concurrent, cross-platform.

## Features

- **Multiple targets**: scan several hosts, subnets, and hostnames in one command
- **DNS resolution** for hostnames
- **Flexible ports**: single (`80`), list (`22,80,443`), range (`1-1024`), mixed, or top-N
- **Service detection**: identifies well-known services by port number
- **Banner grabbing**: reads service version banners from open ports
- **Host discovery**: TCP ping check before scanning (`--ping`)
- **Exclude hosts**: skip specific IPs or subnets (`--exclude`)
- **Scan profiles**: `--fast` and `--full` presets
- **Retry**: retry timed-out connections (`--retry`)
- **Rate limiting**: cap connections per second (`--rate`)
- **Colored terminal output**
- **Multiple output formats**: text, JSON, CSV
- **File output**: save to text, JSON, or CSV files
- Cross-platform: Linux and Windows binaries from a single Docker build

## Requirements

- [Docker](https://docs.docker.com/get-docker/) — that's it. No Rust toolchain needed.

## Build

```bash
docker build -t rscan-builder .
docker run --rm -v ./dist:/dist rscan-builder
```

Rebuild from scratch:

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
rscan <TARGET>... [OPTIONS]
```

### Arguments

| Argument | Description |
|----------|-------------|
| `TARGET` | One or more: IP address, hostname, or CIDR subnet |

### Options

| Option | Default | Description |
|--------|---------|-------------|
| `-p, --ports <PORTS>` | `1-1024` | Ports: `80`, `22,80,443`, `1-1024` |
| `--top <N>` | — | Scan top N most common ports (overrides `-p`) |
| `-b, --banner` | — | Grab service banners |
| `--ping` | — | Discover alive hosts before scanning |
| `--exclude <HOSTS>` | — | Exclude IPs or CIDRs (comma-separated) |
| `--retry <N>` | `0` | Retry timed-out ports N times |
| `--rate <N>` | — | Max connections per second |
| `--fast` | — | Fast profile: top 100 ports, 200ms timeout, 200 threads |
| `--full` | — | Full profile: all 65535 ports, 2s timeout |
| `-t, --timeout <MS>` | `1000` | Connection timeout in milliseconds |
| `-j, --threads <NUM>` | `100` | Max concurrent connections |
| `--json` | — | Output as JSON |
| `-o, --output <FILE>` | — | Save text results to file |
| `--json-file <FILE>` | — | Save JSON results to file |
| `--csv-file <FILE>` | — | Save CSV results to file |
| `-v, --verbose` | — | Show progress and details |

### Examples

Scan multiple targets:

```bash
rscan 192.168.1.1 10.0.0.1 google.com -p 80,443
```

Fast scan with banner grabbing:

```bash
rscan 192.168.1.0/24 --fast -b
```

Full scan of all ports:

```bash
rscan 10.0.0.1 --full
```

Scan subnet with ping discovery, excluding gateway:

```bash
rscan 192.168.1.0/24 -p 22,80,443 --ping --exclude 192.168.1.1
```

Rate-limited scan with retries:

```bash
rscan 10.0.0.0/24 --top 20 --rate 500 --retry 2
```

Export to all formats:

```bash
rscan 192.168.1.1 --top 50 -b -o scan.txt --json-file scan.json --csv-file scan.csv
```

JSON output:

```bash
rscan google.com --top 10 --json -b
```

```json
[
  {
    "host": "142.250.185.14",
    "port": 80,
    "state": "open",
    "service": "http",
    "banner": "HTTP/1.1 200 OK"
  },
  {
    "host": "142.250.185.14",
    "port": 443,
    "state": "open",
    "service": "https"
  }
]
```

## Scan profiles

| Profile | Ports | Timeout | Threads |
|---------|-------|---------|---------|
| default | 1-1024 | 1000ms | 100 |
| `--fast` | top 100 | 200ms | 200 |
| `--full` | 1-65535 | 2000ms | 100 |

Profiles set defaults — explicit flags still override them.

## How it works

1. Resolves targets — DNS for hostnames, CIDR expansion for subnets
2. Optionally discovers alive hosts via TCP ping (`--ping`)
3. Applies host exclusions (`--exclude`)
4. Selects ports: explicit, range, or top-N by frequency
5. Scans each IP × port with async TCP connect, retries, and rate limiting
6. Optionally reads service banners from open ports
7. Identifies services by port number
8. Outputs colored results to terminal and/or saves to files

## Tech stack

- **Rust** — compiled, zero-cost abstractions
- **tokio** — async runtime for concurrent scanning
- **clap** — CLI argument parsing
- **colored** — terminal colors
- **serde** — JSON serialization
- **Docker** — reproducible cross-compilation via mingw-w64

## License

[MIT](LICENSE)
