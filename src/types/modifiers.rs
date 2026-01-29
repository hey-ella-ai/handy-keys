//! Modifier key definitions and parsing

use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::error::{Error, Result};

bitflags! {
    /// Modifier keys for hotkey combinations
    ///
    /// Supports both generic modifiers (CMD, SHIFT, etc.) that match either side,
    /// and specific left/right variants (CMD_LEFT, CMD_RIGHT, etc.).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct Modifiers: u16 {
        // Left modifiers (bits 0-3)
        /// Left Command key (macOS) / Left Windows key (Windows) / Left Super key (Linux)
        const CMD_LEFT = 1 << 0;
        /// Left Shift key
        const SHIFT_LEFT = 1 << 1;
        /// Left Control key
        const CTRL_LEFT = 1 << 2;
        /// Left Option key (macOS) / Left Alt key (Windows/Linux)
        const OPT_LEFT = 1 << 3;

        // Right modifiers (bits 4-7)
        /// Right Command key (macOS) / Right Windows key (Windows) / Right Super key (Linux)
        const CMD_RIGHT = 1 << 4;
        /// Right Shift key
        const SHIFT_RIGHT = 1 << 5;
        /// Right Control key
        const CTRL_RIGHT = 1 << 6;
        /// Right Option key (macOS) / Right Alt key (Windows/Linux)
        const OPT_RIGHT = 1 << 7;

        // Function key (bit 8)
        /// Function key (macOS)
        const FN = 1 << 8;

        // Compatibility aliases - match EITHER left or right side
        /// Command key (either side) - macOS / Windows key / Super key
        const CMD = Self::CMD_LEFT.bits() | Self::CMD_RIGHT.bits();
        /// Shift key (either side)
        const SHIFT = Self::SHIFT_LEFT.bits() | Self::SHIFT_RIGHT.bits();
        /// Control key (either side)
        const CTRL = Self::CTRL_LEFT.bits() | Self::CTRL_RIGHT.bits();
        /// Option key (either side) - macOS / Alt key (Windows/Linux)
        const OPT = Self::OPT_LEFT.bits() | Self::OPT_RIGHT.bits();
    }
}

impl Modifiers {
    /// Check if this modifier set contains a left-side modifier
    pub fn contains_left(&self, modifier_type: ModifierType) -> bool {
        match modifier_type {
            ModifierType::Cmd => self.contains(Modifiers::CMD_LEFT),
            ModifierType::Shift => self.contains(Modifiers::SHIFT_LEFT),
            ModifierType::Ctrl => self.contains(Modifiers::CTRL_LEFT),
            ModifierType::Opt => self.contains(Modifiers::OPT_LEFT),
            ModifierType::Fn => self.contains(Modifiers::FN),
        }
    }

    /// Check if this modifier set contains a right-side modifier
    pub fn contains_right(&self, modifier_type: ModifierType) -> bool {
        match modifier_type {
            ModifierType::Cmd => self.contains(Modifiers::CMD_RIGHT),
            ModifierType::Shift => self.contains(Modifiers::SHIFT_RIGHT),
            ModifierType::Ctrl => self.contains(Modifiers::CTRL_RIGHT),
            ModifierType::Opt => self.contains(Modifiers::OPT_RIGHT),
            ModifierType::Fn => self.contains(Modifiers::FN), // FN has no left/right
        }
    }

    /// Check if this modifier is a specific left or right variant (not generic)
    pub fn is_side_specific(&self) -> bool {
        // A modifier is side-specific if it has only left OR only right bits set
        // for any modifier type, not both
        let has_cmd_left = self.contains(Modifiers::CMD_LEFT);
        let has_cmd_right = self.contains(Modifiers::CMD_RIGHT);
        let has_shift_left = self.contains(Modifiers::SHIFT_LEFT);
        let has_shift_right = self.contains(Modifiers::SHIFT_RIGHT);
        let has_ctrl_left = self.contains(Modifiers::CTRL_LEFT);
        let has_ctrl_right = self.contains(Modifiers::CTRL_RIGHT);
        let has_opt_left = self.contains(Modifiers::OPT_LEFT);
        let has_opt_right = self.contains(Modifiers::OPT_RIGHT);

        // If any modifier has only one side set, it's side-specific
        (has_cmd_left != has_cmd_right && (has_cmd_left || has_cmd_right))
            || (has_shift_left != has_shift_right && (has_shift_left || has_shift_right))
            || (has_ctrl_left != has_ctrl_right && (has_ctrl_left || has_ctrl_right))
            || (has_opt_left != has_opt_right && (has_opt_left || has_opt_right))
    }

    /// Parse a single modifier name (case-insensitive)
    pub(crate) fn parse_single(s: &str) -> Option<Modifiers> {
        match s.to_lowercase().as_str() {
            // Left-specific modifiers
            "leftcmd" | "leftcommand" | "lcmd" | "lcommand" | "lmeta" | "lsuper" | "lwin" => {
                Some(Modifiers::CMD_LEFT)
            }
            "leftshift" | "lshift" => Some(Modifiers::SHIFT_LEFT),
            "leftctrl" | "leftcontrol" | "lctrl" | "lcontrol" => Some(Modifiers::CTRL_LEFT),
            "leftopt" | "leftoption" | "leftalt" | "lopt" | "lalt" => Some(Modifiers::OPT_LEFT),

            // Right-specific modifiers
            "rightcmd" | "rightcommand" | "rcmd" | "rcommand" | "rmeta" | "rsuper" | "rwin" => {
                Some(Modifiers::CMD_RIGHT)
            }
            "rightshift" | "rshift" => Some(Modifiers::SHIFT_RIGHT),
            "rightctrl" | "rightcontrol" | "rctrl" | "rcontrol" => Some(Modifiers::CTRL_RIGHT),
            "rightopt" | "rightoption" | "rightalt" | "ropt" | "ralt" => Some(Modifiers::OPT_RIGHT),

            // Generic modifiers (match either side)
            "cmd" | "command" | "meta" | "super" | "win" | "windows" => Some(Modifiers::CMD),
            "shift" => Some(Modifiers::SHIFT),
            "ctrl" | "control" => Some(Modifiers::CTRL),
            "opt" | "option" | "alt" => Some(Modifiers::OPT),

            // Function key
            "fn" | "function" => Some(Modifiers::FN),

            _ => None,
        }
    }

    /// Check if the required modifiers are satisfied by the pressed modifiers.
    ///
    /// For generic modifiers (CMD, SHIFT, etc.), accepts either left or right.
    /// For specific modifiers (CMD_LEFT, SHIFT_RIGHT, etc.), requires exact match.
    pub fn is_satisfied_by(&self, pressed: Modifiers) -> bool {
        // Handle each modifier type
        // CMD
        if self.intersects(Modifiers::CMD) {
            let required_left = self.contains(Modifiers::CMD_LEFT);
            let required_right = self.contains(Modifiers::CMD_RIGHT);
            let pressed_left = pressed.contains(Modifiers::CMD_LEFT);
            let pressed_right = pressed.contains(Modifiers::CMD_RIGHT);

            if required_left && required_right {
                // Generic CMD: accept either side
                if !pressed_left && !pressed_right {
                    return false;
                }
            } else if required_left {
                // Specific left: require left
                if !pressed_left {
                    return false;
                }
            } else if required_right {
                // Specific right: require right
                if !pressed_right {
                    return false;
                }
            }
        }

        // SHIFT
        if self.intersects(Modifiers::SHIFT) {
            let required_left = self.contains(Modifiers::SHIFT_LEFT);
            let required_right = self.contains(Modifiers::SHIFT_RIGHT);
            let pressed_left = pressed.contains(Modifiers::SHIFT_LEFT);
            let pressed_right = pressed.contains(Modifiers::SHIFT_RIGHT);

            if required_left && required_right {
                if !pressed_left && !pressed_right {
                    return false;
                }
            } else if required_left {
                if !pressed_left {
                    return false;
                }
            } else if required_right
                && !pressed_right {
                    return false;
                }
        }

        // CTRL
        if self.intersects(Modifiers::CTRL) {
            let required_left = self.contains(Modifiers::CTRL_LEFT);
            let required_right = self.contains(Modifiers::CTRL_RIGHT);
            let pressed_left = pressed.contains(Modifiers::CTRL_LEFT);
            let pressed_right = pressed.contains(Modifiers::CTRL_RIGHT);

            if required_left && required_right {
                if !pressed_left && !pressed_right {
                    return false;
                }
            } else if required_left {
                if !pressed_left {
                    return false;
                }
            } else if required_right
                && !pressed_right {
                    return false;
                }
        }

        // OPT
        if self.intersects(Modifiers::OPT) {
            let required_left = self.contains(Modifiers::OPT_LEFT);
            let required_right = self.contains(Modifiers::OPT_RIGHT);
            let pressed_left = pressed.contains(Modifiers::OPT_LEFT);
            let pressed_right = pressed.contains(Modifiers::OPT_RIGHT);

            if required_left && required_right {
                if !pressed_left && !pressed_right {
                    return false;
                }
            } else if required_left {
                if !pressed_left {
                    return false;
                }
            } else if required_right
                && !pressed_right {
                    return false;
                }
        }

        // FN
        if self.contains(Modifiers::FN) && !pressed.contains(Modifiers::FN) {
            return false;
        }

        true
    }

    /// Normalize modifiers by collapsing side-specific flags to generic when both sides are set.
    ///
    /// This is useful for display purposes - if both CMD_LEFT and CMD_RIGHT are set,
    /// we display as "Cmd" rather than "LeftCmd+RightCmd".
    pub fn normalized(&self) -> Modifiers {
        *self // The bitflags already handle this naturally
    }
}

/// Modifier type for querying left/right variants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModifierType {
    Cmd,
    Shift,
    Ctrl,
    Opt,
    Fn,
}

impl fmt::Display for Modifiers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();

        // CTRL
        let has_ctrl_left = self.contains(Modifiers::CTRL_LEFT);
        let has_ctrl_right = self.contains(Modifiers::CTRL_RIGHT);
        if has_ctrl_left && has_ctrl_right {
            parts.push("Ctrl");
        } else if has_ctrl_left {
            parts.push("LeftCtrl");
        } else if has_ctrl_right {
            parts.push("RightCtrl");
        }

        // OPT
        let has_opt_left = self.contains(Modifiers::OPT_LEFT);
        let has_opt_right = self.contains(Modifiers::OPT_RIGHT);
        if has_opt_left && has_opt_right {
            parts.push("Opt");
        } else if has_opt_left {
            parts.push("LeftOpt");
        } else if has_opt_right {
            parts.push("RightOpt");
        }

        // SHIFT
        let has_shift_left = self.contains(Modifiers::SHIFT_LEFT);
        let has_shift_right = self.contains(Modifiers::SHIFT_RIGHT);
        if has_shift_left && has_shift_right {
            parts.push("Shift");
        } else if has_shift_left {
            parts.push("LeftShift");
        } else if has_shift_right {
            parts.push("RightShift");
        }

        // CMD
        let has_cmd_left = self.contains(Modifiers::CMD_LEFT);
        let has_cmd_right = self.contains(Modifiers::CMD_RIGHT);
        if has_cmd_left && has_cmd_right {
            parts.push("Cmd");
        } else if has_cmd_left {
            parts.push("LeftCmd");
        } else if has_cmd_right {
            parts.push("RightCmd");
        }

        // FN
        if self.contains(Modifiers::FN) {
            parts.push("Fn");
        }

        write!(f, "{}", parts.join("+"))
    }
}

impl FromStr for Modifiers {
    type Err = Error;

    /// Parse modifiers from a string like "Cmd+Shift" or "LeftCtrl+RightAlt"
    fn from_str(s: &str) -> Result<Self> {
        let s = s.trim();
        if s.is_empty() {
            return Ok(Modifiers::empty());
        }

        let mut modifiers = Modifiers::empty();
        for part in s.split('+') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            match Modifiers::parse_single(part) {
                Some(m) => modifiers |= m,
                None => return Err(Error::UnknownModifier(part.to_string())),
            }
        }
        Ok(modifiers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Existing tests (updated for new structure)

    #[test]
    fn parse_single_modifiers() {
        // Generic modifiers should match both sides
        assert_eq!("Cmd".parse::<Modifiers>().unwrap(), Modifiers::CMD);
        assert_eq!("command".parse::<Modifiers>().unwrap(), Modifiers::CMD);
        assert_eq!("meta".parse::<Modifiers>().unwrap(), Modifiers::CMD);
        assert_eq!("super".parse::<Modifiers>().unwrap(), Modifiers::CMD);
        assert_eq!("win".parse::<Modifiers>().unwrap(), Modifiers::CMD);

        assert_eq!("Shift".parse::<Modifiers>().unwrap(), Modifiers::SHIFT);
        assert_eq!("SHIFT".parse::<Modifiers>().unwrap(), Modifiers::SHIFT);

        assert_eq!("Ctrl".parse::<Modifiers>().unwrap(), Modifiers::CTRL);
        assert_eq!("control".parse::<Modifiers>().unwrap(), Modifiers::CTRL);

        assert_eq!("Opt".parse::<Modifiers>().unwrap(), Modifiers::OPT);
        assert_eq!("option".parse::<Modifiers>().unwrap(), Modifiers::OPT);
        assert_eq!("alt".parse::<Modifiers>().unwrap(), Modifiers::OPT);

        assert_eq!("Fn".parse::<Modifiers>().unwrap(), Modifiers::FN);
        assert_eq!("function".parse::<Modifiers>().unwrap(), Modifiers::FN);
    }

    #[test]
    fn parse_combined_modifiers() {
        assert_eq!(
            "Cmd+Shift".parse::<Modifiers>().unwrap(),
            Modifiers::CMD | Modifiers::SHIFT
        );
        assert_eq!(
            "Ctrl+Alt+Shift".parse::<Modifiers>().unwrap(),
            Modifiers::CTRL | Modifiers::OPT | Modifiers::SHIFT
        );
    }

    #[test]
    fn parse_empty_modifiers() {
        assert_eq!("".parse::<Modifiers>().unwrap(), Modifiers::empty());
        assert_eq!("  ".parse::<Modifiers>().unwrap(), Modifiers::empty());
    }

    #[test]
    fn parse_unknown_modifier_fails() {
        assert!("Unknown".parse::<Modifiers>().is_err());
        assert!("Cmd+Unknown".parse::<Modifiers>().is_err());
    }

    #[test]
    fn modifiers_display() {
        assert_eq!(format!("{}", Modifiers::CMD), "Cmd");
        assert_eq!(format!("{}", Modifiers::SHIFT), "Shift");
        assert_eq!(
            format!("{}", Modifiers::CMD | Modifiers::SHIFT),
            "Shift+Cmd"
        );
    }

    // New tests for left/right modifier support

    #[test]
    fn test_left_right_flags_are_distinct() {
        assert_ne!(Modifiers::CMD_LEFT, Modifiers::CMD_RIGHT);
        assert_ne!(Modifiers::SHIFT_LEFT, Modifiers::SHIFT_RIGHT);
        assert_ne!(Modifiers::CTRL_LEFT, Modifiers::CTRL_RIGHT);
        assert_ne!(Modifiers::OPT_LEFT, Modifiers::OPT_RIGHT);
    }

    #[test]
    fn test_generic_contains_both_sides() {
        assert!(Modifiers::CMD.contains(Modifiers::CMD_LEFT));
        assert!(Modifiers::CMD.contains(Modifiers::CMD_RIGHT));
        assert!(Modifiers::SHIFT.contains(Modifiers::SHIFT_LEFT));
        assert!(Modifiers::SHIFT.contains(Modifiers::SHIFT_RIGHT));
        assert!(Modifiers::CTRL.contains(Modifiers::CTRL_LEFT));
        assert!(Modifiers::CTRL.contains(Modifiers::CTRL_RIGHT));
        assert!(Modifiers::OPT.contains(Modifiers::OPT_LEFT));
        assert!(Modifiers::OPT.contains(Modifiers::OPT_RIGHT));
    }

    #[test]
    fn test_parse_left_modifiers() {
        assert_eq!("LeftCmd".parse::<Modifiers>().unwrap(), Modifiers::CMD_LEFT);
        assert_eq!(
            "leftshift".parse::<Modifiers>().unwrap(),
            Modifiers::SHIFT_LEFT
        );
        assert_eq!("LCtrl".parse::<Modifiers>().unwrap(), Modifiers::CTRL_LEFT);
        assert_eq!("lopt".parse::<Modifiers>().unwrap(), Modifiers::OPT_LEFT);
        assert_eq!("lalt".parse::<Modifiers>().unwrap(), Modifiers::OPT_LEFT);
    }

    #[test]
    fn test_parse_right_modifiers() {
        assert_eq!(
            "RightCmd".parse::<Modifiers>().unwrap(),
            Modifiers::CMD_RIGHT
        );
        assert_eq!(
            "rightshift".parse::<Modifiers>().unwrap(),
            Modifiers::SHIFT_RIGHT
        );
        assert_eq!("RCtrl".parse::<Modifiers>().unwrap(), Modifiers::CTRL_RIGHT);
        assert_eq!("ropt".parse::<Modifiers>().unwrap(), Modifiers::OPT_RIGHT);
        assert_eq!("ralt".parse::<Modifiers>().unwrap(), Modifiers::OPT_RIGHT);
    }

    #[test]
    fn test_display_left_right() {
        assert_eq!(Modifiers::CMD_LEFT.to_string(), "LeftCmd");
        assert_eq!(Modifiers::CMD_RIGHT.to_string(), "RightCmd");
        assert_eq!(Modifiers::SHIFT_LEFT.to_string(), "LeftShift");
        assert_eq!(Modifiers::SHIFT_RIGHT.to_string(), "RightShift");
        assert_eq!(Modifiers::CTRL_LEFT.to_string(), "LeftCtrl");
        assert_eq!(Modifiers::CTRL_RIGHT.to_string(), "RightCtrl");
        assert_eq!(Modifiers::OPT_LEFT.to_string(), "LeftOpt");
        assert_eq!(Modifiers::OPT_RIGHT.to_string(), "RightOpt");

        // Generic (both sides) should display without Left/Right
        assert_eq!(Modifiers::CMD.to_string(), "Cmd");
        assert_eq!(Modifiers::SHIFT.to_string(), "Shift");
    }

    #[test]
    fn test_combined_left_right_modifiers() {
        let mods = Modifiers::CMD_LEFT | Modifiers::SHIFT_RIGHT;
        assert!(mods.contains(Modifiers::CMD_LEFT));
        assert!(mods.contains(Modifiers::SHIFT_RIGHT));
        assert!(!mods.contains(Modifiers::CMD_RIGHT));
        assert!(!mods.contains(Modifiers::SHIFT_LEFT));
        assert_eq!(mods.to_string(), "RightShift+LeftCmd");
    }

    #[test]
    fn test_fn_unchanged() {
        assert_eq!("Fn".parse::<Modifiers>().unwrap(), Modifiers::FN);
        assert_eq!(Modifiers::FN.to_string(), "Fn");
    }

    #[test]
    fn test_is_satisfied_by_generic() {
        // Generic CMD should be satisfied by either left or right
        let required = Modifiers::CMD;
        assert!(required.is_satisfied_by(Modifiers::CMD_LEFT));
        assert!(required.is_satisfied_by(Modifiers::CMD_RIGHT));
        assert!(required.is_satisfied_by(Modifiers::CMD));
        assert!(!required.is_satisfied_by(Modifiers::empty()));
        assert!(!required.is_satisfied_by(Modifiers::SHIFT_LEFT));
    }

    #[test]
    fn test_is_satisfied_by_specific_left() {
        // Specific CMD_LEFT should only be satisfied by left
        let required = Modifiers::CMD_LEFT;
        assert!(required.is_satisfied_by(Modifiers::CMD_LEFT));
        assert!(!required.is_satisfied_by(Modifiers::CMD_RIGHT));
        assert!(required.is_satisfied_by(Modifiers::CMD)); // CMD has both bits, so left is present
    }

    #[test]
    fn test_is_satisfied_by_specific_right() {
        // Specific CMD_RIGHT should only be satisfied by right
        let required = Modifiers::CMD_RIGHT;
        assert!(!required.is_satisfied_by(Modifiers::CMD_LEFT));
        assert!(required.is_satisfied_by(Modifiers::CMD_RIGHT));
        assert!(required.is_satisfied_by(Modifiers::CMD)); // CMD has both bits, so right is present
    }

    #[test]
    fn test_is_satisfied_by_combined() {
        // LeftCmd+RightShift should require both
        let required = Modifiers::CMD_LEFT | Modifiers::SHIFT_RIGHT;
        assert!(required.is_satisfied_by(Modifiers::CMD_LEFT | Modifiers::SHIFT_RIGHT));
        assert!(!required.is_satisfied_by(Modifiers::CMD_LEFT | Modifiers::SHIFT_LEFT));
        assert!(!required.is_satisfied_by(Modifiers::CMD_RIGHT | Modifiers::SHIFT_RIGHT));
        assert!(!required.is_satisfied_by(Modifiers::CMD_LEFT));
    }

    #[test]
    fn test_contains_left_right() {
        let mods = Modifiers::CMD_LEFT | Modifiers::SHIFT_RIGHT;
        assert!(mods.contains_left(ModifierType::Cmd));
        assert!(!mods.contains_right(ModifierType::Cmd));
        assert!(!mods.contains_left(ModifierType::Shift));
        assert!(mods.contains_right(ModifierType::Shift));
    }

    #[test]
    fn test_is_side_specific() {
        // Side-specific modifiers
        assert!(Modifiers::CMD_LEFT.is_side_specific());
        assert!(Modifiers::CMD_RIGHT.is_side_specific());
        assert!(Modifiers::SHIFT_LEFT.is_side_specific());

        // Generic modifiers (both sides)
        assert!(!Modifiers::CMD.is_side_specific());
        assert!(!Modifiers::SHIFT.is_side_specific());

        // Mixed: has one specific and one generic
        let mixed = Modifiers::CMD_LEFT | Modifiers::SHIFT;
        assert!(mixed.is_side_specific()); // CMD_LEFT is specific

        // FN is not side-specific (no left/right)
        assert!(!Modifiers::FN.is_side_specific());
    }
}
