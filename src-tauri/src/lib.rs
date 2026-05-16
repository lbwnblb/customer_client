use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

#[tauri::command]
fn open_workspace_window(app: tauri::AppHandle, account_id: String, account_name: String) -> Result<String, String> {
    let label = format!("workspace_{}", account_id);

    if app.get_webview_window(&label).is_some() {
        let existing = app.get_webview_window(&label).unwrap();
        existing.set_focus().map_err(|e| e.to_string())?;
        return Ok(format!("窗口已存在，已聚焦: {}", account_name));
    }

    let url: url::Url = "https://im.jinritemai.com/pc_seller_v2/main/workspace"
        .parse()
        .map_err(|e: url::ParseError| e.to_string())?;
    let webview_url = WebviewUrl::External(url);

    let data_dir = app.path().app_data_dir()
        .map_err(|e| e.to_string())?
        .join("sessions")
        .join(&account_id);
    std::fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;

    let window_label = label.clone();
    let title = format!("飞鸽客服 - {}", account_name);

    let main_win = app.get_webview_window("main");
    let (x, y) = if let Some(ref mw) = main_win {
        let pos = mw.outer_position().unwrap_or_default();
        let main_width = mw.inner_size().unwrap_or_default();
        (pos.x + main_width.width as i32 + 10, pos.y)
    } else {
        (330, 0)
    };

    WebviewWindowBuilder::new(&app, &window_label, webview_url)
        .title(&title)
        .inner_size(1200.0, 800.0)
        .position(x as f64, y as f64)
        .data_directory(data_dir)
        .build()
        .map_err(|e| e.to_string())?;

    Ok(format!("已打开窗口: {}", account_name))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![open_workspace_window])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
