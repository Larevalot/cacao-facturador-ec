//! Manejo de Configuración persistente del Emisor en ~/.cacaofacturador/config.json.

use crate::sri::models::EmisorConfig;
use std::fs;
use std::path::PathBuf;

fn get_config_path() -> PathBuf {
    let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push(".cacaofacturador");
    let _ = fs::create_dir_all(&path);
    path.push("config.json");
    path
}

pub fn cargar_configuracion() -> EmisorConfig {
    let path = get_config_path();

    // Fallback: Si no existe en ~/.cacaofacturador/config.json pero existe en ./config.json, migrarlo
    if !path.exists() && std::path::Path::new("config.json").exists() {
        if let Ok(contenido) = fs::read_to_string("config.json") {
            if let Ok(config) = serde_json::from_str::<EmisorConfig>(&contenido) {
                let _ = guardar_configuracion(&config);
                let _ = fs::remove_file("config.json");
                return config;
            }
        }
    }

    if path.exists() {
        if let Ok(contenido) = fs::read_to_string(&path) {
            if let Ok(config) = serde_json::from_str::<EmisorConfig>(&contenido) {
                return config;
            }
        }
    }

    let default = EmisorConfig::default();
    let _ = guardar_configuracion(&default);
    default
}

pub fn guardar_configuracion(config: &EmisorConfig) -> Result<(), String> {
    let path = get_config_path();
    let json = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Error serializando configuración: {}", e))?;
    fs::write(&path, json)
        .map_err(|e| format!("Error guardando '{}': {}", path.display(), e))?;
    Ok(())
}
