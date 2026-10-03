use crate::api::Result;
use chrono::{Duration, Utc};
use tauri::plugin::TauriPlugin;
use tauri::{Manager, Runtime, UserAttentionType};
use theseus::prelude::*;

use tauri_plugin_opener::OpenerExt;

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::<R>::new("auth")
        .invoke_handler(tauri::generate_handler![
            check_reachable,
            login,
            login_offline,
            login_endrage,
            login_endrage_oauth,
            remove_user,
            get_default_user,
            set_default_user,
            get_users,
        ])
        .build()
}

/// Checks if the authentication servers are reachable.
#[tauri::command]
pub async fn check_reachable() -> Result<()> {
    minecraft_auth::check_reachable().await?;
    Ok(())
}

/// Authenticate a user with Hydra - part 1
/// This begins the authentication flow quasi-synchronously, returning a URL to visit (that the user will sign in at)
#[tauri::command]
pub async fn login<R: Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<Option<Credentials>> {
    let flow = minecraft_auth::begin_login().await?;

    let start = Utc::now();

    if let Some(window) = app.get_webview_window("signin") {
        window.close()?;
    }

    let window = tauri::WebviewWindowBuilder::new(
        &app,
        "signin",
        tauri::WebviewUrl::External(flow.auth_request_uri.parse().map_err(
            |_| {
                theseus::ErrorKind::OtherError(
                    "Error parsing auth redirect URL".to_string(),
                )
                .as_error()
            },
        )?),
    )
    .title("Sign into Microsoft")
    .always_on_top(true)
    .center()
    .build()?;

    window.request_user_attention(Some(UserAttentionType::Critical))?;

    while (Utc::now() - start) < Duration::minutes(10) {
        if window.title().is_err() {
            // user closed window, cancelling flow
            return Ok(None);
        }

        if window
            .url()?
            .as_str()
            .starts_with("https://login.live.com/oauth20_desktop.srf")
            && let Some((_, code)) =
                window.url()?.query_pairs().find(|x| x.0 == "code")
        {
            window.close()?;
            let val = minecraft_auth::finish_login(&code.clone(), flow).await?;

            return Ok(Some(val));
        }

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }

    window.close()?;
    Ok(None)
}

#[tauri::command]
pub async fn remove_user(user: uuid::Uuid) -> Result<()> {
    Ok(minecraft_auth::remove_user(user).await?)
}

#[tauri::command]
pub async fn get_default_user() -> Result<Option<uuid::Uuid>> {
    Ok(minecraft_auth::get_default_user().await?)
}

#[tauri::command]
pub async fn set_default_user(user: uuid::Uuid) -> Result<()> {
    Ok(minecraft_auth::set_default_user(user).await?)
}

/// Get a copy of the list of all user credentials
#[tauri::command]
pub async fn get_users() -> Result<Vec<Credentials>> {
    Ok(minecraft_auth::users().await?)
}

#[tauri::command]
pub async fn login_offline(username: String, use_elyby: bool) -> Result<Credentials> {
    let hash = md5::compute(format!("OfflinePlayer:{}", username).as_bytes());
    let mut bytes = hash.0;
    bytes[6] = (bytes[6] & 0x0f) | 0x30;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    let uuid = uuid::Uuid::from_bytes(bytes);

    let access_token = if use_elyby { "offline:elyby" } else { "offline" };

    let credentials = Credentials {
        offline_profile: MinecraftProfile {
            id: uuid,
            name: username.clone(),
            skins: vec![],
            capes: vec![],
            fetch_time: Some(std::time::Instant::now()),
        },
        access_token: access_token.to_string(),
        refresh_token: "offline".to_string(),
        expires: Utc::now() + Duration::days(365 * 10),
        active: true,
    };

    let state = theseus::State::get().await?;
    credentials.upsert(&state.pool).await?;

    Ok(credentials)
}

#[tauri::command]
pub async fn login_endrage(
    username: String,
    token: String,
    uuid_str: Option<String>,
) -> Result<Credentials> {
    let uuid = if let Some(ref u) = uuid_str {
        uuid::Uuid::parse_str(u).unwrap_or_else(|_| {
            let hash = md5::compute(format!("OfflinePlayer:{}", username).as_bytes());
            let mut bytes = hash.0;
            bytes[6] = (bytes[6] & 0x0f) | 0x30;
            bytes[8] = (bytes[8] & 0x3f) | 0x80;
            uuid::Uuid::from_bytes(bytes)
        })
    } else {
        let hash = md5::compute(format!("OfflinePlayer:{}", username).as_bytes());
        let mut bytes = hash.0;
        bytes[6] = (bytes[6] & 0x0f) | 0x30;
        bytes[8] = (bytes[8] & 0x3f) | 0x80;
        uuid::Uuid::from_bytes(bytes)
    };

    let credentials = Credentials {
        offline_profile: MinecraftProfile {
            id: uuid,
            name: username.clone(),
            skins: vec![],
            capes: vec![],
            fetch_time: Some(std::time::Instant::now()),
        },
        access_token: format!("endrage:{}", token),
        refresh_token: "endrage".to_string(),
        expires: Utc::now() + Duration::days(365 * 10),
        active: true,
    };

    let state = theseus::State::get().await?;
    credentials.upsert(&state.pool).await?;

    Ok(credentials)
}

#[derive(serde::Deserialize)]
struct OAuthExchangeResponse {
    #[allow(dead_code)]
    success: Option<bool>,
    #[serde(rename = "accessToken")]
    access_token: Option<String>,
    profile: Option<OAuthProfile>,
    #[serde(rename = "errorMessage")]
    error_message: Option<String>,
    #[allow(dead_code)]
    error: Option<String>,
}

#[derive(serde::Deserialize)]
struct OAuthProfile {
    id: Option<String>,
    name: Option<String>,
}

#[tauri::command]
pub async fn login_endrage_oauth<R: Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<Option<Credentials>> {
    const PORT: u16 = 25585;
    let redirect_uri = format!("http://localhost:{PORT}/callback");
    let authorize_url = format!(
        "https://end-rage.ru/oauth/authorize?client_id=era_launcher_client_app&redirect_uri=http%3A%2F%2Flocalhost%3A{}%2Fcallback&response_type=code",
        PORT
    );

    // 1. Bind local TCP listener on standard redirect port
    let listener = match tokio::net::TcpListener::bind(("127.0.0.1", PORT)).await {
        Ok(l) => l,
        Err(e) => {
            return Err(theseus::ErrorKind::LauncherError(format!(
                "Порт {} уже занят другой программой: {}. Освободите порт для авторизации.",
                PORT, e
            ))
            .as_error()
            .into());
        }
    };

    tracing::info!("End-Rage OAuth listener active on 127.0.0.1:{}", PORT);

    // 2. Open official authorization page in user's default browser
    if let Err(e) = app.opener().open_url(&authorize_url, None::<&str>) {
        tracing::warn!("Tauri opener failed to open browser for End-Rage OAuth: {e}");
    }

    // 3. Wait for browser redirect (timeout 5 minutes)
    let timeout_duration = std::time::Duration::from_secs(300);
    let start_time = tokio::time::Instant::now();
    let mut auth_code = None;

    while start_time.elapsed() < timeout_duration {
        let accept_fut = listener.accept();
        let sleep_fut = tokio::time::sleep(std::time::Duration::from_millis(200));

        tokio::select! {
            res = accept_fut => {
                match res {
                    Ok((mut stream, _addr)) => {
                        let mut buffer = [0u8; 4096];
                        use tokio::io::{AsyncReadExt, AsyncWriteExt};

                        if let Ok(bytes_read) = stream.read(&mut buffer).await {
                            let request_str = String::from_utf8_lossy(&buffer[..bytes_read]);

                            if let Some(first_line) = request_str.lines().next() {
                                if let Some(path_and_query) = first_line.split_whitespace().nth(1) {
                                    if let Ok(parsed_url) = url::Url::parse(&format!("http://localhost{}", path_and_query)) {
                                        if let Some((_, code)) = parsed_url.query_pairs().find(|(k, _)| k == "code") {
                                            auth_code = Some(code.to_string());

                                             let html_body = r#"<!DOCTYPE html>
<html lang="ru">
<head>
  <meta charset="utf-8">
  <title>End-Rage Launcher</title>
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <style>
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; background: #0c0d11; color: #f1f5f9; display: flex; align-items: center; justify-content: center; min-height: 100vh; padding: 20px; }
    .card { background: #13141b; border: 1px solid #232634; border-radius: 18px; padding: 36px 32px; text-align: center; max-width: 380px; width: 100%; box-shadow: 0 20px 40px rgba(0,0,0,0.6); }
    .brand { font-size: 11px; font-weight: 700; text-transform: uppercase; letter-spacing: 1.5px; color: #94a3b8; margin-bottom: 16px; }
    h1 { font-size: 20px; font-weight: 700; color: #ffffff; margin-bottom: 8px; }
    p { font-size: 13px; color: #94a3b8; line-height: 1.5; margin-bottom: 24px; }
    .progress-bar { width: 100%; height: 3px; background: #1e2230; border-radius: 999px; overflow: hidden; }
    .progress-fill { height: 100%; background: #6366f1; width: 0%; animation: fill 1.5s linear forwards; }
    @keyframes fill { from { width: 0%; } to { width: 100%; } }
  </style>
</head>
<body>
  <div class="card">
    <div class="brand">End-Rage Launcher</div>
    <h1>Вход выполнен</h1>
    <p>Авторизация через End-Rage ID завершена.<br>Эту вкладку можно закрыть.</p>
    <div class="progress-bar"><div class="progress-fill"></div></div>
  </div>
  <script>
    setTimeout(() => {
      try { window.close(); } catch (e) {}
    }, 1500);
  </script>
</body>
</html>"#;
                                            let response = format!(
                                                "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                                                html_body.len(),
                                                html_body
                                            );
                                            let _ = stream.write_all(response.as_bytes()).await;
                                            let _ = stream.flush().await;
                                            break;
                                        }
                                    }
                                }
                            }

                            let bad_req = "HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
                            let _ = stream.write_all(bad_req.as_bytes()).await;
                            let _ = stream.flush().await;
                        }
                    }
                    Err(e) => {
                        tracing::warn!("OAuth TCP accept error: {e}");
                    }
                }
            }
            _ = sleep_fut => {}
        }
    }

    let code = match auth_code {
        Some(c) => c,
        None => {
            tracing::info!("End-Rage OAuth flow closed or timed out");
            return Ok(None);
        }
    };

    tracing::info!("End-Rage OAuth code received, exchanging token...");

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(12))
        .build()?;

    let exchange_payload = serde_json::json!({
        "code": code,
        "redirect_uri": redirect_uri
    });

    let mut exchange_result: Option<OAuthExchangeResponse> = None;

    // 1. Try public backend endpoint
    let try_endpoints = [
        "https://api.end-rage.ru/api/oauth/exchange",
        "http://31.77.146.228:4004/api/oauth/exchange",
    ];

    for endpoint in try_endpoints {
        if let Ok(resp) = client.post(endpoint).json(&exchange_payload).send().await {
            if resp.status().is_success() {
                if let Ok(data) = resp.json::<OAuthExchangeResponse>().await {
                    if data.access_token.is_some() {
                        exchange_result = Some(data);
                        break;
                    }
                }
            }
        }
    }

    // 2. Direct fallback to official end-rage.ru API
    if exchange_result.is_none() {
        tracing::info!("Using direct OAuth endpoint on end-rage.ru...");
        let direct_payload = serde_json::json!({
            "grant_type": "authorization_code",
            "client_id": "era_launcher_client_app",
            "client_secret": "ers_launcher_sec_7f9b2d8e4c1a",
            "code": code,
            "redirect_uri": redirect_uri
        });

        if let Ok(resp) = client.post("https://end-rage.ru/api/oauth/token").json(&direct_payload).send().await {
            if let Ok(token_data) = resp.json::<serde_json::Value>().await {
                if let Some(token_str) = token_data.get("access_token").and_then(|v| v.as_str()) {
                    let mut username = "EndRagePlayer".to_string();
                    let mut user_id: Option<String> = None;

                    if let Ok(ui_resp) = client.get("https://end-rage.ru/api/oauth/userinfo")
                        .header("Authorization", format!("Bearer {token_str}"))
                        .send()
                        .await
                    {
                        if let Ok(ui_data) = ui_resp.json::<serde_json::Value>().await {
                            if let Some(nick) = ui_data.get("launcherNick").or_else(|| ui_data.get("username")).and_then(|v| v.as_str()) {
                                username = nick.to_string();
                            }
                            if let Some(id_str) = ui_data.get("id").and_then(|v| v.as_str()) {
                                user_id = Some(id_str.to_string());
                            }
                        }
                    }

                    exchange_result = Some(OAuthExchangeResponse {
                        success: Some(true),
                        access_token: Some(token_str.to_string()),
                        profile: Some(OAuthProfile {
                            id: user_id,
                            name: Some(username),
                        }),
                        error_message: None,
                        error: None,
                    });
                }
            }
        }
    }

    let final_res = exchange_result.ok_or_else(|| {
        theseus::ErrorKind::LauncherError(
            "Не удалось связаться с сервером авторизации End-Rage. Проверьте подключение к интернету.".to_string()
        )
        .as_error()
    })?;

    if let Some(err_msg) = final_res.error_message {
        return Err(theseus::ErrorKind::LauncherError(format!(
            "Ошибка авторизации End-Rage: {}",
            err_msg
        ))
        .as_error()
        .into());
    }

    let token = final_res.access_token.ok_or_else(|| {
        theseus::ErrorKind::LauncherError("Токен авторизации не был получен от сервера".to_string()).as_error()
    })?;

    let profile = final_res.profile.ok_or_else(|| {
        theseus::ErrorKind::LauncherError("Профиль пользователя не найден в ответе сервера".to_string()).as_error()
    })?;

    let username = profile.name.unwrap_or_else(|| "EndRagePlayer".to_string());

    let uuid = if let Some(ref u) = profile.id {
        uuid::Uuid::parse_str(u.as_str()).unwrap_or_else(|_| {
            let hash = md5::compute(format!("OfflinePlayer:{}", username).as_bytes());
            let mut bytes = hash.0;
            bytes[6] = (bytes[6] & 0x0f) | 0x30;
            bytes[8] = (bytes[8] & 0x3f) | 0x80;
            uuid::Uuid::from_bytes(bytes)
        })
    } else {
        let hash = md5::compute(format!("OfflinePlayer:{}", username).as_bytes());
        let mut bytes = hash.0;
        bytes[6] = (bytes[6] & 0x0f) | 0x30;
        bytes[8] = (bytes[8] & 0x3f) | 0x80;
        uuid::Uuid::from_bytes(bytes)
    };

    let credentials = Credentials {
        offline_profile: MinecraftProfile {
            id: uuid,
            name: username.clone(),
            skins: vec![],
            capes: vec![],
            fetch_time: Some(std::time::Instant::now()),
        },
        access_token: format!("endrage:{}", token),
        refresh_token: "endrage".to_string(),
        expires: Utc::now() + Duration::days(365 * 10),
        active: true,
    };

    let state = theseus::State::get().await?;
    credentials.upsert(&state.pool).await?;
    minecraft_auth::set_default_user(uuid).await?;

    if let Some(window) = app.get_window("main") {
        let _ = window.set_focus();
        let _ = window.request_user_attention(Some(UserAttentionType::Informational));
    }

    Ok(Some(credentials))
}

