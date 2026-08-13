# 🍫 Cacao Facturador SRI & Gestor de Inventario 🇪🇨

[![License: MIT](https://img.shields.io/badge/License-MIT-amber.svg)](https://opensource.org/licenses/MIT)
[![Version](https://img.shields.io/badge/version-1.0.0-chocolate.svg)](https://github.com/cacaoscript/cacaofacturador-ec/releases)
[![Rust](https://img.shields.io/badge/Rust-2024-orange.svg)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/Tauri-v2-blue.svg)](https://v2.tauri.app/)
[![Svelte](https://img.shields.io/badge/Svelte-v5-red.svg)](https://svelte.dev/)

Sistema **Open Source** de **Facturación Electrónica para el SRI de Ecuador** y **Gestión de Inventario & Servicios**, construido con **Rust + SQLite (SQLx) + Tauri v2 + Svelte 5**.

Desarrollado por [cacaoscript.com](https://cacaoscript.com) para ser **100% gratuito, privado y libre**.

---

## ⚡ Características Principales

- **Cumplimiento SRI Ecuador (Esquema Offline v2.1/v2.34)**:
  - Generación de estructura XML oficial.
  - Firma Electrónica nativa en formato **XAdES-BES** (archivos `.p12` / `.pfx`).
  - Validación de Módulo 11 para la Clave de Acceso (49 dígitos).
  - Envío y recepción asíncrona ante los WebServices del SRI (Pruebas y Producción).
  - Polleo dinámico para respuestas `EN PROCESO` / `RECIBIDA` / `AUTORIZADO`.
- **Soporte Completo para Consumidor Final**:
  - Autocompletado automático con datos oficiales del SRI (`07`, `9999999999999`, `CONSUMIDOR FINAL`).
  - Verificación preventiva del límite legal de $50.00 USD.
- **Gestión de Productos y Servicios**:
  - Distinción entre **📦 Productos** (con control de stock) y **🛠️ Servicios** (sin stock).
  - Cálculo bi-direccional en tiempo real entre **Precio Unitario (Sin IVA)** y **Precio Final (Con IVA)** a 2 decimales exactos.
  - Tarifas de IVA soportadas (15% y 0%).
- **Optimizado para Equipos de Bajos Recursos**:
  - Diseñado para máxima velocidad sin consumo excesivo de CPU/GPU en máquinas de punto de venta (POS) o PCs antiguas.
  - Interfaz de usuario limpia, reactiva e instantánea.
- **Privacidad Local First**:
  - Todos tus productos, clientes, historial de facturas y certificados permanecen en tu computadora alojados en una base de datos local SQLite (`~/.cacaofacturador/cacao_facturador.db`).

---

## 🚀 Requisitos de Desarrollo

- **Node.js**: `>= 20.0.0`
- **pnpm**: `>= 10.0.0`
- **Rust**: `>= 1.80.0` (Edición 2024)
- **OpenSSL 3.0** (librerías de desarrollo en Linux/macOS/Windows)

---

## 🛠️ Comandos de Instalación y Ejecución

```bash
# 1. Clonar el repositorio
git clone https://github.com/cacaoscript/cacaofacturador-ec.git
cd cacaofacturador-ec

# 2. Instalar dependencias del frontend
pnpm install

# 3. Ejecutar en modo Desarrollo Escritorio (Tauri v2)
pnpm desktop

# 4. Compilar assets del frontend
pnpm build:ui

# 5. Ejecutar Pruebas Unitarias del Backend (Rust)
pnpm test
```

---

## 📦 Compilación para Distribución

Para generar los ejecutables nativos redistribuibles:

```bash
# Compilar paquete nativo para tu sistema operativo
pnpm desktop:build
```

- **Windows**: Genera instalador `.exe` (NSIS) y `.msi` en `src-tauri/target/release/bundle/`.
- **Linux**: Genera paquetes `.AppImage`, `.deb` y `.tar.gz`.
- **macOS**: Genera instalador `.dmg` y `.app`.

---

## 📜 Licencia

Este proyecto está bajo la Licencia **MIT**. Puedes usarlo, modificarlo, redistribuirlo y adaptarlo libremente tanto para fines personales como comerciales.

```text
MIT License - Copyright (c) 2026 CacaoScript / Cacao Apps
```

---

## 🌐 Enlaces Oficiales

- **Web Oficial de Descargas**: [apps.cacaoscript.com](https://apps.cacaoscript.com)
- **Sitio Web Principal**: [cacaoscript.com](https://cacaoscript.com)
