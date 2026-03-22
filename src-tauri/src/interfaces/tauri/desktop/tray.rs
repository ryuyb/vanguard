use tauri::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Runtime};

use crate::bootstrap::app_state::AppState;
use crate::bootstrap::config::AppConfig;
use crate::interfaces::tauri::desktop::constants::{
    TRAY_ICON_ID, TRAY_MENU_LOCK_ID, TRAY_MENU_OPEN_QUICK_ACCESS_ID, TRAY_MENU_OPEN_VANGUARD_ID,
    TRAY_MENU_QUIT_ID, TRAY_MENU_SETTINGS_ID,
};
use crate::interfaces::tauri::desktop::main_window::MainWindowFeature;
use crate::interfaces::tauri::desktop::shortcut_utils::shortcut_to_accelerator;
use crate::interfaces::tauri::desktop::spotlight::SpotlightFeature;
use crate::interfaces::tauri::desktop::tray_click_snapshot::{
    TrayClickSnapshot, TrayClickSnapshotStore,
};
use crate::interfaces::tauri::desktop::tray_i18n::{TrayLocale, TrayMenuTexts};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TrayMenuAction {
    OpenMain,
    OpenQuickAccess,
    Lock,
    Settings,
    Quit,
    Unknown,
}

impl TrayMenuAction {
    fn from_menu_id(menu_id: &str) -> Self {
        match menu_id {
            TRAY_MENU_OPEN_VANGUARD_ID => Self::OpenMain,
            TRAY_MENU_OPEN_QUICK_ACCESS_ID => Self::OpenQuickAccess,
            TRAY_MENU_LOCK_ID => Self::Lock,
            TRAY_MENU_SETTINGS_ID => Self::Settings,
            TRAY_MENU_QUIT_ID => Self::Quit,
            _ => Self::Unknown,
        }
    }
}

pub struct TrayFeature;

impl TrayFeature {
    /// 更新托盘菜单(包括语言和锁定状态)
    pub fn update_menu<R: Runtime>(app_handle: &AppHandle<R>) {
        // 检查是否已解锁 - 使用 blocking 调用
        let is_unlocked = app_handle
            .try_state::<AppState>()
            .and_then(|state| {
                state.active_account_id().ok().and_then(|account_id| {
                    // Use tokio runtime to block on the async call
                    let rt = tokio::runtime::Handle::try_current();
                    match rt {
                        Ok(handle) => handle
                            .block_on(state.get_vault_user_key(&account_id))
                            .unwrap_or_default(),
                        Err(_) => None,
                    }
                })
            })
            .is_some();

        // 重建托盘菜单
        if let Ok(new_menu) = Self::build_menu_with_lock_state(app_handle, is_unlocked) {
            if let Some(tray) = app_handle.tray_by_id(TRAY_ICON_ID) {
                let _ = tray.set_menu(Some(new_menu));
            }
        }
    }

    /// 更新托盘菜单中锁定按钮的启用状态
    pub fn update_lock_menu_state<R: Runtime>(app_handle: &AppHandle<R>) {
        Self::update_menu(app_handle);
    }

    pub(super) fn install<R: Runtime>(app: &tauri::App<R>) -> tauri::Result<()> {
        let tray_menu = Self::build_menu(app)?;

        let mut tray_builder = TrayIconBuilder::with_id(TRAY_ICON_ID)
            .menu(&tray_menu)
            .show_menu_on_left_click(true)
            .on_tray_icon_event(move |_tray, event: TrayIconEvent| {
                if let TrayIconEvent::Click { position, rect, .. } = event {
                    TrayClickSnapshotStore::record(TrayClickSnapshot::from_click(
                        position, rect.size,
                    ));
                }
            })
            .on_menu_event(move |app_handle, event: MenuEvent| {
                Self::handle_menu_event(app_handle, &event);
            });

        // 使用专门的托盘图标(22x22,适合macOS托盘)
        if let Ok(tray_icon) = Self::load_tray_icon() {
            tray_builder = tray_builder.icon(tray_icon);
        } else if let Some(default_icon) = app.default_window_icon().cloned() {
            // 降级方案:使用窗口图标
            tray_builder = tray_builder.icon(default_icon);
        }

        let _ = tray_builder.build(app)?;
        Ok(())
    }

    fn load_tray_icon() -> Result<tauri::image::Image<'static>, Box<dyn std::error::Error>> {
        // 加载并解码托盘图标
        let icon_bytes = include_bytes!("../../../../icons/tray-icon.png");
        let img = image::load_from_memory(icon_bytes)?;
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();

        Ok(tauri::image::Image::new_owned(
            rgba.into_raw(),
            width,
            height,
        ))
    }

    fn build_menu<R: Runtime>(app: &tauri::App<R>) -> tauri::Result<Menu<R>> {
        Self::build_menu_with_lock_state(app, false)
    }

    fn build_menu_with_lock_state<R: Runtime>(
        manager: &impl Manager<R>,
        lock_enabled: bool,
    ) -> tauri::Result<Menu<R>> {
        // 从配置中读取语言和快捷键设置
        let config = AppConfig::load(manager).ok();
        let locale = config
            .as_ref()
            .map(|c| TrayLocale::from_locale_str(&c.locale))
            .unwrap_or(TrayLocale::Zh);

        let texts = TrayMenuTexts::get(locale);

        // 从配置中获取快捷键并转换为加速器格式
        let quick_access_accelerator = config
            .as_ref()
            .and_then(|c| shortcut_to_accelerator(&c.quick_access_shortcut));
        let lock_accelerator = config
            .as_ref()
            .and_then(|c| shortcut_to_accelerator(&c.lock_shortcut));

        let open_vanguard = MenuItem::with_id(
            manager,
            TRAY_MENU_OPEN_VANGUARD_ID,
            texts.open_vanguard,
            true,
            None::<&str>,
        )?;
        let open_quick_access = MenuItem::with_id(
            manager,
            TRAY_MENU_OPEN_QUICK_ACCESS_ID,
            texts.open_quick_access,
            true,
            quick_access_accelerator.as_deref(),
        )?;
        let separator = PredefinedMenuItem::separator(manager)?;
        let lock = MenuItem::with_id(
            manager,
            TRAY_MENU_LOCK_ID,
            texts.lock,
            lock_enabled,
            lock_accelerator.as_deref(),
        )?;
        let settings = MenuItem::with_id(
            manager,
            TRAY_MENU_SETTINGS_ID,
            texts.settings,
            true,
            None::<&str>,
        )?;
        let quit = MenuItem::with_id(manager, TRAY_MENU_QUIT_ID, texts.quit, true, None::<&str>)?;

        Menu::with_items(
            manager,
            &[
                &open_vanguard,
                &open_quick_access,
                &separator,
                &lock,
                &settings,
                &quit,
            ],
        )
    }

    fn handle_menu_event<R: Runtime>(app_handle: &tauri::AppHandle<R>, event: &MenuEvent) {
        match TrayMenuAction::from_menu_id(event.id().as_ref()) {
            TrayMenuAction::OpenMain => MainWindowFeature::open_from_tray(app_handle),
            TrayMenuAction::OpenQuickAccess => SpotlightFeature::toggle(app_handle),
            TrayMenuAction::Lock => log::info!(
                target: "vanguard::tray",
                "lock action is not wired yet; ignore menu event"
            ),
            TrayMenuAction::Settings => log::info!(
                target: "vanguard::tray",
                "settings action is not wired yet; ignore menu event"
            ),
            TrayMenuAction::Quit => app_handle.exit(0),
            TrayMenuAction::Unknown => {}
        }
    }
}
