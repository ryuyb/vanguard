use std::time::{SystemTime, UNIX_EPOCH};

/// Information about a captured focus state
#[derive(Debug, Clone)]
pub struct FocusInfo {
    /// The bundle identifier of the application that had focus
    pub app_bundle_id: String,
    /// The process ID of the application
    pub pid: i32,
    /// The window ID of the focused window (platform-specific)
    pub window_id: u64,
    /// Whether the focused element is a text input field
    pub is_text_input: bool,
    /// The timestamp when focus was captured
    pub captured_at_ms: i64,
}

impl FocusInfo {
    /// Check if the focus info is still valid (not expired)
    pub fn is_valid(&self, timeout_ms: i64) -> bool {
        match now_unix_ms() {
            Ok(now_ms) => {
                let elapsed = now_ms.saturating_sub(self.captured_at_ms);
                elapsed < timeout_ms
            }
            Err(_) => false,
        }
    }
}

/// Tracks the current focus state for autofill functionality
pub struct FocusTracker {
    last_focus: Option<FocusInfo>,
    /// Timeout for focus validity in milliseconds (default: 5 minutes)
    validity_timeout_ms: i64,
}

impl FocusTracker {
    pub fn new() -> Self {
        Self {
            last_focus: None,
            validity_timeout_ms: 5 * 60 * 1000, // 5 minutes
        }
    }

    /// Capture the current focus information
    pub fn capture_focus(&mut self) {
        match capture_focus_platform() {
            Ok(focus_info) => {
                log::debug!(
                    target: "vanguard::focus_tracker",
                    "Captured focus: app={}, window_id={}, is_text_input={}",
                    focus_info.app_bundle_id,
                    focus_info.window_id,
                    focus_info.is_text_input
                );
                self.last_focus = Some(focus_info);
            }
            Err(error) => {
                log::warn!(
                    target: "vanguard::focus_tracker",
                    "Failed to capture focus: {}",
                    error
                );
                self.last_focus = None;
            }
        }
    }

    /// Get the last captured focus if it's still valid
    pub fn get_valid_focus(&self) -> Option<&FocusInfo> {
        self.last_focus
            .as_ref()
            .filter(|f| f.is_valid(self.validity_timeout_ms))
    }

    /// Clear the stored focus information
    pub fn clear(&mut self) {
        self.last_focus = None;
    }

    /// Set a custom validity timeout
    pub fn set_validity_timeout(&mut self, timeout_ms: i64) {
        self.validity_timeout_ms = timeout_ms;
    }
}

impl Default for FocusTracker {
    fn default() -> Self {
        Self::new()
    }
}

fn now_unix_ms() -> Result<i64, ()> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .map_err(|_| ())
}

/// Platform-specific focus capture implementation
#[cfg(target_os = "macos")]
fn capture_focus_platform() -> Result<FocusInfo, String> {
    use objc2::msg_send;
    use objc2::runtime::AnyObject;
    use objc2_foundation::{NSAutoreleasePool, NSString};

    unsafe {
        let _pool = NSAutoreleasePool::new();

        // Get the frontmost application
        let workspace_class =
            objc2::runtime::AnyClass::get(c"NSWorkspace")
                .ok_or("NSWorkspace not found")?;

        let workspace: *mut AnyObject = msg_send![workspace_class, sharedWorkspace];
        if workspace.is_null() {
            return Err("Failed to get shared workspace".to_string());
        }

        let frontmost_app: *mut AnyObject = msg_send![workspace, frontmostApplication];
        if frontmost_app.is_null() {
            return Err("No frontmost application".to_string());
        }

        // Get bundle identifier
        let bundle_id_nsstring: *mut NSString = msg_send![frontmost_app, bundleIdentifier];
        let app_bundle_id = if bundle_id_nsstring.is_null() {
            "unknown".to_string()
        } else {
            let nsstring_ref: &NSString = &*bundle_id_nsstring;
            nsstring_ref.to_string()
        };

        // Get process identifier
        let pid: i32 = msg_send![frontmost_app, processIdentifier];

        // For now, we assume it's a text input - we'll refine this later
        // The actual text input detection requires Accessibility permissions
        let is_text_input = true;

        // Generate a window ID from pid (simplified approach)
        let window_id = pid as u64;

        Ok(FocusInfo {
            app_bundle_id,
            pid,
            window_id,
            is_text_input,
            captured_at_ms: now_unix_ms().unwrap_or(0),
        })
    }
}

#[cfg(not(target_os = "macos"))]
fn capture_focus_platform() -> Result<FocusInfo, String> {
    // Non-macOS platforms: return a placeholder
    // This allows the code to compile on other platforms but won't provide actual focus tracking
    Err("Focus tracking not implemented for this platform".to_string())
}
