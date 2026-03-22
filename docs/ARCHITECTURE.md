# Архитектура — rscan

## Назначение
CLI port scanner. TCP-сканирование портов на хостах, подсетях, hostname.

## Стек
- **Язык:** Rust (stable)
- **Async runtime:** tokio (multi-threaded)
- **CLI:** clap (derive API)
- **Сериализация:** serde + serde_json
- **Цветной вывод:** colored
- **Сборка:** Docker-only, multi-stage (rust:latest → debian:bookworm-slim)
- **Кросс-компиляция:** mingw-w64 для Windows
- **Платформы:** Linux (x86_64), Windows (x86_64)

## Модули

### main.rs — точка входа
- CLI через clap derive: множественные targets, все флаги
- Профили сканирования: --fast (top100, 200ms, 200 threads), --full (65535, 2s)
- Оркестрация: parse targets → exclude → ping → scan → output

### scanner.rs — логика сканирования
- `ScanConfig` — конфигурация (timeout, concurrency, banners, retries, rate)
- `scan()` — async параллельное сканирование с semaphore
- `scan_port()` — TCP connect с retry
- `grab_banner_from_stream()` — чтение первой строки от сервиса
- `ping_host()` / `discover_hosts()` — TCP ping на 80/443/22
- Rate limiting через задержку между spawn

### network.rs — работа с адресами
- `parse_targets()` — множественные цели, дедупликация
- `parse_target()` — IP, CIDR, hostname (DNS resolve)
- `apply_excludes()` — исключение хостов (IP, CIDR)
- `parse_ports()` — одиночный, список, диапазон, смешанный

### output.rs — вывод результатов
- `print_text()` — цветная таблица (colored): green/open, yellow/service, cyan/port
- `print_json()` — JSON в stdout
- `save_text()` — plain text в файл (без цветов)
- `save_json()` — JSON в файл
- `save_csv()` — CSV в файл

### services.rs — маппинг сервисов
- `lookup(port)` — 100+ записей (ssh, http, mysql, redis, kubernetes...)
- `top_ports(n)` — топ-N портов по частотности (данные nmap)

## CLI-интерфейс
```
rscan <TARGET>... [OPTIONS]

Arguments:
  <TARGET>...   IP, CIDR, или hostname (множественные)

Options:
  -p, --ports <PORTS>      Порты [default: 1-1024]
  --top <N>                Топ-N портов (вместо -p)
  -b, --banner             Захват баннеров
  --ping                   TCP ping discovery
  --exclude <HOSTS>        Исключить хосты
  --retry <N>              Повторы при таймауте [default: 0]
  --rate <N>               Макс. соединений/сек
  --fast                   Профиль: top100, 200ms, 200 threads
  --full                   Профиль: 1-65535, 2000ms
  -t, --timeout <MS>       Таймаут [default: 1000]
  -j, --threads <NUM>      Параллельность [default: 100]
  --json                   JSON в stdout
  -o, --output <FILE>      Текст в файл
  --json-file <FILE>       JSON в файл
  --csv-file <FILE>        CSV в файл
  -v, --verbose            Подробный вывод
```

## Сборка и дистрибуция
- **На хосте Rust/Cargo НЕ установлены**
- `docker build -t rscan-builder . && docker run --rm -v ./dist:/dist rscan-builder`
- Multi-stage: builder (rust:latest + mingw) → slim (debian:bookworm-slim)
- Кэширование зависимостей через dummy main.rs
- `./dist/linux/rscan` + `./dist/windows/rscan.exe`
