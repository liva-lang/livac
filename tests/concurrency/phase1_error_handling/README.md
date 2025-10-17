// Test: README para tests de Fase 1

# Fase 1: Error Handling Tests

## Tests OK (deben compilar y ejecutar)

### ok_error_binding_async.liva
- Error binding básico con `async`
- Chequeo de error con `if err`
- Acceso seguro después del chequeo

### ok_error_binding_par.liva
- Error binding con `par`
- Mismo patrón que async

### ok_error_ignore.liva
- Ignorar error explícitamente con `_`
- Debe compilar sin warnings
- Puede panic en runtime si falla

### ok_multiple_errors.liva
- Múltiples tasks con error binding
- Chequeo de cada error independientemente
- Paralelismo de tasks

## Tests ERR (deben fallar en compilación)

### err_wrong_binding_name.liva
- Segunda variable no se llama `err` o `_`
- Error: "Second binding must be named 'err' or '_'"

### err_no_result_type.liva
- Función no retorna `Result<T, E>`
- Error: "Function does not return Result type for error binding"

## Cómo ejecutar

```bash
# Ejecutar todos los tests de fase 1
cargo test --test concurrency_tests -- phase1

# Ejecutar test específico
cargo test --test concurrency_tests -- ok_error_binding_async

# Con output
cargo test --test concurrency_tests -- phase1 --nocapture
```

## Rust generado esperado

### Para ok_error_binding_async.liva:

```rust
let task = tokio::spawn(async move { get_user(0) });
let result = task.await;

let (user, err) = match result {
    Ok(Ok(u)) => (u, None),
    Ok(Err(e)) => (User::default(), Some(e)),
    Err(e) => (User::default(), Some(e.into())),
};

if err.is_some() {
    println!("Error: {}", err.unwrap());
    return;
}

println!("User: {}", user.name);
```

## Estado

- [ ] Parser implementado
- [ ] AST actualizado
- [ ] Semantic checking
- [ ] Codegen implementado
- [ ] Tests passing
