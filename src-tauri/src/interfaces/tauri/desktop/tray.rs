use tauri::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::Runtime;

use crate::interfaces::tauri::desktop::constants::{
    TRAY_ICON_ID, TRAY_MENU_LOCK_ID, TRAY_MENU_OPEN_QUICK_ACCESS_ACCELERATOR,
    TRAY_MENU_OPEN_QUICK_ACCESS_ID, TRAY_MENU_OPEN_VANGUARD_ID, TRAY_MENU_QUIT_ID,
    TRAY_MENU_SETTINGS_ID,
};
use crate::interfaces::tauri::desktop::main_window::MainWindowFeature;
use crate::interfaces::tauri::desktop::spotlight::SpotlightFeature;
use crate::interfaces::tauri::desktop::tray_click_snapshot::{
    TrayClickSnapshot, TrayClickSnapshotStore,
};

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

pub(super) struct TrayFeature;

impl TrayFeature {
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

        if let Some(default_icon) = app.default_window_icon().cloned() {
            tray_builder = tray_builder.icon(default_icon);
        }

        let _ = tray_builder.build(app)?;
        Ok(())
    }

    fn build_menu<R: Runtime>(app: &tauri::App<R>) -> tauri::Result<Menu<R>> {
        let open_vanguard = MenuItem::with_id(
            app,
            TRAY_MENU_OPEN_VANGUARD_ID,
            "打开 Vanguard",
            true,
            None::<&str>,
        )?;
        let open_quick_access = MenuItem::with_id(
            app,
            TRAY_MENU_OPEN_QUICK_ACCESS_ID,
            "打开快速访问",
            true,
            Some(TRAY_MENU_OPEN_QUICK_ACCESS_ACCELERATOR),
        )?;
        let separator = PredefinedMenuItem::separator(app)?;
        let lock = MenuItem::with_id(app, TRAY_MENU_LOCK_ID, "锁定", true, None::<&str>)?;
        let settings = MenuItem::with_id(app, TRAY_MENU_SETTINGS_ID, "设置", true, None::<&str>)?;
        let quit = MenuItem::with_id(app, TRAY_MENU_QUIT_ID, "退出", true, None::<&str>)?;

        Menu::with_items(
            app,
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
