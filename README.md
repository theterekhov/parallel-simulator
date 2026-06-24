# Симулятор параллельных вычислений

Веб-приложение для наглядного моделирования многопоточного выполнения: deadlock-детекция, три стратегии планировщика, графовое представление состояния системы.

Курсовая работа, НИУ ВШЭ (Пермь), 2026.

## Архитектура

Три крейта в одном workspace:

- **simulator_core** — дискретно-событийное ядро (DES), DFS-детектор дедлоков, три стратегии планировщика. Компилируется в WASM.
- **backend** — HTTP-сервер на Axum, отдаёт статику и JSON-сценарии из `tasks/`.
- **frontend** — SPA на Leptos (CSR), рисует SVG-граф ресурсов, цветной лог, панель конфигурации.

## Быстрый старт

```bash

# зависимости
rustup target add wasm32-unknown-unknown
cargo install trunk

# сборка и запуск
make run
# → http://localhost:3000

```

Для разработки в двух терминалах:

```bash
make dev-back   # backend на :3000
make dev-front  # frontend на :8080 (hot-reload)
```

## Структура

```
├── Cargo.toml                 # workspace (resolver = "3")
├── Makefile                   # сборка, запуск, проверка
├── task.schema.json           # JSON Schema Draft-07
├── crates/
│   ├── simulator_core/src/    # lib, models, simulator, strategy
│   ├── backend/src/main.rs    # 3 REST-эндпоинта
│   └── frontend/              # Leptos SPA
│       ├── index.html
│       ├── style.css
│       └── src/
│           ├── main.rs        # корневой компонент
│           ├── lib.rs         # реэкспорт simulator_core
│           ├── models/        # toast, report
│           ├── utils.rs       # generate_svg
│           └── components/    # 5 UI-компонентов
└── tasks/                     # 7 встроенных сценариев
```

## Стратегии планировщика

| Стратегия | Принцип |
|-----------|---------|
| C (Pthreads) | Вытесняющий round-robin, FIFO-очереди |
| Python (GIL) | Кооперативная, один поток в момент |
| Go (Channels) | Честное FIFO по времени ожидания |

## Встроенные сценарии

1. Обедающие философы — 5 потоков, круговое ожидание
2. Сложный дедлок — цикл A→B→C→A
3. Производитель-потребитель — семафор capacity=3
4. Читатели и писатели — семафор capacity=2
5. Гонка за ресурс — 2 потока, 1 мьютекс
6. Голодание потоков — 6 потоков, 1 мьютекс, высокая конкуренция
7. Инверсия приоритетов — 3 потока с разными приоритетами

## API

| Метод | Путь | Описание |
|-------|------|----------|
| `GET` | `/api/tasks` | Список задач |
| `GET` | `/api/tasks/{id}` | Конфигурация задачи |
| `POST` | `/api/tasks/validate` | Валидация по схеме |

## Формат задачи

JSON с тремя обязательными полями: `metadata`, `environment`, `initial_state`, — и массивом `threads` с шагами (`compute` / `lock` / `unlock`). Полная спецификация в `task.schema.json`.

## Команды

```bash
make build      # production-сборка (trunk + cargo)
make run        # сборка + запуск
make check      # cargo check --workspace
make clean      # очистка артефактов
cargo test --workspace  # модульные тесты
```

## Требования

- Rust 1.85+ (edition 2024)
- wasm32-unknown-unknown target
- trunk
- Браузер: Chrome 90+, Firefox 88+, Safari 14+
