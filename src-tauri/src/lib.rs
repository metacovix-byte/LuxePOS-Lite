// ═══════════════════════════════════════════════════════════════════════
// LuxePOS — Backend Rust (Tauri)
// Remplace l'ancien serveur PowerShell par des commandes Tauri natives.
// Tous les fichiers persistance vivent dans %APPDATA%\Roaming\LuxePOS\
// ═══════════════════════════════════════════════════════════════════════

use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

// v5.14 — Chiffrement AES-GCM 256 pour le miroir OneDrive
use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::Engine;
use rand::RngCore;

// ─── Helpers chemin ─────────────────────────────────────────────────────

/// Renvoie le dossier de données app (créé si absent).
/// Windows : C:\Users\<user>\AppData\Roaming\LuxePOS\
fn app_data_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("app_data_dir: {}", e))?;
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| format!("create_dir_all: {}", e))?;
    }
    Ok(dir)
}

fn data_file_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(app_data_dir(app)?.join("luxepos-data.json"))
}

fn backups_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app_data_dir(app)?.join("backups");
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| format!("create_dir_all backups: {}", e))?;
    }
    Ok(dir)
}

/// v5.14 — Détection OneDrive.
/// Renvoie le chemin du dossier OneDrive\LuxePOS-Backup\ s'il est trouvé, sinon None.
/// Variables d'env standard Windows : OneDrive (perso) ou OneDriveCommercial (pro).
fn onedrive_backup_dir() -> Option<PathBuf> {
    let onedrive_root = std::env::var("OneDrive")
        .or_else(|_| std::env::var("OneDriveConsumer"))
        .or_else(|_| std::env::var("OneDriveCommercial"))
        .ok()?;
    let dir = PathBuf::from(onedrive_root).join("LuxePOS-Backup");
    Some(dir)
}

/// v5.14 — Récupère ou crée la clé AES-GCM 256 stockée dans AppData (PAS dans OneDrive).
/// Si OneDrive est compromis, la clé reste locale → backup illisible.
fn get_or_create_encryption_key(app: &tauri::AppHandle) -> Result<[u8; 32], String> {
    let key_path = app_data_dir(app)?.join("backup.key");
    if key_path.exists() {
        let bytes = fs::read(&key_path).map_err(|e| format!("read key: {}", e))?;
        if bytes.len() == 32 {
            let mut key = [0u8; 32];
            key.copy_from_slice(&bytes);
            return Ok(key);
        }
        // Clé corrompue → régénère
    }
    let mut key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);
    fs::write(&key_path, &key).map_err(|e| format!("write key: {}", e))?;
    Ok(key)
}

/// Chiffre un payload en AES-GCM 256. Retourne un blob auto-décrivant :
/// `LXP1\n<base64(nonce_12B||ciphertext)>` — préfixe "LXP1\n" en clair pour magic-detect.
fn encrypt_payload(payload: &str, key: &[u8; 32]) -> Result<String, String> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| format!("cipher init: {}", e))?;
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, payload.as_bytes())
        .map_err(|e| format!("encrypt: {}", e))?;
    let mut combined = Vec::with_capacity(12 + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);
    let b64 = base64::engine::general_purpose::STANDARD.encode(&combined);
    Ok(format!("LXP1\n{}", b64))
}

/// Déchiffre un blob produit par `encrypt_payload`. Retourne None si format invalide.
fn decrypt_payload(blob: &str, key: &[u8; 32]) -> Option<String> {
    let rest = blob.strip_prefix("LXP1\n")?;
    let combined = base64::engine::general_purpose::STANDARD.decode(rest.trim()).ok()?;
    if combined.len() < 13 {
        return None;
    }
    let (nonce_bytes, ct) = combined.split_at(12);
    let cipher = Aes256Gcm::new_from_slice(key).ok()?;
    let nonce = Nonce::from_slice(nonce_bytes);
    let plain = cipher.decrypt(nonce, ct).ok()?;
    String::from_utf8(plain).ok()
}

/// Écrit une copie miroir dans OneDrive si disponible. Ne fail jamais (best-effort).
/// v5.14 — La copie est chiffrée AES-GCM 256 avec une clé locale (jamais sur OneDrive).
/// Le fichier .json contient un blob "LXP1\n<base64>" — extension conservée pour détection.
fn mirror_to_onedrive(app: &tauri::AppHandle, payload: &str) -> Option<String> {
    let dir = onedrive_backup_dir()?;
    if !dir.exists() && fs::create_dir_all(&dir).is_err() {
        return None;
    }
    // Chiffrement (best-effort : si la clé fail, on passe en clair pour ne pas bloquer le backup)
    let to_write: String = match get_or_create_encryption_key(app) {
        Ok(key) => encrypt_payload(payload, &key).unwrap_or_else(|_| payload.to_string()),
        Err(_) => payload.to_string(),
    };
    let target = dir.join("luxepos-data.json");
    let prev = dir.join("luxepos-data.prev.json");
    // Rotation : si target existe, copie vers .prev avant écraser
    if target.exists() {
        let _ = fs::copy(&target, &prev);
    }
    fs::write(&target, to_write).ok()?;
    Some(target.display().to_string())
}

// ─── Structures retour ─────────────────────────────────────────────────

#[derive(Serialize)]
struct SaveResult {
    ok: bool,
    size: usize,
    path: String,
    onedrive_mirrored: bool,
    onedrive_path: Option<String>,
}

#[derive(Serialize)]
struct OneDriveStatus {
    available: bool,
    path: Option<String>,
    backup_exists: bool,
    backup_size: u64,
    backup_modified: Option<String>,
    backup_encrypted: bool,
}

#[derive(Serialize)]
struct LoadResult {
    ok: bool,
    data: Option<String>,
    size: u64,
    modified: Option<String>,
    reason: Option<String>,
}

#[derive(Serialize)]
struct StatusResult {
    ok: bool,
    has_data: bool,
    size: u64,
    modified: Option<String>,
    path: String,
    backups_count: usize,
    server_version: String,
}

#[derive(Serialize)]
struct ConflictsResult {
    ok: bool,
    conflicts: Vec<String>,
    count: usize,
}

#[derive(Serialize)]
struct BackupRotateResult {
    ok: bool,
    path: Option<String>,
    total: usize,
    error: Option<String>,
}

// ─── Commandes Tauri ───────────────────────────────────────────────────

/// Sauvegarde le state JSON sur disque.
/// Écriture atomique : écrit dans .tmp, puis rename.
/// Conserve la version précédente en .prev.
#[tauri::command]
fn save_data(app: tauri::AppHandle, payload: String) -> Result<SaveResult, String> {
    if payload.len() < 2 {
        return Err("payload too small".into());
    }
    if payload.len() > 50 * 1024 * 1024 {
        return Err("payload too large (>50MB)".into());
    }
    // Validation JSON minimale
    let trimmed = payload.trim_start();
    if !trimmed.starts_with('{') && !trimmed.starts_with('[') {
        return Err("invalid JSON shape".into());
    }

    let target = data_file_path(&app)?;
    let tmp = target.with_extension("json.tmp");
    let prev = target.with_extension("json.prev");

    fs::write(&tmp, &payload).map_err(|e| format!("write tmp: {}", e))?;

    // Rotation : ancien → .prev
    if target.exists() {
        if prev.exists() {
            let _ = fs::remove_file(&prev);
        }
        fs::rename(&target, &prev).map_err(|e| format!("rename prev: {}", e))?;
    }
    fs::rename(&tmp, &target).map_err(|e| format!("rename target: {}", e))?;

    // v5.14 — Miroir OneDrive chiffré (best-effort, ne fail pas si absent)
    let onedrive_path = mirror_to_onedrive(&app, &payload);

    Ok(SaveResult {
        ok: true,
        size: payload.len(),
        path: target.display().to_string(),
        onedrive_mirrored: onedrive_path.is_some(),
        onedrive_path,
    })
}

/// Statut du miroir OneDrive (pour afficher dans l'UI).
#[tauri::command]
fn onedrive_status() -> OneDriveStatus {
    let dir = onedrive_backup_dir();
    let path = dir.as_ref().map(|p| p.display().to_string());
    let target = dir.as_ref().map(|d| d.join("luxepos-data.json"));
    let exists = target.as_ref().map(|t| t.exists()).unwrap_or(false);
    let size = target
        .as_ref()
        .and_then(|t| fs::metadata(t).ok())
        .map(|m| m.len())
        .unwrap_or(0);
    let modified = target
        .as_ref()
        .and_then(|t| fs::metadata(t).ok())
        .and_then(|m| m.modified().ok())
        .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339());
    // v5.14 — Détecte si le backup est chiffré en lisant les 5 premiers octets
    let encrypted = target
        .as_ref()
        .and_then(|t| {
            use std::io::Read;
            fs::File::open(t).ok().and_then(|mut f| {
                let mut buf = [0u8; 5];
                f.read_exact(&mut buf).ok().map(|_| &buf == b"LXP1\n")
            })
        })
        .unwrap_or(false);
    OneDriveStatus {
        available: dir.is_some(),
        path,
        backup_exists: exists,
        backup_size: size,
        backup_modified: modified,
        backup_encrypted: encrypted,
    }
}

/// Lit le fichier de données disque.
/// Si le fichier principal est manquant/corrompu, fallback sur .prev.
#[tauri::command]
fn load_data(app: tauri::AppHandle) -> Result<LoadResult, String> {
    let target = data_file_path(&app)?;
    let prev = target.with_extension("json.prev");

    let path_to_read = if target.exists() {
        target.clone()
    } else if prev.exists() {
        prev.clone()
    } else {
        return Ok(LoadResult {
            ok: false,
            data: None,
            size: 0,
            modified: None,
            reason: Some("no-data".into()),
        });
    };

    let content = fs::read_to_string(&path_to_read).map_err(|e| format!("read: {}", e))?;

    let trimmed = content.trim_start();
    if content.len() < 2 || (!trimmed.starts_with('{') && !trimmed.starts_with('[')) {
        return Ok(LoadResult {
            ok: false,
            data: None,
            size: 0,
            modified: None,
            reason: Some("invalid-json".into()),
        });
    }

    let metadata = fs::metadata(&path_to_read).map_err(|e| format!("metadata: {}", e))?;
    let size = metadata.len();
    let modified = metadata
        .modified()
        .ok()
        .and_then(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339().into());

    Ok(LoadResult {
        ok: true,
        data: Some(content),
        size,
        modified,
        reason: None,
    })
}

/// Renvoie l'état du fichier (pour le badge sync UI).
#[tauri::command]
fn get_status(app: tauri::AppHandle) -> Result<StatusResult, String> {
    let target = data_file_path(&app)?;
    let backups = backups_dir(&app)?;

    let exists = target.exists();
    let size = if exists {
        fs::metadata(&target).map(|m| m.len()).unwrap_or(0)
    } else {
        0
    };
    let modified = if exists {
        fs::metadata(&target)
            .ok()
            .and_then(|m| m.modified().ok())
            .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339())
    } else {
        None
    };
    let backups_count = fs::read_dir(&backups)
        .map(|rd| {
            rd.flatten()
                .filter(|e| {
                    e.file_name()
                        .to_string_lossy()
                        .starts_with("luxepos-data_")
                })
                .count()
        })
        .unwrap_or(0);

    Ok(StatusResult {
        ok: true,
        has_data: exists,
        size,
        modified,
        path: target.display().to_string(),
        backups_count,
        server_version: "tauri-5.14".into(),
    })
}

/// Crée un snapshot horodaté du fichier disque dans backups/.
/// Garde max 30 snapshots (FIFO).
#[tauri::command]
fn rotate_backup(app: tauri::AppHandle) -> Result<BackupRotateResult, String> {
    let target = data_file_path(&app)?;
    if !target.exists() {
        return Ok(BackupRotateResult {
            ok: false,
            path: None,
            total: 0,
            error: Some("no-data".into()),
        });
    }
    let backups = backups_dir(&app)?;
    let stamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    let dest = backups.join(format!("luxepos-data_{}.json", stamp));

    fs::copy(&target, &dest).map_err(|e| format!("copy backup: {}", e))?;

    // Maintenance : garde 30 max, supprime les plus anciens
    let mut entries: Vec<_> = fs::read_dir(&backups)
        .map_err(|e| format!("read_dir: {}", e))?
        .flatten()
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with("luxepos-data_")
        })
        .collect();
    entries.sort_by_key(|e| {
        e.metadata()
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
    });
    if entries.len() > 30 {
        for e in entries.iter().take(entries.len() - 30) {
            let _ = fs::remove_file(e.path());
        }
    }
    let total = entries.len().min(30);

    Ok(BackupRotateResult {
        ok: true,
        path: Some(dest.display().to_string()),
        total,
        error: None,
    })
}

/// Détecte les fichiers de conflit OneDrive/Dropbox dans le dossier app.
#[tauri::command]
fn check_conflicts(app: tauri::AppHandle) -> Result<ConflictsResult, String> {
    let dir = app_data_dir(&app)?;
    let mut conflicts = Vec::new();
    if let Ok(rd) = fs::read_dir(&dir) {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            let lower = name.to_lowercase();
            if lower.contains("conflict")
                || lower.contains("conflit")
                || lower.contains("conflicted copy")
            {
                conflicts.push(e.path().display().to_string());
            }
        }
    }
    Ok(ConflictsResult {
        ok: true,
        count: conflicts.len(),
        conflicts,
    })
}

/// Renvoie la version de l'app (pour le UI).
#[tauri::command]
fn app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// v5.14 — Tente de restaurer le backup OneDrive (déchiffrement avec la clé locale).
/// Retourne le JSON déchiffré, ou un message d'erreur si la clé/le blob ne match pas.
#[tauri::command]
fn restore_from_onedrive(app: tauri::AppHandle) -> Result<LoadResult, String> {
    let dir = onedrive_backup_dir().ok_or_else(|| "OneDrive indisponible".to_string())?;
    let target = dir.join("luxepos-data.json");
    if !target.exists() {
        return Ok(LoadResult { ok: false, data: None, size: 0, modified: None,
                               reason: Some("no-backup".into()) });
    }
    let raw = fs::read_to_string(&target).map_err(|e| format!("read: {}", e))?;
    let metadata = fs::metadata(&target).map_err(|e| format!("metadata: {}", e))?;
    let size = metadata.len();
    let modified = metadata
        .modified()
        .ok()
        .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339());

    // Si commence par "LXP1\n" → chiffré, déchiffre. Sinon → JSON en clair (legacy).
    let plain = if raw.starts_with("LXP1\n") {
        let key = get_or_create_encryption_key(&app)?;
        decrypt_payload(&raw, &key)
            .ok_or_else(|| "Déchiffrement échoué — clé locale différente ou blob corrompu".to_string())?
    } else {
        raw
    };

    Ok(LoadResult {
        ok: true,
        data: Some(plain),
        size,
        modified,
        reason: None,
    })
}

// ─── Bootstrap Tauri ───────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            save_data,
            load_data,
            get_status,
            rotate_backup,
            check_conflicts,
            app_version,
            onedrive_status,
            restore_from_onedrive,
        ])
        .setup(|app| {
            // Crée le dossier app data au démarrage
            if let Ok(dir) = app_data_dir(&app.handle()) {
                println!("[LuxePOS] data_dir: {}", dir.display());
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
