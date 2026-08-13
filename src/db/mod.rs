//! Módulo de Persistencia SQLite utilizando SQLx para Cacao Facturador e Inventario.

pub mod clientes;
pub mod facturas;
pub mod productos;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::str::FromStr;

pub async fn inicializar_db() -> Result<SqlitePool, Box<dyn std::error::Error>> {
    let db_dir = match std::env::var("HOME") {
        Ok(home) => std::path::PathBuf::from(home).join(".cacaofacturador"),
        Err(_) => std::env::current_dir()?.join("data"),
    };
    std::fs::create_dir_all(&db_dir)?;

    let target_db = db_dir.join("cacao_facturador.db");
    
    // Migrar base de datos si existe en directorio local
    let local_db = std::path::PathBuf::from("cacao_facturador.db");
    if local_db.exists() && !target_db.exists() {
        let _ = std::fs::copy(&local_db, &target_db);
    }

    let db_url = format!("sqlite:{}?mode=rwc", target_db.to_string_lossy());
    let options = SqliteConnectOptions::from_str(&db_url)?
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect_with(options)
        .await?;

    // Crear Tablas si no existen
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS productos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            codigo TEXT UNIQUE NOT NULL,
            codigo_auxiliar TEXT,
            descripcion TEXT NOT NULL,
            precio_unitario REAL NOT NULL,
            stock REAL NOT NULL DEFAULT 0.0,
            codigo_iva TEXT NOT NULL DEFAULT '4',
            tarifa_iva REAL NOT NULL DEFAULT 15.0,
            tipo TEXT NOT NULL DEFAULT 'PRODUCTO',
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS clientes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            tipo_identificacion TEXT NOT NULL,
            identificacion TEXT UNIQUE NOT NULL,
            razon_social TEXT NOT NULL,
            direccion TEXT,
            email TEXT,
            telefono TEXT,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS facturas (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            clave_acceso TEXT UNIQUE NOT NULL,
            secuencial TEXT NOT NULL,
            fecha_emision TEXT NOT NULL,
            cliente_identificacion TEXT NOT NULL,
            cliente_razon TEXT NOT NULL,
            total_sin_impuestos REAL NOT NULL,
            total_iva REAL NOT NULL,
            importe_total REAL NOT NULL,
            estado TEXT NOT NULL,
            xml_firmado TEXT,
            xml_autorizado TEXT,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        "#,
    )
    .execute(&pool)
    .await?;

    // Migración silenciosa para añadir columna tipo si no existe
    let _ = sqlx::query("ALTER TABLE productos ADD COLUMN tipo TEXT NOT NULL DEFAULT 'PRODUCTO'")
        .execute(&pool)
        .await;

    // Insertar productos iniciales de ejemplo si la tabla está vacía
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM productos")
        .fetch_one(&pool)
        .await?;

    if count.0 == 0 {
        sqlx::query(
            r#"
            INSERT INTO productos (codigo, descripcion, precio_unitario, stock, codigo_iva, tarifa_iva)
            VALUES 
            ('PRD-001', 'Servicio de Desarrollo / Consultoría', 150.00, 100.0, '4', 15.0),
            ('PRD-002', 'Cacao en Grano 1kg (Ecuador Extra)', 12.50, 50.0, '0', 0.0),
            ('PRD-003', 'Licor de Cacao 500g', 25.00, 30.0, '4', 15.0);
            "#
        )
        .execute(&pool)
        .await?;
    }

    Ok(pool)
}
