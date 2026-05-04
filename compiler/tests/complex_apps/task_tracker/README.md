# Task Tracker — multi-module exemplar

A small CLI app that exercises classes, enums, modules and the standard
library, written entirely in idiomatic Liva (per the language skill).

## Layout

```
task_tracker/
├── main.liva                # entry point — composes everything
├── models/
│   ├── task.liva            # Status / Priority enums + Task class
│   └── store.liva           # TaskStore class (CRUD over [Task])
├── services/
│   ├── filters.liva         # byStatus / byPriority / search / sortedByPriorityDesc
│   └── report.liva          # buildReport / toCsv
└── tests/
    ├── task.test.liva       # 12 tests
    ├── store.test.liva      # 13 tests
    └── filters.test.liva    #  9 tests
```

Each file imports from neighbours via relative paths (`./task`, `../models/task`).

## Running the demo

```bash
livac build compiler/tests/complex_apps/task_tracker/main.liva --output /tmp/tt
/tmp/tt/target/debug/liva_project
```

Expected output starts with:

```
== Task Report ==
total: 5
open: 2 | in-progress: 2 | done: 1
```

## Running the tests

The suite uses Liva's built-in test framework (`import { describe, test, expect } from "liva/test"`):

```bash
cd compiler/tests/complex_apps/task_tracker
livac test
```

Expected: **34 passed, 0 failed** across 3 files.

## Skill-faithful conventions used here

- No `fn` / `class` keywords — top-level `name(args): T { ... }` defines a function, `Name { ... }` defines a class.
- No semicolons; newline terminates statements.
- Counters use `x = x + 1` (no `+=`/`++`).
- Enums are accessed with dot syntax: `Status.Open`, `Priority.High`.
- String templates: `$"#{id} [{status}/{priority}] {title}"`.
- Underscore-prefixed fields are private (`_tasks`, `_nextId`).
- Imports are extension-less and use relative paths.
