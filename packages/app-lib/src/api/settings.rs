//! Theseus settings management interface

pub use crate::{
    State,
    state::{Hooks, MemorySettings, Settings, WindowSize},
};

/// Gets entire settings
#[tracing::instrument]
pub async fn get() -> crate::Result<Settings> {
    let state = State::get().await?;
    let settings = Settings::get(&state.pool).await?;
    Ok(settings)
}

/// Sets entire settings
#[tracing::instrument]
pub async fn set(mut settings: Settings) -> crate::Result<()> {
    let state = State::get().await?;
    let current = Settings::get(&state.pool).await?;

    // Always preserve custom_dir and prev_custom_dir during general settings updates
    // to prevent other settings tabs from overwriting directory configuration.
    settings.custom_dir = current.custom_dir;
    settings.prev_custom_dir = current.prev_custom_dir;

    settings.update(&state.pool).await?;

    Ok(())
}

/// Changes the launcher directory safely
#[tracing::instrument]
pub async fn set_launcher_directory(
    custom_dir: Option<String>,
) -> crate::Result<()> {
    let state = State::get().await?;
    let mut current = Settings::get(&state.pool).await?;

    let default_dir = crate::state::DirectoryInfo::initial_settings_dir_path(&state.directories.app_identifier)
        .map(|p| p.to_string_lossy().to_string());
    let old_dir = current.custom_dir.clone().or(default_dir);

    if custom_dir == current.custom_dir {
        return Ok(());
    }

    if let Some(ref dir_str) = custom_dir {
        let p = std::path::Path::new(dir_str);
        tokio::fs::create_dir_all(p).await.map_err(|e| {
            crate::ErrorKind::FSError(format!("Failed to create directory {dir_str}: {e}"))
        })?;
    }

    current.prev_custom_dir = old_dir;
    current.custom_dir = custom_dir;
    current.update(&state.pool).await?;

    Ok(())
}

#[tracing::instrument]
pub async fn cancel_directory_change(
    app_identifier: &str,
) -> crate::Result<()> {
    // This is called to handle state initialization errors due to folder migrations
    // failing, so fetching a DB connection pool from `State::get` is not reliable here
    let pool = crate::state::db::connect(app_identifier).await?;
    let mut settings = Settings::get(&pool).await?;

    if let Some(prev_custom_dir) = settings.prev_custom_dir {
        let cancelled_new_dir = settings.custom_dir.clone();
        settings.custom_dir = Some(prev_custom_dir.clone());
        settings.prev_custom_dir = None;

        // Also revert any java_version paths that were updated in the DB
        if let Some(new_dir) = cancelled_new_dir {
            let new_dir_clean = new_dir.trim_end_matches('/').trim_end_matches('\\');
            let prev_dir_clean = prev_custom_dir.trim_end_matches('/').trim_end_matches('\\');
            if let Ok(java_versions) = crate::state::JavaVersion::get_all(&pool).await {
                for (_, mut jv) in java_versions {
                    if jv.path.starts_with(new_dir_clean) {
                        jv.path = jv.path.replace(new_dir_clean, prev_dir_clean);
                        let _ = jv.upsert(&pool).await;
                    }
                }
            }
        }
    }

    settings.update(&pool).await?;

    Ok(())
}
