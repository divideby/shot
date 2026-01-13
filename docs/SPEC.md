# Shot — Package Manager for Claude Code

## Контекст
Артефакты для Claude Code (команды, агенты, правила, скрипты) разбросаны по проектам без структуры. Хочу переиспользовать их как пакеты — например, плагин `reading` со скриптом для оглавления книг с Литреса, командами для работы с книгами и правилами.

## Цель
CLI-инструмент **Shot** — пакетный менеджер для Claude Code артефактов на Rust.

## Что управляем

| Артефакт | Описание | Куда ставится | Контекст |
|----------|----------|---------------|----------|
| **commands** | Slash-команды (`/read`, `/toc`) | `.claude/commands/` | on-demand |
| **agents** | Субагенты | `.claude/agents/` | on-demand |
| **rules** | Правила для CLAUDE.md | Добавляются в `CLAUDE.md` | always |
| **scripts** | Внешние скрипты | `~/.local/bin/` или `.shot/bin/` | — |
| **specs** | Спецификации (справка для Claude) | `.shot/specs/` | on-demand через команду |

**Важно про контекст:** commands/agents загружаются только при вызове. Rules — всегда в контексте. Поэтому глобальные пакеты (`-g`) **не устанавливают rules** по умолчанию, только commands/agents/scripts.

## Скоуп

**MVP (v0.1):**
- `shot init` — создаёт `shot.toml`, `.shot/`
- `shot install <path>` — установка из локальной папки
- `shot install -g` — глобальная установка в `~/.claude/`
- `shot list` — список установленных
- `shot remove <pkg>` — удаление
- Локальный registry (папка `~/shot-packages/`)

**v0.2:**
- `shot install github:user/repo` — установка из GitHub
- `shot.lock` — фиксация версий
- Зависимости между пакетами

**Позже:**
- Собственный registry (shot.dev)
- `shot publish`, `shot search`, `shot update`
- Песочница для скриптов
- MCP-интеграция

## Архитектура

### Source — абстракция для registry

```rust
trait Source {
    fn resolve(&self, pkg: &str) -> Result<PackageMeta>;
    fn fetch(&self, pkg: &str, dest: &Path) -> Result<()>;
}

struct LocalSource { path: PathBuf }      // ~/shot-packages/
struct GitHubSource { token: Option<String> }  // github:user/repo
// struct RegistrySource { url: String }  // позже: shot.dev
```

MVP использует `LocalSource`. Добавление GitHub — реализация нового `Source`.

### Структура пакета

```
reading/
├── shot.toml           # манифест (обязателен)
├── commands/           # slash-команды
│   ├── read-book.md
│   └── toc.md
├── agents/             # субагенты
│   └── book-analyzer.md
├── rules.md            # правила для CLAUDE.md (опционально)
├── scripts/            # внешние скрипты
│   └── litres-toc.py
└── specs/              # справочные спецификации
    └── reading-workflow.md
```

### shot.toml (манифест пакета)

```toml
[package]
name = "reading"
version = "0.1.0"
description = "Инструменты для чтения книг"

# Что устанавливать (по умолчанию — всё из соответствующих папок)
[install]
commands = ["commands/*.md"]
agents = ["agents/*.md"]
rules = "rules.md"
scripts = ["scripts/*"]
specs = ["specs/*.md"]

# Зависимости (v0.2+)
[dependencies]
# base-tools = "0.1"
```

### shot.toml (манифест проекта)

```toml
# Создаётся shot init в корне проекта
[project]
name = "my-project"

[dependencies]
reading = { path = "~/shot-packages/reading" }
frontend = { path = "~/shot-packages/frontend" }
# react = { git = "github:user/prompts", tag = "v1.0" }  # v0.2+

# Алиасы команд (при конфликтах или для удобства)
[dependencies.reading.aliases]
build = "read-build"  # команда build из reading → /read-build

[dependencies.frontend.aliases]
# build не переименован → /build (frontend установлен первым)
```

Алиасы работают как в npm scripts — позволяют давать короткие имена и разрешать конфликты.

### Структура хранения (подход Cargo)

**Глобальный кэш** — все пакеты всех проектов:
```
~/.shot/
└── cache/
    ├── reading/
    │   └── 0.1.0/              # версия
    │       ├── shot.toml       # манифест = источник истины
    │       ├── commands/
    │       ├── agents/
    │       └── scripts/
    └── frontend/
        └── 0.2.0/
            └── ...
```

**Проект** — только манифест, lock и результат:
```
my-project/
├── shot.toml              # зависимости + алиасы
├── shot.lock              # зафиксированные версии
├── CLAUDE.md              # + rules из пакетов
└── .claude/
    ├── commands/
    │   └── read-book.md   # скопировано из кэша
    └── agents/
        └── book-analyzer.md
```

**Нет `.shot/` в проекте!** Как в Cargo — в git только `shot.toml` и `shot.lock`.

**Глобальная установка (`-g`):**
```
~/.claude/
├── commands/
│   └── read-book.md
└── agents/
    └── book-analyzer.md

~/.shot/
├── cache/                 # тот же кэш
└── global.toml            # глобальные зависимости
```

**Глобальная установка НЕ добавляет rules** — чтобы не засорять контекст.

### CLI-команды (MVP)

| Команда | Описание |
|---------|----------|
| `shot init` | Создаёт `shot.toml` в текущей папке |
| `shot install <source>` | Устанавливает пакет (путь или github:) |
| `shot install` | Устанавливает все зависимости из `shot.toml` |
| `shot install -g <source>` | Глобальная установка |
| `shot list` | Список установленных пакетов (из `shot.lock`) |
| `shot list -g` | Список глобальных пакетов |
| `shot remove <pkg>` | Удаляет пакет |
| `shot doctor` | Проверяет соответствие `.claude/` и `shot.lock` |
| `shot repair` | Переустанавливает всё из кэша по `shot.lock` |
| `shot cache clean` | Удаляет неиспользуемые пакеты из кэша |

### Разрешение конфликтов имён

Команды в пакетах — это файлы (`build.md`). При установке могут быть конфликты.

**Логика:**
1. Нет конфликта → копируем как есть (`build.md` → `/build`)
2. Конфликт → интерактивный промпт:

```
$ shot install ~/shot-packages/reading

Conflict: command 'build' already exists (from frontend)

  [1] Skip this command
  [2] Replace existing
  [3] Install as 'reading-build'
  [4] Custom name: ___

Choice [3]: 4
Custom name: read-build

✓ Installed reading v0.1.0
  + .claude/commands/read-book.md
  + .claude/commands/toc.md
  + .claude/commands/read-build.md (aliased from build)
```

**Результат в shot.toml:**
```toml
[dependencies.reading]
path = "~/shot-packages/reading"

[dependencies.reading.aliases]
build = "read-build"
```

**Non-interactive режим** (`--yes` или CI):
```bash
shot install reading --on-conflict=prefix  # reading-build
shot install reading --on-conflict=skip    # пропустить
shot install reading --on-conflict=replace # перезаписать
```

### shot.lock

Фиксирует версии и источники (как Cargo.lock):

```toml
[[package]]
name = "reading"
version = "0.1.0"
source = "path:~/my-packages/reading"

[[package]]
name = "frontend"
version = "0.2.0"
source = "github:user/prompts/frontend"
commit = "abc123"
```

**Для remove/repair:** читаем `shot.lock` → находим пакет в кэше → читаем его манифест → знаем какие файлы удалить/восстановить.

**Алиасы** хранятся в `shot.toml`, не в lock — они project-specific решения.

## Интеграция с Claude Code

Claude Code автоматически находит:
- `.claude/commands/*.md` — проектные команды
- `.claude/agents/*.md` — проектные агенты
- `~/.claude/commands/*.md` — глобальные команды
- `~/.claude/agents/*.md` — глобальные агенты

Shot копирует файлы в эти папки. Дополнительная конфигурация не нужна.

**Rules:** добавляются в `CLAUDE.md` в специальную секцию:
```markdown
<!-- shot:rules:start -->
## Reading Rules (from reading package)
...
<!-- shot:rules:end -->
```

При `shot remove` эта секция удаляется.

## Acceptance Criteria (MVP)

- [ ] `shot init` создаёт `shot.toml`
- [ ] `shot install ~/packages/reading` кэширует в `~/.shot/cache/`, копирует в `.claude/`
- [ ] `shot.lock` создаётся/обновляется при install
- [ ] `shot install` без аргументов устанавливает всё из `shot.toml`
- [ ] `shot install -g` устанавливает глобально в `~/.claude/`
- [ ] `shot list` показывает пакеты из `shot.lock`
- [ ] `shot remove reading` удаляет файлы и запись из lock
- [ ] `shot doctor` показывает расхождения между lock и `.claude/`
- [ ] `shot repair` восстанавливает `.claude/` из кэша
- [ ] Claude Code видит установленные команды
- [ ] При конфликте имён — интерактивный промпт
- [ ] Алиасы в `shot.toml` применяются при install/repair

## Риски

| Риск | Митигация |
|------|-----------|
| Конфликты имён команд | Интерактивный промпт с алиасами |
| Скрипты без зависимостей | MVP: скрипты должны быть self-contained |
| Безопасность скриптов | MVP: доверяем источнику. Позже: песочница |

## План реализации

### Этап 1: Каркас
1. `cargo new shot` — структура проекта
2. CLI с clap: init, install, list, remove, doctor, repair
3. Структуры данных: PackageManifest, ProjectManifest, LockFile
4. Парсинг TOML (serde + toml)

### Этап 2: Кэш и установка
5. `~/.shot/cache/` — создание структуры
6. `LocalSource` — копирование пакета в кэш
7. `shot install <path>` — кэширование + копирование в `.claude/`
8. `shot.lock` — создание/обновление

### Этап 3: Управление пакетами
9. `shot list` — чтение из `shot.lock`
10. `shot remove` — удаление файлов по манифесту из кэша
11. `shot doctor` — сравнение lock vs `.claude/`
12. `shot repair` — переустановка из кэша

### Этап 4: Конфликты и алиасы
13. Детекция конфликтов при install
14. Интерактивный промпт (dialoguer crate)
15. Сохранение алиасов в `shot.toml`

### Этап 5: Глобальная установка
16. `-g` флаг, `~/.shot/global.toml`
17. Установка в `~/.claude/`

### Этап 6: Тестирование
18. Создать тестовый пакет `reading`
19. E2E тесты всех команд
20. Проверить интеграцию с Claude Code

### Этап 7: GitHub (v0.2)
21. `GitHubSource` — скачивание из репо
22. Зависимости между пакетами

## Технологии

- **Язык:** Rust
- **CLI:** clap
- **Конфиги:** toml, serde
- **JSON:** serde_json
- **Файлы:** std::fs, walkdir
- **Интерактив:** dialoguer
- **GitHub (v0.2):** reqwest, octocrab

## Пример использования

```bash
# Создать новый проект
$ shot init
Created shot.toml

# Установить пакет для чтения книг
$ shot install ~/shot-packages/reading
✓ Installed reading v0.1.0
  + .claude/commands/read-book.md
  + .claude/commands/toc.md
  + scripts/litres-toc.py

# Установить пакет с конфликтом
$ shot install ~/shot-packages/frontend
Conflict: command 'toc' already exists (from reading)

  [1] Skip this command
  [2] Replace existing
  [3] Install as 'frontend-toc'
  [4] Custom name: ___

Choice [3]: 3

✓ Installed frontend v0.2.0
  + .claude/commands/build.md
  + .claude/commands/frontend-toc.md (aliased from toc)

# Проверить
$ shot list
reading  v0.1.0  (path:~/shot-packages/reading)
frontend v0.2.0  (path:~/shot-packages/frontend)
  aliases: toc → frontend-toc

# Теперь в Claude Code:
# /read-book, /toc (из reading)
# /build, /frontend-toc (из frontend)
```
