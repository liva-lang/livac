# TOML

> Parse and serialize TOML using the same dynamic Value as JSON. Backed by the `toml` crate.

## Overview

TOML.parse and TOML.stringify mirror the JSON namespace API. parse returns a
tuple `(value, err)` over the same dynamic Value used by `JSON.parse`. All
indexed access, field navigation, and type coercion work identically.

## API

```liva
let src = "name = \"liva\"\nversion = \"2.1.0\"\n[author]\nname = \"liva-lang\"\n"
let v, err = TOML.parse(src)
if err == "" {
    print(v["name"])              // "liva"
    print(v["author"]["name"])    // "liva-lang"
}

let s, err = TOML.stringify(v)
```

## Top-level table requirement

`TOML.stringify` requires the top-level Value to be a table (Map). Stringifying
a primitive or array at the root surfaces a serializer error in the `err`
binding — by TOML spec, not a Liva limitation.

## Type-safe parsing

Same type-hint pattern as `JSON.parse`:

```liva
class Config { name: string; version: string }

let cfg: Config, err = TOML.parse("name = \"liva\"\nversion = \"2.1.0\"\n")
print(cfg.name)
```

See [json-basics.md](json-basics.md) for the full type-hint reference — every
pattern documented there applies to `TOML.parse` too.

## Errors

`TOML.parse` returns a non-empty `err` for malformed input. `TOML.stringify`
fails when the root Value isn't a table (see above).
