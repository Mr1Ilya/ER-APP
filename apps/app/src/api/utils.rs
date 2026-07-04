use serde::{Deserialize, Serialize};
use tauri::Runtime;
use tauri_plugin_opener::OpenerExt;
use theseus::{
    handler,
    prelude::{CommandPayload, DirectoryInfo, app_db_backup_dir},
};

use crate::api::{Result, TheseusSerializableError};
use dashmap::DashMap;
use std::path::{Path, PathBuf};
use theseus::prelude::canonicalize;
use url::Url;

pub fn init<R: Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("utils")
        .invoke_handler(tauri::generate_handler![
            get_os,
            is_network_metered,
            should_disable_mouseover,
            highlight_in_folder,
            open_path,
            show_launcher_logs_folder,
            show_app_db_backups_folder,
            progress_bars_list,
            get_opening_command,
            get_telegram_news
        ])
        .build()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::enum_variant_names)]
pub enum OS {
    Windows,
    Linux,
    MacOS,
}

/// Gets OS
#[tauri::command]
pub fn get_os() -> OS {
    #[cfg(target_os = "windows")]
    let os = OS::Windows;
    #[cfg(target_os = "linux")]
    let os = OS::Linux;
    #[cfg(target_os = "macos")]
    let os = OS::MacOS;
    os
}

#[tauri::command]
pub async fn is_network_metered() -> Result<bool> {
    Ok(theseus::prelude::is_network_metered().await?)
}

// Lists active progress bars
// Create a new HashMap with the same keys
// Values provided should not be used directly, as they are not guaranteed to be up-to-date
#[tauri::command]
pub async fn progress_bars_list()
-> Result<DashMap<uuid::Uuid, theseus::LoadingBar>> {
    let res = theseus::EventState::list_progress_bars().await?;
    Ok(res)
}

// disables mouseover and fixes a random crash error only fixed by recent versions of macos
#[tauri::command]
pub async fn should_disable_mouseover() -> bool {
    if cfg!(target_os = "macos") {
        // We try to match version to 12.2 or higher. If unrecognizable to pattern or lower, we default to the css with disabled mouseover for safety
        if let tauri_plugin_os::Version::Semantic(major, minor, _) =
            tauri_plugin_os::version()
            && major >= 12
            && minor >= 3
        {
            // Mac os version is 12.3 or higher, we allow mouseover
            return false;
        }
        true
    } else {
        // Not macos, we allow mouseover
        false
    }
}

#[tauri::command]
pub async fn highlight_in_folder<R: Runtime>(
    app: tauri::AppHandle<R>,
    path: PathBuf,
) {
    tauri::async_runtime::spawn_blocking(move || {
        if let Err(e) = app.opener().reveal_item_in_dir(path) {
            tracing::error!("Failed to highlight file in folder: {}", e);
        }
    })
    .await
    .ok();
}

#[tauri::command]
pub async fn open_path<R: Runtime>(app: tauri::AppHandle<R>, path: PathBuf) {
    tauri::async_runtime::spawn_blocking(move || {
        if let Err(e) =
            app.opener().open_path(path.to_string_lossy(), None::<&str>)
        {
            tracing::error!("Failed to open path: {}", e);
        }
    })
    .await
    .ok();
}

#[tauri::command]
pub async fn show_launcher_logs_folder<R: Runtime>(app: tauri::AppHandle<R>) {
    if let Some(d) = DirectoryInfo::global_handle_if_ready() {
        let path = d.launcher_logs_dir().unwrap_or_default();
        // failure to get folder just opens filesystem
        // (ie: if in debug mode only and launcher_logs never created)
        open_path(app, path).await;
    }
}

#[tauri::command]
pub async fn show_app_db_backups_folder<R: Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<()> {
    let path = app_db_backup_dir()?;
    tokio::fs::create_dir_all(&path).await?;
    open_path(app, path).await;
    Ok(())
}

// Get opening command
// For example, if a user clicks on an .mrpack to open the app.
// This should be called once and only when the app is done booting up and ready to receive a command
// Returns a Command struct- see events.js
#[tauri::command]
#[cfg(target_os = "macos")]
pub async fn get_opening_command(
    state: tauri::State<'_, crate::macos::deep_link::InitialPayload>,
) -> Result<Option<CommandPayload>> {
    let payload = state.payload.lock().await;
    let cmd_arg = std::env::args_os()
        .nth(1)
        .map(|path| path.to_string_lossy().to_string());

    return if let Some(payload) = payload.as_ref() {
        tracing::info!("opening command {payload}");

        Ok(Some(handler::parse_command(payload).await?))
    } else if let Some(cmd_arg) = cmd_arg {
        tracing::info!("opening command {cmd_arg:?}");

        Ok(Some(handler::parse_command(&cmd_arg).await?))
    } else {
        Ok(None)
    };
}

#[tauri::command]
#[cfg(not(target_os = "macos"))]
pub async fn get_opening_command() -> Result<Option<CommandPayload>> {
    // Tauri is not CLI, we use arguments as path to file to call
    let cmd_arg = std::env::args_os().nth(1);

    tracing::info!("opening command {cmd_arg:?}");

    let cmd_arg = cmd_arg.map(|path| path.to_string_lossy().to_string());
    if let Some(cmd) = cmd_arg {
        tracing::debug!("Opening command: {:?}", cmd);
        return Ok(Some(handler::parse_command(&cmd).await?));
    }
    Ok(None)
}

// helper function called when redirected by a weblink (ie: modrith://do-something) or when redirected by a .mrpack file (in which case its a filepath)
// We hijack the deep link library (which also contains functionality for instance-checking)
pub async fn handle_command(command: String) -> Result<()> {
    tracing::info!("handle command: {command}");
    Ok(theseus::handler::parse_and_emit_command(&command).await?)
}

// Remove when (and if) https://github.com/tauri-apps/tauri/issues/12022 is implemented
pub(crate) fn tauri_convert_file_src(path: &Path) -> Result<Url> {
    #[cfg(any(windows, target_os = "android"))]
    const BASE: &str = "http://asset.localhost/";
    #[cfg(not(any(windows, target_os = "android")))]
    const BASE: &str = "asset://localhost/";

    macro_rules! theseus_try {
        ($test:expr) => {
            match $test {
                Ok(val) => val,
                Err(e) => {
                    return Err(TheseusSerializableError::Theseus(e.into()))
                }
            }
        };
    }

    let path = theseus_try!(canonicalize(path));
    let path = path.to_string_lossy();
    let encoded = urlencoding::encode(&path);

    Ok(theseus_try!(Url::parse(&format!("{BASE}{encoded}"))))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsItem {
    pub id: String,
    pub date: String,
    pub text: String,
}

#[tauri::command]
pub async fn get_telegram_news() -> Result<Vec<NewsItem>> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()
        .map_err(|e| theseus::ErrorKind::LauncherError(format!("Failed to build HTTP client: {e}")).as_error())?;

    // Try direct Telegram first
    let mut news = Vec::new();
    let mut success = false;

    tracing::info!("Attempting to fetch Telegram news directly from t.me...");
    match client.get("https://t.me/s/ERTeamAPP").send().await {
        Ok(resp) => {
            if resp.status().is_success() {
                match resp.text().await {
                    Ok(html) => {
                        let parts: Vec<&str> = html.split("<div class=\"tgme_widget_message_wrap js-widget_message_wrap\">").collect();
                        
                        let text_regex = regex::Regex::new(r#"(?s)<div class="tgme_widget_message_text js-message_text" dir="auto">(.*?)</div>"#);
                        let date_regex = regex::Regex::new(r#"<time datetime="([^"]+)" class="time">"#);
                        let id_regex = regex::Regex::new(r#"data-post="[^/]+/(\d+)""#);

                        match (text_regex, date_regex, id_regex) {
                            (Ok(text_regex), Ok(date_regex), Ok(id_regex)) => {
                                for part in parts.iter().skip(1) {
                                    if part.contains("service_message") {
                                        continue;
                                    }
                                    let id = id_regex.captures(part).map(|c| c[1].to_string()).unwrap_or_default();
                                    let date = date_regex.captures(part).map(|c| c[1].to_string()).unwrap_or_default();
                                    let text = text_regex.captures(part).map(|c| c[1].to_string()).unwrap_or_default();

                                    // Extract photos from background-image styles
                                    let mut photo_urls = Vec::new();
                                    if let Ok(photo_regex) = regex::Regex::new(r#"background-image:url\('([^']+)'\)"#) {
                                        for caps in photo_regex.captures_iter(part) {
                                            photo_urls.push(caps[1].to_string());
                                        }
                                    }

                                    if !text.is_empty() || !photo_urls.is_empty() {
                                        let mut formatted_text = text;
                                        for url in photo_urls {
                                            formatted_text.push_str(&format!(
                                                "<img src=\"{}\" class=\"news-image\" style=\"max-width: 100% !important; height: auto !important; display: block !important; border-radius: 6px; margin-top: 8px;\" />",
                                                url
                                            ));
                                        }
                                        news.push(NewsItem { id, date, text: formatted_text });
                                    }
                                }
                                if !news.is_empty() {
                                    success = true;
                                    tracing::info!("Successfully parsed {} news items directly from t.me", news.len());
                                } else {
                                    tracing::warn!("Parsed 0 news items directly from t.me html");
                                }
                            }
                            _ => {
                                tracing::error!("Regex compilation error");
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to read text from t.me response: {e}");
                    }
                }
            } else {
                tracing::warn!("Direct t.me request returned non-success status: {}", resp.status());
            }
        }
        Err(e) => {
            tracing::warn!("Direct t.me request failed: {e}");
        }
    }

    if !success {
        // Try the JSON proxy mirror
        tracing::info!("Direct fetch failed. Fetching Telegram news via JSON proxy (tg.i-c-a.su)...");
        match client.get("https://tg.i-c-a.su/json/ERTeamAPP?limit=20").send().await {
            Ok(response) => {
                if response.status().is_success() {
                    #[derive(Deserialize, Debug)]
                    struct TgProxyResponse {
                        messages: Option<Vec<TgProxyMessage>>,
                    }
                    #[derive(Deserialize, Debug)]
                    struct TgProxyMessage {
                        #[serde(rename = "_")]
                        msg_type: String,
                        id: i64,
                        date: i64,
                        message: Option<String>,
                        media: Option<serde_json::Value>,
                    }

                    match response.json::<TgProxyResponse>().await {
                        Ok(data) => {
                            if let Some(messages) = data.messages {
                                for msg in messages {
                                    if msg.msg_type == "message" {
                                        let text = msg.message.unwrap_or_default();
                                        let has_media = msg.media.is_some();

                                        if !text.trim().is_empty() || has_media {
                                            // Convert date timestamp to ISO 8601 string
                                            let dt = chrono::DateTime::from_timestamp(msg.date, 0)
                                                .map(|d| d.to_rfc3339())
                                                .unwrap_or_default();
                                                
                                            // Convert plain text newlines to <br/> to match HTML formatting in the frontend
                                            let mut formatted_text = text.replace('\n', "<br/>");

                                            if has_media {
                                                formatted_text.push_str(&format!(
                                                    "<img src=\"https://tg.i-c-a.su/media/ERTeamAPP/{}\" class=\"news-image\" style=\"max-width: 100% !important; height: auto !important; display: block !important; border-radius: 6px; margin-top: 8px;\" />",
                                                    msg.id
                                                ));
                                            }

                                            news.push(NewsItem {
                                                id: msg.id.to_string(),
                                                date: dt,
                                                text: formatted_text,
                                            });
                                        }
                                    }
                                }
                                if !news.is_empty() {
                                    success = true;
                                    tracing::info!("Successfully fetched {} news items from tg.i-c-a.su mirror", news.len());
                                } else {
                                    tracing::warn!("Fetched 0 news items from tg.i-c-a.su mirror JSON");
                                }
                            } else {
                                tracing::warn!("tg.i-c-a.su response JSON has no messages field");
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Failed to deserialize tg.i-c-a.su response JSON: {e}");
                        }
                    }
                } else {
                    tracing::warn!("tg.i-c-a.su request returned non-success status: {}", response.status());
                }
            }
            Err(e) => {
                tracing::warn!("tg.i-c-a.su request failed: {e}");
            }
        }
    }

    if !success && news.is_empty() {
        return Err(theseus::ErrorKind::LauncherError("Failed to load news from all mirrors.".to_string()).as_error().into());
    }

    // Reverse to show latest news first
    news.reverse();

    Ok(news)
}
