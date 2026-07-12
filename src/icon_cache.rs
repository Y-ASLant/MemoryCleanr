use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result};
use windows::Win32::Storage::FileSystem::{
    FILE_ATTRIBUTE_HIDDEN, FILE_ATTRIBUTE_READONLY, FILE_ATTRIBUTE_SYSTEM,
    FILE_FLAGS_AND_ATTRIBUTES, GetFileAttributesW, INVALID_FILE_ATTRIBUTES, SetFileAttributesW,
};
use windows::Win32::System::Registry::{
    HKEY, HKEY_CLASSES_ROOT, KEY_SET_VALUE, RegCloseKey, RegDeleteValueW, RegOpenKeyExW,
};
use windows::core::PCWSTR;

const EXPLORER_WAIT: Duration = Duration::from_millis(800);

const THUMBCACHE_FILES: &[&str] = &[
    "thumbcache_32.db",
    "thumbcache_96.db",
    "thumbcache_102.db",
    "thumbcache_256.db",
    "thumbcache_1024.db",
    "thumbcache_idx.db",
    "thumbcache_sr.db",
];

const TRAY_NOTIFY_SUBKEY: &str =
    "Local Settings\\Software\\Microsoft\\Windows\\CurrentVersion\\TrayNotify";

const TRAY_NOTIFY_VALUES: &[&str] = &["IconStreams", "PastIconsStream"];

/// Refresh the desktop icon cache by restarting Explorer and clearing cached files.
pub fn refresh() -> Result<()> {
    crate::log::write("[icon_cache] 开始刷新桌面图标缓存");

    crate::win32::process::kill_process_by_name("explorer.exe")?;
    std::thread::sleep(EXPLORER_WAIT);

    let local_app_data = local_app_data_dir()?;

    let icon_cache = local_app_data.join("IconCache.db");
    clear_attributes(&icon_cache)?;
    delete_file_best_effort(&icon_cache);

    let explorer_dir = local_app_data
        .join("Microsoft")
        .join("Windows")
        .join("Explorer");
    clear_dir_attributes(&explorer_dir)?;

    for name in THUMBCACHE_FILES {
        let path = explorer_dir.join(name);
        clear_attributes(&path)?;
        delete_file_best_effort(&path);
    }

    for value in TRAY_NOTIFY_VALUES {
        delete_registry_value(TRAY_NOTIFY_SUBKEY, value);
    }

    restart_explorer()?;
    crate::log::write("[icon_cache] 桌面图标缓存刷新完成");
    Ok(())
}

fn local_app_data_dir() -> Result<PathBuf> {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .context("LOCALAPPDATA is not set")
}

fn to_wide(value: &str) -> Vec<u16> {
    OsStr::new(value).encode_wide().chain(Some(0)).collect()
}

fn clear_attributes(path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    let wide = to_wide(&path.to_string_lossy());
    unsafe {
        let attrs = GetFileAttributesW(PCWSTR(wide.as_ptr()));
        if attrs == INVALID_FILE_ATTRIBUTES {
            return Ok(());
        }
        let mask = FILE_ATTRIBUTE_HIDDEN.0 | FILE_ATTRIBUTE_SYSTEM.0 | FILE_ATTRIBUTE_READONLY.0;
        let new_attrs = attrs & !mask;
        let _ = SetFileAttributesW(PCWSTR(wide.as_ptr()), FILE_FLAGS_AND_ATTRIBUTES(new_attrs));
    }
    Ok(())
}

fn clear_dir_attributes(dir: &Path) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }

    for entry in std::fs::read_dir(dir)
        .with_context(|| format!("read_dir {}", dir.display()))?
        .flatten()
    {
        let path = entry.path();
        clear_attributes(&path)?;
        if path.is_dir() {
            clear_dir_attributes(&path)?;
        }
    }
    Ok(())
}

fn delete_file_best_effort(path: &Path) {
    if !path.is_file() {
        return;
    }
    if let Err(e) = std::fs::remove_file(path) {
        crate::log::write(&format!("[icon_cache] 删除 {} 失败: {e}", path.display()));
    }
}

fn delete_registry_value(subkey: &str, value: &str) {
    unsafe {
        let subkey_wide = to_wide(subkey);
        let mut key = HKEY::default();
        if RegOpenKeyExW(
            HKEY_CLASSES_ROOT,
            PCWSTR(subkey_wide.as_ptr()),
            Some(0),
            KEY_SET_VALUE,
            &mut key,
        )
        .is_err()
        {
            return;
        }

        let value_wide = to_wide(value);
        let err = RegDeleteValueW(key, PCWSTR(value_wide.as_ptr()));
        if err.is_err() {
            crate::log::write(&format!(
                "[icon_cache] 删除注册表值 {value} 失败: {:?}",
                err
            ));
        }
        let _ = RegCloseKey(key);
    }
}

fn restart_explorer() -> Result<()> {
    std::process::Command::new("explorer.exe")
        .spawn()
        .context("failed to restart explorer.exe")?;
    Ok(())
}
