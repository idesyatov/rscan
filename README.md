# rscan

Fast CLI port scanner written in Rust. Scans TCP ports on hosts, hostnames, or across entire subnets. Async, concurrent, cross-platform.

## Features

- **Multiple targets**: scan several hosts, subnets, and hostnames in one command
- **Target file**: load hosts from a file (`-i hosts.txt`)
- **DNS resolution** for hostnames
- **Flexible ports**: single (`80`), list (`22,80,443`), range (`1-1024`), mixed, or top-N
- **Service detection**: identifies well-known services by port number
- **Banner grabbing**: reads service version banners from open ports
- **Host discovery**: TCP ping check before scanning (`--ping`)
- **Exclude hosts**: skip specific IPs or subnets (`--exclude`)
- **Scan profiles**: `--profile fast`, `full`, `stealth`
- **Retry**: retry timed-out connections (`--retry`)
- **Rate limiting**: cap connections per second (`--rate`)
- **Colored terminal output**
- **Multiple output formats**: text, JSON, CSV
- **Auto-detect format**: `-o scan.json` saves JSON, `-o scan.csv` saves CSV
- **Multiple outputs**: `-o scan.txt -o scan.json -o scan.csv` in one command
- Cross-platform: Linux and Windows binaries from a single Docker build

## Installation

### From GitHub Releases

Download the latest binary for your platform from [Releases](https://github.com/idesyatov/rscan/releases):

- **Linux**: `rscan-linux-amd64`
- **Windows**: `rscan-windows-amd64.exe`

```bash
# Linux
curl -L https://github.com/idesyatov/rscan/releases/latest/download/rscan-linux-amd64 -o rscan
chmod +x rscan
sudo mv rscan /usr/local/bin/

# Windows (PowerShell)
Invoke-WebRequest -Uri https://github.com/idesyatov/rscan/releases/latest/download/rscan-windows-amd64.exe -OutFile rscan.exe
```

### Build from source

Requires [Docker](https://docs.docker.com/get-docker/) — no Rust toolchain needed.

```bash
docker build -t rscan-builder .

# Linux / macOS
docker run --rm -v ./dist:/dist rscan-builder

# Windows (PowerShell)
docker run --rm -v "${PWD}/dist:/dist" rscan-builder
```

Rebuild from scratch:

```bash
docker build --no-cache -t rscan-builder .
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
rscan -i targets.txt [OPTIONS]
```

### Options

**Target:**

| Option | Description |
|--------|-------------|
| `<TARGET>...` | IP, CIDR, or hostname (multiple allowed) |
| `-i, --target-file <FILE>` | Read targets from file |
| `--exclude <HOSTS>` | Exclude IPs/CIDRs (comma-separated) |

**Ports:**

| Option | Default | Description |
|--------|---------|-------------|
| `-p, --ports <PORTS>` | `1-1024` | Ports: `80`, `22,80,443`, `1-1024` |
| `--top <N>` | — | Scan top N most common ports |

**Scan:**

| Option | Default | Description |
|--------|---------|-------------|
| `-t, --timeout <MS>` | `1000` | Connection timeout in milliseconds |
| `-j, --threads <NUM>` | `100` | Max concurrent connections |
| `-b, --banner` | — | Grab service banners |
| `--ping` | — | Discover alive hosts first |
| `--retry <N>` | `0` | Retry timed-out ports N times |
| `--rate <N>` | — | Max connections per second |

**Profile:**

| Option | Description |
|--------|-------------|
| `--profile fast` | Top 100 ports, 200ms timeout, 200 threads |
| `--profile full` | All 65535 ports, 2s timeout |
| `--profile stealth` | Top 20, 3s timeout, 10 threads, 10 conn/s |

**Output:**

| Option | Description |
|--------|-------------|
| `--json` | Output JSON to stdout |
| `-o, --output <FILE>` | Save to file (format by extension: .txt, .json, .csv) |
| `-v, --verbose` | Show progress and details |

### Target file format

```
# hosts.txt — one target per line
# Comments start with #, blank lines are ignored

192.168.1.1
192.168.1.0/24
10.0.0.1
google.com
example.com
```

### Examples

Scan from a target file:

```bash
rscan -i hosts.txt -p 80,443
```

Combine file and CLI targets:

```bash
rscan 10.0.0.1 -i hosts.txt --top 20 -b
```

Scan multiple targets:

```bash
rscan 192.168.1.1 10.0.0.1 google.com -p 80,443
```

Fast scan with banner grabbing:

```bash
rscan 192.168.1.0/24 --profile fast -b
```

Full scan of all ports:

```bash
rscan 10.0.0.1 --profile full
```

Stealth scan:

```bash
rscan 10.0.0.0/24 --profile stealth
```

Scan subnet with ping discovery, excluding gateway:

```bash
rscan 192.168.1.0/24 -p 22,80,443 --ping --exclude 192.168.1.1
```

Rate-limited scan with retries:

```bash
rscan 10.0.0.0/24 --top 20 --rate 500 --retry 2
```

Export to all formats in one command:

```bash
rscan google.com --top 50 -b -o scan.txt -o scan.json -o scan.csv
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

| Profile | Ports | Timeout | Threads | Rate |
|---------|-------|---------|---------|------|
| default | 1-1024 | 1000ms | 100 | — |
| fast | top 100 | 200ms | 200 | — |
| full | 1-65535 | 2000ms | 100 | — |
| stealth | top 20 | 3000ms | 10 | 10/s |

Profiles set defaults — explicit flags still override them.

## How it works

1. Collects targets from CLI arguments and/or target file (`-i`)
2. Resolves hostnames via DNS, expands CIDRs
3. Applies host exclusions (`--exclude`)
4. Optionally discovers alive hosts via TCP ping (`--ping`)
5. Selects ports: explicit, range, or top-N by frequency
6. Scans each IP × port with async TCP connect, retries, and rate limiting
7. Optionally reads service banners from open ports
8. Identifies services by port number
9. Outputs colored results to terminal and/or saves to files

## Tech stack

- **Rust** — compiled, zero-cost abstractions
- **tokio** — async runtime for concurrent scanning
- **clap** — CLI argument parsing
- **colored** — terminal colors
- **serde** — JSON serialization
- **Docker** — reproducible cross-compilation via mingw-w64

## License

[MIT](LICENSE)
