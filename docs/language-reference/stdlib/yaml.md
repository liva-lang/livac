# YAML

> Parse and serialize YAML using the same dynamic Value as JSON. Backed by `serde_yaml`.

## Overview

YAML.parse and YAML.stringify mirror the JSON namespace API exactly: parse
returns a tuple `(value, err)` where the value is the same dynamic Value type
used by `JSON.parse`. All indexed access, field navigation, and type coercion
work identically.

## API

```liva
// Parse a YAML string into a dynamic Value.
let v, err = YAML.parse("name: liva\nversion: 2.1.0\n")
if err == "" {
    print(v["name"])     // "liva"
    print(v["version"])  // "2.1.0"
}

// Serialize a Value back to YAML.
let s, err = YAML.stringify(v)
```

## Type-safe parsing

Same type-hint pattern as `JSON.parse`:

```liva
class Config { name: string; version: string }

let cfg: Config, err = YAML.parse("name: liva\nversion: 2.1.0\n")
print(cfg.name)
```

See [json-basics.md](json-basics.md) for the full type-hint reference — every
pattern documented there applies to `YAML.parse` too.

## Errors

`YAML.parse` returns a non-empty `err` for malformed input. `YAML.stringify`
can fail on Values containing non-serializable contents (rare).
