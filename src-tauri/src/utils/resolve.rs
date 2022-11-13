use crate::log_err;
use crate::{config::VergeN, core::*, utils::init, utils::server};
use tauri::{App, AppHandle, Manager};

/// handle something when start app
pub fn resolve_setup(app: &mut App) {
    #[cfg(target_os = "macos")]
    app.set_activation_policy(tauri::ActivationPolicy::Accessory);

    handle::Handle::global().init(app.app_handle());

    init::init_resources(app.package_info());

    // 启动核心
    log_err!(CoreManager::global().init());

    // log_err!(app
    //     .tray_handle()
    //     .set_menu(tray::Tray::tray_menu(&app.app_handle())));

    log_err!(tray::Tray::update_systray(&app.app_handle()));

    // setup a simple http server for singleton
    server::embed_server(app.app_handle());

    let silent_start = {
        let verge = VergeN::global().config.lock();
        verge.enable_silent_start.clone()
    };
    if !silent_start.unwrap_or(false) {
        create_window(&app.app_handle());
    }

    log_err!(sysopt::Sysopt::global().init_launch());
    log_err!(sysopt::Sysopt::global().init_sysproxy());

    log_err!(handle::Handle::update_systray_part());
    log_err!(hotkey::Hotkey::global().init(app.app_handle()));
    log_err!(timer::Timer::global().init());
}

/// reset system proxy
pub fn resolve_reset() {
    log_err!(sysopt::Sysopt::global().reset_sysproxy());
    log_err!(CoreManager::global().stop_core());
}

/// create main window
pub fn create_window(app_handle: &AppHandle) {
    if let Some(window) = app_handle.get_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
        return;
    }

    let builder = tauri::window::WindowBuilder::new(
        app_handle,
        "main".to_string(),
        tauri::WindowUrl::App("index.html".into()),
    )
    .title("Clash Verge")
    .center()
    .fullscreen(false)
    .min_inner_size(600.0, 520.0);

    #[cfg(target_os = "windows")]
    {
        use crate::utils::winhelp;
        use std::time::Duration;
        use tokio::time::sleep;
        use window_shadows::set_shadow;
        use window_vibrancy::apply_blur;

        match builder
            .decorations(false)
            .transparent(true)
            .inner_size(800.0, 636.0)
            .build()
        {
            Ok(_) => {
                let app_handle = app_handle.clone();

                tauri::async_runtime::spawn(async move {
                    sleep(Duration::from_secs(1)).await;

                    if let Some(window) = app_handle.get_window("main") {
                        let _ = window.show();
                        let _ = set_shadow(&window, true);

                        if !winhelp::is_win11() {
                            let _ = apply_blur(&window, None);
                        }
                    }
                });
            }
            Err(err) => log::error!(target: "app", "{err}"),
        }
    }

    #[cfg(target_os = "macos")]
    crate::log_if_err!(builder.decorations(true).inner_size(800.0, 642.0).build());

    #[cfg(target_os = "linux")]
    crate::log_if_err!(builder
        .decorations(false)
        .transparent(true)
        .inner_size(800.0, 636.0)
        .build());
}
