use std::path::PathBuf;

use sqlx::Row;

use crate::{
    State,
    install::{InstallPhaseDetails, InstallProgressReporter},
    prelude::ModLoader,
    state::{AppliedContentSetPatch, EditInstance, InstanceInstallStage},
};

use super::{finish_import, recache_icon};

pub async fn is_valid_modrinth(instance_folder: PathBuf) -> bool {
    if !instance_folder.is_dir() {
        return false;
    }

    let indicators = [
        ".fabric",
        ".forge",
        ".neoforge",
        ".quilt",
        "mods",
        "saves",
        "options.txt",
        "servers.dat",
        "debug-profile.json",
        "config",
    ];

    for indicator in &indicators {
        if instance_folder.join(indicator).exists() {
            return true;
        }
    }

    false
}

pub async fn import_modrinth(
    base_path: PathBuf,
    instance_folder: String,
    instance_id: &str,
    reporter: InstallProgressReporter,
    details: InstallPhaseDetails,
) -> crate::Result<()> {
    let modrinth_instance_path =
        if base_path.join("profiles").join(&instance_folder).exists() {
            base_path.join("profiles").join(&instance_folder)
        } else if base_path.join(&instance_folder).exists() {
            base_path.join(&instance_folder)
        } else {
            base_path.join("profiles").join(&instance_folder)
        };

    // Locate Modrinth's app.db
    let db_path = if base_path.join("app.db").exists() {
        base_path.join("app.db")
    } else if let Some(parent) = base_path.parent() {
        if parent.join("app.db").exists() {
            parent.join("app.db")
        } else {
            base_path.join("app.db")
        }
    } else {
        base_path.join("app.db")
    };

    let mut friendly_name: Option<String> = None;
    let mut db_icon_path: Option<String> = None;
    let mut game_version: Option<String> = None;
    let mut loader_type_str: Option<String> = None;
    let mut loader_version_str: Option<String> = None;

    if db_path.exists() {
        let conn_opts = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(&db_path)
            .read_only(true)
            .create_if_missing(false);

        if let Ok(pool) = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(conn_opts)
            .await
        {
            let query = "
                SELECT i.name, i.icon_path, ics.game_version, ics.loader, ics.loader_version
                FROM instances i
                LEFT JOIN instance_content_sets ics ON (
                    ics.id = i.applied_content_set_id OR ics.instance_id = i.id
                )
                WHERE i.path = ? OR i.path LIKE ? OR i.id = ? OR i.name = ?
                ORDER BY (ics.id = i.applied_content_set_id) DESC
                LIMIT 1
            ";
            let pattern = format!("%{}", instance_folder);
            if let Ok(Some(row)) = sqlx::query(query)
                .bind(&instance_folder)
                .bind(&pattern)
                .bind(&instance_folder)
                .bind(&instance_folder)
                .fetch_optional(&pool)
                .await
            {
                friendly_name = row.try_get::<String, _>("name").ok();
                db_icon_path = row.try_get::<String, _>("icon_path").ok();
                game_version = row.try_get::<String, _>("game_version").ok();
                loader_type_str = row.try_get::<String, _>("loader").ok();
                loader_version_str =
                    row.try_get::<String, _>("loader_version").ok();
            }
        }
    }

    // Determine ModLoader
    let mod_loader = match loader_type_str
        .as_deref()
        .unwrap_or("")
        .trim()
        .to_lowercase()
        .as_str()
    {
        "fabric" => ModLoader::Fabric,
        "forge" => ModLoader::Forge,
        "neoforge" => ModLoader::NeoForge,
        "quilt" => ModLoader::Quilt,
        _ => {
            if modrinth_instance_path.join(".fabric").exists() {
                ModLoader::Fabric
            } else if modrinth_instance_path.join(".neoforge").exists() {
                ModLoader::NeoForge
            } else if modrinth_instance_path.join(".forge").exists() {
                ModLoader::Forge
            } else if modrinth_instance_path.join(".quilt").exists() {
                ModLoader::Quilt
            } else {
                ModLoader::Vanilla
            }
        }
    };

    // Determine Game Version
    let game_version = game_version.unwrap_or_else(|| "1.20.1".to_string());

    let loader_version = if mod_loader != ModLoader::Vanilla {
        crate::launcher::get_loader_version_from_profile(
            &game_version,
            mod_loader,
            loader_version_str.as_deref(),
        )
        .await?
    } else {
        None
    };

    // Determine icon
    let mut icon = None;
    if let Some(db_icon) = db_icon_path {
        let p = PathBuf::from(db_icon);
        if p.exists() {
            icon = recache_icon(p).await.ok().flatten();
        }
    }
    if icon.is_none() {
        let local_icon = modrinth_instance_path.join("icon.png");
        if local_icon.exists() {
            icon = recache_icon(local_icon).await.ok().flatten();
        }
    }

    let backup_name = format!("Modrinth-{}", instance_folder);
    let name = friendly_name.unwrap_or(backup_name);

    crate::api::instance::edit(
        instance_id,
        EditInstance {
            install_stage: Some(InstanceInstallStage::PackInstalling),
            name: Some(name),
            icon_path: Some(icon.map(|x| x.to_string_lossy().to_string())),
            content_set_patch: Some(AppliedContentSetPatch {
                source_kind: None,
                game_version: Some(game_version),
                protocol_version: Some(None),
                loader: Some(mod_loader),
                loader_version: Some(loader_version.map(|x| x.id)),
            }),
            ..EditInstance::default()
        },
    )
    .await?;

    let state = State::get().await?;
    finish_import(
        instance_id,
        modrinth_instance_path,
        &state.io_semaphore,
        reporter,
        details,
    )
    .await?;

    Ok(())
}
