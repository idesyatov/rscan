# Архитектура — rscan

## Назначение
CLI port scanner. Сканирование TCP-портов на хосте или в подсети.

## Стек
- **Язык:** Rust (stable)
- **Async runtime:** tokio (multi-threaded)
- **CLI:** clap (derive API)
- **Сериализация:** serde + serde_json
- **Сборка:** Docker-only (Rust/Cargo на хосте не требуются)
- **Кросс-компиляция:** mingw-w64 для Windows-таргета
- **Целевые платформы:** Linux (x86_64-unknown-linux-gnu), Windows (x86_64-pc-windows-gnu)

## Модули

### main.rs — точка входа
- Определение CLI-интерфейса через clap (derive)
- Валидация входных данных
- Запуск async runtime, вызов сканера, вывод результатов

### scanner.rs — логика сканирования
- `scan_host(ip, ports, timeout)` — сканирование списка портов на одном хосте
- `scan_port(ip, port, timeout)` — TCP connect к одному порту
- Параллельное сканирование через tokio tasks с ограничением concurrency (semaphore)

### network.rs — работа с адресами
- Парсинг одиночного IP-адреса
- Парсинг CIDR-нотации (192.168.1.0/24) → список IP
- Парсинг портов: одиночный (80), список (22,80,443), диапазон (1-1024)

### output.rs — вывод результатов
- Текстовый формат (таблица: host, port, status)
- JSON-формат (массив объектов)
- Verbose-режим (прогресс сканирования)

## CLI-интерфейс
```
rscan <TARGET> [OPTIONS]

Arguments:
  <TARGET>    IP-адрес или подсеть в CIDR (192.168.1.1, 10.0.0.0/24)

Options:
  -p, --ports <PORTS>      Порты: 80 | 22,80,443 | 1-1024 [default: 1-1024]
  -t, --timeout <MS>       Таймаут соединения в мс [default: 1000]
  -j, --threads <NUM>      Макс. параллельных соединений [default: 100]
      --json               Вывод в JSON
  -v, --verbose            Подробный вывод
  -h, --help               Справка
  -V, --version            Версия
```

## Алгоритм работы
1. Парсинг аргументов (clap)
2. Парсинг target → список IP-адресов (network.rs)
3. Парсинг портов → список портов (network.rs)
4. Для каждого IP × порт → async TCP connect с таймаутом (scanner.rs)
5. Сбор результатов → форматирование и вывод (output.rs)

## Ограничение concurrency
Semaphore (tokio::sync::Semaphore) ограничивает количество одновременных TCP-соединений. По умолчанию 100, настраивается через --threads.

## Сборка и дистрибуция
- **На хосте Rust/Cargo НЕ установлены**
- Сборка: `docker build -t rscan-builder . && docker run --rm -v ./dist:/dist rscan-builder`
- Пересборка без кэша: `docker build --no-cache -t rscan-builder .`
- Dockerfile на базе `rust:latest` + `gcc-mingw-w64-x86-64` для Windows-таргета
- Внутри контейнера: две сборки — Linux (native) и Windows (cross-compile)
- Результат выводится через volume mount в `./dist/`:
  - `./dist/linux/rscan` — Linux ELF x86_64
  - `./dist/windows/rscan.exe` — Windows PE x86_64
- Docker НЕ используется для запуска утилиты — только для сборки

## Тестирование
- Unit-тесты: запуск через Docker (Linux-сборка)
- Парсинг адресов, портов, CIDR
- Integration-тесты: сканирование localhost
