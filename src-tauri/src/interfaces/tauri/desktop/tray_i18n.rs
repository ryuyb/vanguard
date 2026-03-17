/// 托盘菜单国际化支持
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayLocale {
    Zh,
    En,
}

impl TrayLocale {
    pub fn from_locale_str(locale: &str) -> Self {
        match locale {
            "en" => Self::En,
            _ => Self::Zh, // 默认中文
        }
    }
}

pub struct TrayMenuTexts {
    pub open_vanguard: &'static str,
    pub open_quick_access: &'static str,
    pub lock: &'static str,
    pub settings: &'static str,
    pub quit: &'static str,
}

impl TrayMenuTexts {
    pub fn get(locale: TrayLocale) -> Self {
        match locale {
            TrayLocale::Zh => Self {
                open_vanguard: "打开 Vanguard",
                open_quick_access: "打开快速访问",
                lock: "锁定",
                settings: "设置",
                quit: "退出",
            },
            TrayLocale::En => Self {
                open_vanguard: "Open Vanguard",
                open_quick_access: "Open Quick Access",
                lock: "Lock",
                settings: "Settings",
                quit: "Quit",
            },
        }
    }
}
