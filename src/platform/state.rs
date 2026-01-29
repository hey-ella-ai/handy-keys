//! Shared state for platform-specific keyboard listeners

use std::collections::HashSet;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use crate::types::{Hotkey, Key, KeyEvent, Modifiers};

/// Hotkeys that should be blocked when triggered
pub type BlockingHotkeys = Arc<Mutex<HashSet<Hotkey>>>;

/// Internal state shared with platform-specific event callbacks
pub struct ListenerState {
    pub event_sender: Sender<KeyEvent>,
    /// Track which modifiers are currently held (with specific left/right info)
    pub current_modifiers: Modifiers,
    /// Hotkeys to block (if any)
    pub blocking_hotkeys: Option<BlockingHotkeys>,
}

impl ListenerState {
    pub fn new(event_sender: Sender<KeyEvent>, blocking_hotkeys: Option<BlockingHotkeys>) -> Self {
        Self {
            event_sender,
            current_modifiers: Modifiers::empty(),
            blocking_hotkeys,
        }
    }

    /// Check if an event matches a blocking hotkey.
    ///
    /// Uses `is_satisfied_by()` to handle generic vs specific modifier matching:
    /// - A hotkey registered with generic `CMD` will match if either CMD_LEFT or CMD_RIGHT is pressed
    /// - A hotkey registered with specific `CMD_LEFT` will only match if CMD_LEFT is pressed
    pub fn should_block(&self, modifiers: Modifiers, key: Option<Key>) -> bool {
        if let Some(ref hotkeys) = self.blocking_hotkeys {
            if let Ok(set) = hotkeys.lock() {
                for hotkey in set.iter() {
                    // Check if the hotkey's key matches
                    if hotkey.key != key {
                        continue;
                    }
                    // Check if the hotkey's modifiers are satisfied by the pressed modifiers
                    if hotkey.modifiers.is_satisfied_by(modifiers) {
                        // Also verify no extra modifiers are pressed that aren't in the hotkey
                        // This prevents "Cmd+K" from matching when "Cmd+Shift+K" is pressed
                        if modifiers_match_exactly(hotkey.modifiers, modifiers) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
}

/// Check if pressed modifiers match the required modifiers exactly.
///
/// For each modifier type (CMD, SHIFT, CTRL, OPT, FN):
/// - If required has it (either side), pressed must have it (either side)
/// - If required doesn't have it, pressed must not have it
///
/// This prevents "Cmd+K" from matching when "Cmd+Shift+K" is pressed.
fn modifiers_match_exactly(required: Modifiers, pressed: Modifiers) -> bool {
    // Check each modifier type
    let req_has_cmd = required.intersects(Modifiers::CMD);
    let pressed_has_cmd = pressed.intersects(Modifiers::CMD);
    if req_has_cmd != pressed_has_cmd {
        return false;
    }

    let req_has_shift = required.intersects(Modifiers::SHIFT);
    let pressed_has_shift = pressed.intersects(Modifiers::SHIFT);
    if req_has_shift != pressed_has_shift {
        return false;
    }

    let req_has_ctrl = required.intersects(Modifiers::CTRL);
    let pressed_has_ctrl = pressed.intersects(Modifiers::CTRL);
    if req_has_ctrl != pressed_has_ctrl {
        return false;
    }

    let req_has_opt = required.intersects(Modifiers::OPT);
    let pressed_has_opt = pressed.intersects(Modifiers::OPT);
    if req_has_opt != pressed_has_opt {
        return false;
    }

    let req_has_fn = required.contains(Modifiers::FN);
    let pressed_has_fn = pressed.contains(Modifiers::FN);
    if req_has_fn != pressed_has_fn {
        return false;
    }

    true
}
