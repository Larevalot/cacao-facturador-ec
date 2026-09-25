# 🍫 Cacao Facturador SRI & Gestor de Inventario 🇪🇨

[![License: MIT](https://img.shields.io/badge/License-MIT-amber.svg)](https://opensource.org/licenses/MIT)
[![Version](https://img.shields.io/badge/version-1.0.0-chocolate.svg)](https://github.com/Larevalot/cacao-facturador-ec/releases)
[![Rust](https://img.shields.io/badge/Rust-2024-orange.svg)](https://www.rust-lang.org/)
[![GUI](https://img.shields.io/badge/GUI-Iced-blue.svg)](https://iced.rs/)
[![Database](https://img.shields.io/badge/Database-SQLite-lightgrey.svg)](https://www.sqlite.org/)

Sistema **Open Source** de **Facturación Electrónica para el SRI de Ecuador** y **Gestión de Inventario & Servicios**, construido como una aplicación de escritorio nativa y ligera en **Rust**, **Iced GUI**, **SQLite (SQLx)** y **printpdf**.

Desarrollado por [cacaoscript.com](https://cacaoscript.com) para ser **100% gratuito, privado, sin dependencias web pesadas y libre**.

---

## ⚡ Características Principales

- **Cumplimiento Normativo SRI Ecuador (Esquema Offline v2.1/v2.34)**:
  - Generación de estructura XML oficial de factura electrónica.
  - Firma Electrónica nativa en formato **XAdES-BES** (archivos `.p12` / `.pfx`).
  - Validación de Módulo 11 para la Clave de Acceso (49 dígitos).
  - Envío y recepción asíncrona ante los WebServices del SRI (Ambientes de Pruebas y Producción).
  - Polleo dinámico para respuestas `RECIBIDA`, `EN PROCESO`, `AUTORIZADO` y `RECHAZADO`.
- **Plantillas PDF RIDE Profesionales**:
  - Tres modelos de comprobante: **Clásica (Oficial SRI)**, **Moderna (Cacao)** y **Compacta (Ahorro Tinta)**.
  - Soporte de subida y escalado proporcional de logotipo corporativo (PNG / JPG).
  - Código de barras Code 128 nítido de la clave de acceso.
  - Exportación individual o conjunta de archivo `.pdf` y `.xml` firmado.
- **Gestión de Inventario y Catálogo**:
  - Distinción entre **📦 Productos** (con control de existencias) y **🛠️ Servicios** (sin stock).
  - Cálculo bidireccional en tiempo real entre Precio Unitario (sin impuestos) y Total (con IVA).
  - Tarifas de IVA vigentes (15% y 0%).
- **Clientes y Facturación Rápida**:
  - Búsqueda y autocompletado automático de clientes frecuentes por RUC o Cédula.
  - Soporte nativo para Consumidor Final con verificación preventiva del límite legal ($50.00 USD).
- **Interfaz Nativa y Seguridad**:
  - Construida con **Iced GUI**: interfaz fluida, bajo consumo de memoria RAM y arranque instantáneo (sin Chromium ni WebViews).
  - Bloqueo y protección mediante PIN de 4 dígitos (con teclado táctil y teclado físico).
  - Notificaciones emergentes con descarte automático tras 6 segundos.
- **Privacidad Local-First**:
  - Todos los certificados, productos, clientes y facturas permanecen exclusivamente en tu computadora dentro de SQLite local (`~/.cacaofacturador/cacao_facturador.db`).

---

## 🚀 Requisitos de Sistema

- **Rust**: `>= 1.85.0` (Edición 2024)
- **OpenSSL 3.0** (librerías de desarrollo en Linux)

### En Linux (Ubuntu / Debian):
```bash
sudo apt-get update
sudo apt-get install -y libssl-dev pkg-config
```

---

## 🛠️ Comandos de Desarrollo

```bash
# 1. Clonar el repositorio
git clone https://github.com/Larevalot/cacao-facturador-ec.git
cd cacao-facturador-ec

# 2. Ejecutar la aplicación en modo desarrollo
cargo run

# 3. Ejecutar la suite completa de pruebas automatizadas
cargo test
```

---

## 📦 Compilación para Distribución

Para generar el ejecutable binario nativo optimizado para tu sistema operativo:

```bash
cargo build --release
```

El binario compilado se ubicará en `target/release/cacaofacturador-ec` (en Linux) o `target/release/cacaofacturador-ec.exe` (en Windows).

---

## 📜 Licencia

Este proyecto está bajo la Licencia **MIT**. Puedes usarlo, modificarlo, redistribuirlo y adaptarlo libremente tanto para fines personales como comerciales.

```text
MIT License - Copyright (c) 2026 CacaoScript / Cacao Apps
```

---

## 🌐 Enlaces Oficiales

- **Web Oficial**: [cacaoscript.com](https://cacaoscript.com)
