#![allow(dead_code)]

/// Color palette based on Catppuccin Latte
/// https://catppuccin.com/palette/
///
/// This module provides a consistent color scheme for Discord embeds
/// using the Catppuccin Latte palette.
// Catppuccin Latte Base Colors
pub const ROSEWATER: u32 = 0xdc8a78;
pub const FLAMINGO: u32 = 0xdd7878;
pub const PINK: u32 = 0xea76cb;
pub const MAUVE: u32 = 0x8839ef;
pub const RED: u32 = 0xd20f39;
pub const MAROON: u32 = 0xe64553;
pub const PEACH: u32 = 0xfe640b;
pub const YELLOW: u32 = 0xdf8e1d;
pub const GREEN: u32 = 0x40a02b;
pub const TEAL: u32 = 0x179299;
pub const SKY: u32 = 0x04a5e5;
pub const SAPPHIRE: u32 = 0x209fb5;
pub const BLUE: u32 = 0x1e66f5;
pub const LAVENDER: u32 = 0x7287fd;

pub const BASE: u32 = 0x303446;
pub const MANTLE: u32 = 0x292c3c;
pub const CRUST: u32 = 0x232634;

// Semantic Colors for Common Use Cases
pub const SUCCESS: u32 = GREEN; // For successful operations
pub const ERROR: u32 = RED; // For errors and failures
pub const WARNING: u32 = YELLOW; // For warnings
pub const INFO: u32 = BLUE; // For informational messages
pub const PRIMARY: u32 = SAPPHIRE; // Primary/default color
pub const ACCENT: u32 = MAUVE; // Accent color
pub const LIVE: u32 = GREEN; // For live/active states
