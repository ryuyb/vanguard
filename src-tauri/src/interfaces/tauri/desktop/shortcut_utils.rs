//! Utility functions for parsing and converting keyboard shortcuts.
//!
//! Shortcut format: Uses Unicode symbols for modifier keys
//! - ⌃ = Ctrl
//! - ⇧ = Shift
//! - ⌥ = Alt
//! - ⌘ = Cmd (Command)
//! - ␣ = Space
//!
//! Example: "⌃⇧␣" means Ctrl+Shift+Space

use tauri_plugin_global_shortcut::{Code, Modifiers};

/// Converts a shortcut string with Unicode symbols to accelerator format for tray menus.
///
/// # Examples
/// - "⌃⇧␣" → "Ctrl+Shift+Space"
/// - "⇧⌘L" → "Shift+Cmd+L"
/// - "" → None (no shortcut)
///
/// Returns `None` if the input is empty or contains only whitespace.
pub fn shortcut_to_accelerator(shortcut: &str) -> Option<String> {
    let shortcut = shortcut.trim();
    if shortcut.is_empty() {
        return None;
    }

    let mut parts: Vec<String> = Vec::new();

    for ch in shortcut.chars() {
        match ch {
            '⌃' => parts.push("Ctrl".to_string()),
            '⇧' => parts.push("Shift".to_string()),
            '⌥' => parts.push("Alt".to_string()),
            '⌘' => parts.push("Cmd".to_string()),
            '␣' => parts.push("Space".to_string()),
            // Regular characters (letters, numbers, etc.)
            c if c.is_ascii_alphanumeric() => parts.push(c.to_string()),
            // Skip unknown characters
            _ => continue,
        }
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join("+"))
    }
}

/// Parses a shortcut string into Modifiers and Code for global shortcuts.
///
/// # Examples
/// - "⌃⇧␣" → (Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::Space)
/// - "⇧⌘L" → (Some(Modifiers::SHIFT | Modifiers::SUPER), Code::KeyL)
/// - "" → (None, None) - means no shortcut
///
/// Returns `None` for both values if the input is empty or cannot be parsed.
pub fn parse_shortcut(shortcut: &str) -> (Option<Modifiers>, Option<Code>) {
    let shortcut = shortcut.trim();
    if shortcut.is_empty() {
        return (None, None);
    }

    let mut modifiers = Modifiers::empty();
    let mut key_code: Option<Code> = None;

    for ch in shortcut.chars() {
        match ch {
            '⌃' => modifiers |= Modifiers::CONTROL,
            '⇧' => modifiers |= Modifiers::SHIFT,
            '⌥' => modifiers |= Modifiers::ALT,
            '⌘' => modifiers |= Modifiers::SUPER,
            '␣' => key_code = Some(Code::Space),
            // Letters A-Z
            'A'..='Z' => {
                key_code = Some(char_to_code(ch));
            }
            'a'..='z' => {
                key_code = Some(char_to_code(ch.to_ascii_uppercase()));
            }
            // Numbers 0-9
            '0'..='9' => {
                key_code = Some(char_to_code(ch));
            }
            // Skip unknown characters (including 'F' which is handled separately below)
            _ => continue,
        }
    }

    // Handle special case for F-keys (F1-F24)
    if shortcut.contains("F1")
        && !shortcut.contains("F10")
        && !shortcut.contains("F11")
        && !shortcut.contains("F12")
    {
        key_code = Some(Code::F1);
    } else if shortcut.contains("F2")
        && !shortcut.contains("F20")
        && !shortcut.contains("F21")
        && !shortcut.contains("F22")
        && !shortcut.contains("F23")
        && !shortcut.contains("F24")
    {
        key_code = Some(Code::F2);
    } else if shortcut.contains("F3") {
        key_code = Some(Code::F3);
    } else if shortcut.contains("F4") {
        key_code = Some(Code::F4);
    } else if shortcut.contains("F5") {
        key_code = Some(Code::F5);
    } else if shortcut.contains("F6") {
        key_code = Some(Code::F6);
    } else if shortcut.contains("F7") {
        key_code = Some(Code::F7);
    } else if shortcut.contains("F8") {
        key_code = Some(Code::F8);
    } else if shortcut.contains("F9") {
        key_code = Some(Code::F9);
    } else if shortcut.contains("F10") {
        key_code = Some(Code::F10);
    } else if shortcut.contains("F11") {
        key_code = Some(Code::F11);
    } else if shortcut.contains("F12") {
        key_code = Some(Code::F12);
    }

    let modifiers_opt = if modifiers.is_empty() {
        None
    } else {
        Some(modifiers)
    };

    (modifiers_opt, key_code)
}

/// Converts an ASCII alphanumeric character to a Code.
fn char_to_code(ch: char) -> Code {
    match ch {
        'A' => Code::KeyA,
        'B' => Code::KeyB,
        'C' => Code::KeyC,
        'D' => Code::KeyD,
        'E' => Code::KeyE,
        'F' => Code::KeyF,
        'G' => Code::KeyG,
        'H' => Code::KeyH,
        'I' => Code::KeyI,
        'J' => Code::KeyJ,
        'K' => Code::KeyK,
        'L' => Code::KeyL,
        'M' => Code::KeyM,
        'N' => Code::KeyN,
        'O' => Code::KeyO,
        'P' => Code::KeyP,
        'Q' => Code::KeyQ,
        'R' => Code::KeyR,
        'S' => Code::KeyS,
        'T' => Code::KeyT,
        'U' => Code::KeyU,
        'V' => Code::KeyV,
        'W' => Code::KeyW,
        'X' => Code::KeyX,
        'Y' => Code::KeyY,
        'Z' => Code::KeyZ,
        '0' => Code::Digit0,
        '1' => Code::Digit1,
        '2' => Code::Digit2,
        '3' => Code::Digit3,
        '4' => Code::Digit4,
        '5' => Code::Digit5,
        '6' => Code::Digit6,
        '7' => Code::Digit7,
        '8' => Code::Digit8,
        '9' => Code::Digit9,
        _ => Code::Space, // Fallback
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shortcut_to_accelerator_ctrl_shift_space() {
        assert_eq!(
            shortcut_to_accelerator("⌃⇧␣"),
            Some("Ctrl+Shift+Space".to_string())
        );
    }

    #[test]
    fn test_shortcut_to_accelerator_shift_cmd_l() {
        assert_eq!(
            shortcut_to_accelerator("⇧⌘L"),
            Some("Shift+Cmd+L".to_string())
        );
    }

    #[test]
    fn test_shortcut_to_accelerator_empty() {
        assert_eq!(shortcut_to_accelerator(""), None);
        assert_eq!(shortcut_to_accelerator("   "), None);
    }

    #[test]
    fn test_shortcut_to_accelerator_alt() {
        assert_eq!(
            shortcut_to_accelerator("⌥⌘A"),
            Some("Alt+Cmd+A".to_string())
        );
    }

    #[test]
    fn test_parse_shortcut_ctrl_shift_space() {
        let (modifiers, code) = parse_shortcut("⌃⇧␣");
        assert_eq!(modifiers, Some(Modifiers::CONTROL | Modifiers::SHIFT));
        assert_eq!(code, Some(Code::Space));
    }

    #[test]
    fn test_parse_shortcut_shift_cmd_l() {
        let (modifiers, code) = parse_shortcut("⇧⌘L");
        assert_eq!(modifiers, Some(Modifiers::SHIFT | Modifiers::SUPER));
        assert_eq!(code, Some(Code::KeyL));
    }

    #[test]
    fn test_parse_shortcut_empty() {
        let (modifiers, code) = parse_shortcut("");
        assert_eq!(modifiers, None);
        assert_eq!(code, None);
    }

    #[test]
    fn test_parse_shortcut_whitespace() {
        let (modifiers, code) = parse_shortcut("   ");
        assert_eq!(modifiers, None);
        assert_eq!(code, None);
    }

    #[test]
    fn test_parse_shortcut_only_modifiers() {
        let (modifiers, code) = parse_shortcut("⌃⇧");
        assert_eq!(modifiers, Some(Modifiers::CONTROL | Modifiers::SHIFT));
        assert_eq!(code, None);
    }

    #[test]
    fn test_parse_shortcut_only_key() {
        let (modifiers, code) = parse_shortcut("A");
        assert_eq!(modifiers, None);
        assert_eq!(code, Some(Code::KeyA));
    }

    #[test]
    fn test_parse_shortcut_lowercase() {
        let (modifiers, code) = parse_shortcut("⌃a");
        assert_eq!(modifiers, Some(Modifiers::CONTROL));
        assert_eq!(code, Some(Code::KeyA));
    }

    #[test]
    fn test_parse_shortcut_with_number() {
        let (modifiers, code) = parse_shortcut("⌃1");
        assert_eq!(modifiers, Some(Modifiers::CONTROL));
        assert_eq!(code, Some(Code::Digit1));
    }

    #[test]
    fn test_parse_shortcut_alt() {
        let (modifiers, code) = parse_shortcut("⌥⇧F");
        assert_eq!(modifiers, Some(Modifiers::ALT | Modifiers::SHIFT));
        assert_eq!(code, Some(Code::KeyF));
    }
}
