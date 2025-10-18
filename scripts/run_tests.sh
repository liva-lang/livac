#!/bin/bash

# Script para ejecutar todos los tests de livac
set -e

echo "🧪 Ejecutando tests de livac..."

# Crear directorio de snapshots si no existe
mkdir -p tests/snapshots

echo "📋 Ejecutando tests unitarios..."
cargo test --lib

echo "🔍 Ejecutando tests de lexer..."
cargo test --test lexer_tests

echo "🌳 Ejecutando tests de parser..."
cargo test --test parser_tests

echo "🧠 Ejecutando tests de semántica..."
cargo test --test semantics_tests

echo "🔄 Ejecutando tests de desugaring..."
cargo test --test desugar_tests

echo "🔗 Ejecutando tests de integración..."
cargo test --test integration_tests

echo "🎲 Ejecutando property tests..."
cargo test --test property_tests

echo "✅ Todos los tests completados exitosamente!"

# Opcional: ejecutar con coverage
if command -v cargo-tarpaulin &> /dev/null; then
    echo "📊 Generando reporte de cobertura..."
    cargo tarpaulin --out Html --output-dir coverage
    echo "📊 Reporte de cobertura generado en coverage/tarpaulin-report.html"
fi
