use std::sync::Mutex;
use tauri::{
    LogicalPosition, LogicalSize, Manager, WebviewUrl,
    webview::WebviewBuilder,
};

const SIDEBAR_WIDTH: f64 = 280.0;
const FEIGE_URL: &str = "https://www.aiyiyong.com";

struct AppState {
    active_account: Mutex<Option<String>>,
    account_labels: Mutex<Vec<String>>,
}

/// 获取右侧区域的尺寸
fn get_right_panel_size(app: &tauri::AppHandle) -> (f64, f64) {
    if let Some(window) = app.get_window("main") {
        if let (Ok(size), Ok(scale)) = (window.inner_size(), window.scale_factor()) {
            let w = size.width as f64 / scale;
            let h = size.height as f64 / scale;
            return ((w - SIDEBAR_WIDTH).max(100.0), h);
        }
    }
    (1120.0, 800.0)
}

/// 隐藏所有账号 webview（移到屏幕外）
fn hide_all_accounts(app: &tauri::AppHandle, state: &AppState) {
    let labels = state.account_labels.lock().unwrap();
    for label in labels.iter() {
        if let Some(wv) = app.get_webview(label) {
            let _ = wv.set_position(LogicalPosition::new(-9999.0, -9999.0));
        }
    }
}

/// 显示指定账号 webview
fn show_account(app: &tauri::AppHandle, state: &AppState, label: &str) {
    hide_all_accounts(app, state);
    let (w, h) = get_right_panel_size(app);
    if let Some(wv) = app.get_webview(label) {
        let _ = wv.set_position(LogicalPosition::new(SIDEBAR_WIDTH, 0.0));
        let _ = wv.set_size(LogicalSize::new(w, h));
    }
    *state.active_account.lock().unwrap() = Some(label.to_string());
}

/// 打开/切换账号 webview
#[tauri::command]
fn open_account(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    account_id: String,
    account_name: String,
) -> Result<String, String> {
    let label = format!("account_{}", account_id);

    // 已存在则切换过去
    if app.get_webview(&label).is_some() {
        show_account(&app, &state, &label);
        return Ok(format!("已切换到: {}", account_name));
    }

    // 创建独立数据目录（Cookie 隔离）
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("sessions")
        .join(&account_id);
    std::fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;

    let url: url::Url = FEIGE_URL.parse().map_err(|e: url::ParseError| e.to_string())?;

    // 先隐藏其他账号
    hide_all_accounts(&app, &state);

    // 获取右侧面板尺寸
    let (w, h) = get_right_panel_size(&app);

    // 在主窗口中创建新 webview
    let window = app.get_window("main").ok_or("主窗口不存在")?;
    let webview = WebviewBuilder::new(&label, WebviewUrl::External(url))
        .data_directory(data_dir);

    window
        .add_child(
            webview,
            LogicalPosition::new(SIDEBAR_WIDTH, 0.0),
            LogicalSize::new(w, h),
        )
        .map_err(|e| e.to_string())?;

    // 更新状态
    state.account_labels.lock().unwrap().push(label.clone());
    *state.active_account.lock().unwrap() = Some(label);

    Ok(format!("已打开: {}", account_name))
}

/// 切换到指定账号
#[tauri::command]
fn switch_account(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    account_id: String,
) -> Result<String, String> {
    let label = format!("account_{}", account_id);
    if app.get_webview(&label).is_none() {
        return Err(format!("账号 {} 未打开", account_id));
    }
    show_account(&app, &state, &label);
    Ok(format!("已切换到: {}", account_id))
}

/// 关闭指定账号的 webview
#[tauri::command]
fn close_account(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    account_id: String,
) -> Result<String, String> {
    let label = format!("account_{}", account_id);

    // 关闭 webview
    if let Some(wv) = app.get_webview(&label) {
        wv.close().map_err(|e| e.to_string())?;
    }

    // 从状态中移除
    state.account_labels.lock().unwrap().retain(|l| l != &label);

    // 如果关的是当前激活的，清空
    let mut active = state.active_account.lock().unwrap();
    if active.as_deref() == Some(&label) {
        *active = None;
    }

    Ok(format!("已关闭: {}", account_id))
}

/// 获取当前激活的账号
#[tauri::command]
fn get_active_account(state: tauri::State<'_, AppState>) -> Option<String> {
    state.active_account.lock().unwrap().clone()
}

/// 窗口大小变化时重新布局
fn relayout(app: &tauri::AppHandle, state: &AppState) {
    let window = match app.get_window("main") {
        Some(w) => w,
        None => return,
    };
    let size = match window.inner_size() {
        Ok(s) => s,
        Err(_) => return,
    };
    let scale = window.scale_factor().unwrap_or(1.0);
    let logical_w = size.width as f64 / scale;
    let logical_h = size.height as f64 / scale;

    // 调整侧边栏高度
    if let Some(sidebar) = app.get_webview("sidebar") {
        let _ = sidebar.set_size(LogicalSize::new(SIDEBAR_WIDTH, logical_h));
    }

    // 调整当前激活的账号 webview
    let active = state.active_account.lock().unwrap();
    if let Some(ref label) = *active {
        if let Some(wv) = app.get_webview(label) {
            let right_w = (logical_w - SIDEBAR_WIDTH).max(100.0);
            let _ = wv.set_position(LogicalPosition::new(SIDEBAR_WIDTH, 0.0));
            let _ = wv.set_size(LogicalSize::new(right_w, logical_h));
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            active_account: Mutex::new(None),
            account_labels: Mutex::new(Vec::new()),
        })
        .setup(|app| {
            let handle = app.handle().clone();

            // 创建主窗口（裸窗口，不自带 webview）
            let window = tauri::window::WindowBuilder::new(app, "main")
                .title("飞鸽客服多开管理")
                .inner_size(1400.0, 800.0)
                .build()?;

            // 添加侧边栏 webview（Vue 管理界面）
            let sidebar = WebviewBuilder::new(
                "sidebar",
                WebviewUrl::App("index.html".into()),
            );

            window.add_child(
                sidebar,
                LogicalPosition::new(0.0, 0.0),
                LogicalSize::new(SIDEBAR_WIDTH, 800.0),
            )?;

            // 监听窗口大小变化，重新布局
            let app_handle = handle.clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::Resized(_) = event {
                    let state = app_handle.state::<AppState>();
                    relayout(&app_handle, state.inner());
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            open_account,
            switch_account,
            close_account,
            get_active_account,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}