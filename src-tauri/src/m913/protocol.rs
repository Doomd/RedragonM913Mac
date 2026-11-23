//! Redragon M913 Impact Elite (Areson 25a7:fa07 / 25a7:fa08) HID protocol.
//! Reference: Qehbr/m913-ctl (reverse-engineered from USB captures).

use serde::{Deserialize, Serialize};

pub const M913_VID: u16 = 0x25a7;
pub const M913_PID_WIRELESS: u16 = 0xfa07;
pub const M913_PID_WIRED: u16 = 0xfa08;

pub const PACKET_SIZE: usize = 17;
pub const REPORT_ID_OUT: u8 = 0x08;
pub const REPORT_ID_IN: u8 = 0x09;

pub const ADDR_BTN_BASE: u16 = 0x0060;
pub const BTN_STRIDE: u16 = 4;
pub const ADDR_KB_BASE: u16 = 0x0100;
pub const KB_STRIDE: u16 = 0x0020;

/// 4-byte marker in the button table meaning "look up the keyboard event list at 0x0100 + idx*0x20".
pub const KB_LIST_MARKER: [u8; 4] = [0x05, 0x00, 0x00, 0x50];

pub type Packet = [u8; PACKET_SIZE];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonState {
    pub id: String,
    pub label: String,
    pub action: String,
    pub default_action: String,
    pub is_modified: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Button {
    Side1 = 0,
    Side2 = 1,
    Side3 = 2,
    Side4 = 3,
    Side5 = 4,
    Side6 = 5,
    Right = 6,
    Left = 7,
    Side7 = 8,
    Side8 = 9,
    Middle = 10,
    Fire = 11,
    Side9 = 12,
    Side10 = 13,
    Side11 = 14,
    Side12 = 15,
}

pub const ALL_BUTTONS: [Button; 16] = [
    Button::Side1,
    Button::Side2,
    Button::Side3,
    Button::Side4,
    Button::Side5,
    Button::Side6,
    Button::Right,
    Button::Left,
    Button::Side7,
    Button::Side8,
    Button::Middle,
    Button::Fire,
    Button::Side9,
    Button::Side10,
    Button::Side11,
    Button::Side12,
];

impl Button {
    pub fn id(self) -> &'static str {
        match self {
            Button::Side1 => "side1",
            Button::Side2 => "side2",
            Button::Side3 => "side3",
            Button::Side4 => "side4",
            Button::Side5 => "side5",
            Button::Side6 => "side6",
            Button::Right => "right",
            Button::Left => "left",
            Button::Side7 => "side7",
            Button::Side8 => "side8",
            Button::Middle => "middle",
            Button::Fire => "fire",
            Button::Side9 => "side9",
            Button::Side10 => "side10",
            Button::Side11 => "side11",
            Button::Side12 => "side12",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Button::Side1 => "1",
            Button::Side2 => "2",
            Button::Side3 => "3",
            Button::Side4 => "4",
            Button::Side5 => "5",
            Button::Side6 => "6",
            Button::Right => "Right Click",
            Button::Left => "Left Click",
            Button::Side7 => "7",
            Button::Side8 => "8",
            Button::Middle => "Middle Click",
            Button::Fire => "Fire (3x)",
            Button::Side9 => "9",
            Button::Side10 => "10",
            Button::Side11 => "11",
            Button::Side12 => "12",
        }
    }

    pub fn factory_default(self) -> &'static str {
        match self {
            Button::Side1 => "1",
            Button::Side2 => "2",
            Button::Side3 => "3",
            Button::Side4 => "4",
            Button::Side5 => "5",
            Button::Side6 => "6",
            Button::Right => "right",
            Button::Left => "left",
            Button::Side7 => "7",
            Button::Side8 => "8",
            Button::Middle => "middle",
            Button::Fire => "fire",
            Button::Side9 => "9",
            Button::Side10 => "0",
            Button::Side11 => "numplus",
            Button::Side12 => "numminus",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "side1" => Some(Button::Side1),
            "side2" => Some(Button::Side2),
            "side3" => Some(Button::Side3),
            "side4" => Some(Button::Side4),
            "side5" => Some(Button::Side5),
            "side6" => Some(Button::Side6),
            "right" => Some(Button::Right),
            "left" => Some(Button::Left),
            "side7" => Some(Button::Side7),
            "side8" => Some(Button::Side8),
            "middle" => Some(Button::Middle),
            "fire" => Some(Button::Fire),
            "side9" => Some(Button::Side9),
            "side10" => Some(Button::Side10),
            "side11" => Some(Button::Side11),
            "side12" => Some(Button::Side12),
            _ => None,
        }
    }

    pub fn kb_addr(self) -> u16 {
        ADDR_KB_BASE + (self as u16) * KB_STRIDE
    }
}

#[derive(Debug, Clone)]
pub enum EncodedAction {
    Modifier(u8),
    Key(u8),
    Raw([u8; 4]),
}

/// Host → device checksum: `(0x55 - sum(bytes[0..15])) & 0xFF`.
pub fn compute_checksum(pkt: &Packet) -> u8 {
    let sum: u16 = pkt[..PACKET_SIZE - 1].iter().map(|&b| b as u16).sum();
    (0x55u16.wrapping_sub(sum) & 0xFF) as u8
}

/// Device → host checksum: `(0x4C - sum(bytes[1..15])) & 0xFF` (excludes Report ID 0x09).
pub fn verify_reply_checksum(pkt: &Packet) -> bool {
    let sum: u16 = pkt[1..PACKET_SIZE - 1].iter().map(|&b| b as u16).sum();
    pkt[PACKET_SIZE - 1] == (0x4Cu16.wrapping_sub(sum) & 0xFF) as u8
}

pub fn ack_matches(req: &Packet, reply: &Packet) -> bool {
    reply[0] == REPORT_ID_IN
        && reply[1] == req[1]
        && reply[3] == req[3]
        && reply[4] == req[4]
}

pub fn build_status_packet() -> Packet {
    let mut p = [0u8; PACKET_SIZE];
    p[0] = REPORT_ID_OUT;
    p[1] = 0x03;
    p[16] = compute_checksum(&p);
    p
}

pub fn build_read_packet(addr: u16) -> Packet {
    let mut p = [0u8; PACKET_SIZE];
    p[0] = REPORT_ID_OUT;
    p[1] = 0x08;
    p[3] = (addr >> 8) as u8;
    p[4] = (addr & 0xFF) as u8;
    p[5] = 0x0a;
    p[16] = compute_checksum(&p);
    p
}

pub fn build_write_packet(addr: u16, data: &[u8]) -> Packet {
    assert!(data.len() <= 10, "write payload max 10 bytes");
    let mut p = [0u8; PACKET_SIZE];
    p[0] = REPORT_ID_OUT;
    p[1] = 0x07;
    p[3] = (addr >> 8) as u8;
    p[4] = (addr & 0xFF) as u8;
    p[5] = data.len() as u8;
    p[6..6 + data.len()].copy_from_slice(data);
    p[16] = compute_checksum(&p);
    p
}

pub fn build_commit_packet() -> Packet {
    let mut p = [0u8; PACKET_SIZE];
    p[0] = REPORT_ID_OUT;
    p[1] = 0x04;
    p[16] = compute_checksum(&p);
    p
}

pub fn build_modifier_kb_packet(btn: Button, mod_bit: u8) -> Packet {
    let addr = btn.kb_addr();
    let isum: u8 = 0x55u8
        .wrapping_sub(0x02)
        .wrapping_sub(0x80)
        .wrapping_sub(mod_bit)
        .wrapping_sub(0x40)
        .wrapping_sub(mod_bit);
    let payload = [0x02, 0x80, mod_bit, 0x00, 0x40, mod_bit, 0x00, isum];
    build_write_packet(addr, &payload)
}

pub fn build_key_kb_packet(btn: Button, hid_key: u8) -> Packet {
    let addr = btn.kb_addr();
    let isum: u8 = 0x55u8
        .wrapping_sub(0x02)
        .wrapping_sub(0x81)
        .wrapping_sub(hid_key)
        .wrapping_sub(0x41)
        .wrapping_sub(hid_key);
    let payload = [0x02, 0x81, hid_key, 0x00, 0x41, hid_key, 0x00, isum];
    build_write_packet(addr, &payload)
}

pub fn parse_action(spec: &str) -> Result<EncodedAction, String> {
    let s = spec.trim().to_ascii_lowercase();
    match s.as_str() {
        // Mouse & special
        "left" => Ok(EncodedAction::Raw([0x01, 0x01, 0x00, 0x53])),
        "right" => Ok(EncodedAction::Raw([0x01, 0x02, 0x00, 0x52])),
        "middle" => Ok(EncodedAction::Raw([0x01, 0x04, 0x00, 0x50])),
        "backward" => Ok(EncodedAction::Raw([0x01, 0x08, 0x00, 0x4c])),
        "forward" => Ok(EncodedAction::Raw([0x01, 0x10, 0x00, 0x44])),
        "dpi-cycle" => Ok(EncodedAction::Raw([0x02, 0x01, 0x00, 0x52])),
        "dpi+" => Ok(EncodedAction::Raw([0x02, 0x02, 0x00, 0x51])),
        "dpi-" => Ok(EncodedAction::Raw([0x02, 0x03, 0x00, 0x50])),
        "led_toggle" => Ok(EncodedAction::Raw([0x08, 0x00, 0x00, 0x4d])),
        "none" | "disable" => Ok(EncodedAction::Raw([0x00, 0x00, 0x00, 0x55])),
        "fire" => Ok(EncodedAction::Raw([0x04, 0x14, 0x03, 0x3a])),

        // macOS / USB HID Modifiers
        "control" | "ctrl" | "ctrl_l" => Ok(EncodedAction::Modifier(0x01)),
        "shift" | "shift_l" => Ok(EncodedAction::Modifier(0x02)),
        "option" | "opt" | "alt" | "alt_l" => Ok(EncodedAction::Modifier(0x04)),
        "command" | "cmd" | "super" | "meta" => Ok(EncodedAction::Modifier(0x08)),
        "control_r" | "ctrl_r" => Ok(EncodedAction::Modifier(0x10)),
        "shift_r" => Ok(EncodedAction::Modifier(0x20)),
        "option_r" | "alt_r" => Ok(EncodedAction::Modifier(0x40)),
        "command_r" | "cmd_r" | "super_r" => Ok(EncodedAction::Modifier(0x80)),

        // Letters a-z
        k if k.len() == 1 && k.as_bytes()[0].is_ascii_lowercase() => {
            Ok(EncodedAction::Key(0x04 + (k.as_bytes()[0] - b'a')))
        }
        // Numbers 1-9, 0
        "1" => Ok(EncodedAction::Key(0x1e)),
        "2" => Ok(EncodedAction::Key(0x1f)),
        "3" => Ok(EncodedAction::Key(0x20)),
        "4" => Ok(EncodedAction::Key(0x21)),
        "5" => Ok(EncodedAction::Key(0x22)),
        "6" => Ok(EncodedAction::Key(0x23)),
        "7" => Ok(EncodedAction::Key(0x24)),
        "8" => Ok(EncodedAction::Key(0x25)),
        "9" => Ok(EncodedAction::Key(0x26)),
        "0" => Ok(EncodedAction::Key(0x27)),

        // Common keys
        "enter" | "return" => Ok(EncodedAction::Key(0x28)),
        "esc" | "escape" => Ok(EncodedAction::Key(0x29)),
        "backspace" => Ok(EncodedAction::Key(0x2a)),
        "tab" => Ok(EncodedAction::Key(0x2b)),
        "space" => Ok(EncodedAction::Key(0x2c)),
        "minus" | "-" => Ok(EncodedAction::Key(0x2d)),
        "equal" | "=" => Ok(EncodedAction::Key(0x2e)),
        "lbracket" | "[" => Ok(EncodedAction::Key(0x2f)),
        "rbracket" | "]" => Ok(EncodedAction::Key(0x30)),
        "backslash" | "\\" => Ok(EncodedAction::Key(0x31)),
        "semicolon" | ";" => Ok(EncodedAction::Key(0x33)),
        "quote" | "'" => Ok(EncodedAction::Key(0x34)),
        "grave" | "`" => Ok(EncodedAction::Key(0x35)),
        "comma" | "," => Ok(EncodedAction::Key(0x36)),
        "dot" | "." => Ok(EncodedAction::Key(0x37)),
        "slash" | "/" => Ok(EncodedAction::Key(0x38)),
        "capslock" => Ok(EncodedAction::Key(0x39)),

        // Function keys
        "f1" => Ok(EncodedAction::Key(0x3a)),
        "f2" => Ok(EncodedAction::Key(0x3b)),
        "f3" => Ok(EncodedAction::Key(0x3c)),
        "f4" => Ok(EncodedAction::Key(0x3d)),
        "f5" => Ok(EncodedAction::Key(0x3e)),
        "f6" => Ok(EncodedAction::Key(0x3f)),
        "f7" => Ok(EncodedAction::Key(0x40)),
        "f8" => Ok(EncodedAction::Key(0x41)),
        "f9" => Ok(EncodedAction::Key(0x42)),
        "f10" => Ok(EncodedAction::Key(0x43)),
        "f11" => Ok(EncodedAction::Key(0x44)),
        "f12" => Ok(EncodedAction::Key(0x45)),
        "f13" => Ok(EncodedAction::Key(0x68)),
        "f14" => Ok(EncodedAction::Key(0x69)),
        "f15" => Ok(EncodedAction::Key(0x6a)),
        "f16" => Ok(EncodedAction::Key(0x6b)),
        "f17" => Ok(EncodedAction::Key(0x6c)),
        "f18" => Ok(EncodedAction::Key(0x6d)),
        "f19" => Ok(EncodedAction::Key(0x6e)),
        "f20" => Ok(EncodedAction::Key(0x6f)),

        // Navigation
        "home" => Ok(EncodedAction::Key(0x4a)),
        "pageup" => Ok(EncodedAction::Key(0x4b)),
        "delete" => Ok(EncodedAction::Key(0x4c)),
        "end" => Ok(EncodedAction::Key(0x4d)),
        "pagedown" => Ok(EncodedAction::Key(0x4e)),
        "arrow_right" => Ok(EncodedAction::Key(0x4f)),
        "arrow_left" => Ok(EncodedAction::Key(0x50)),
        "arrow_down" => Ok(EncodedAction::Key(0x51)),
        "arrow_up" => Ok(EncodedAction::Key(0x52)),

        // Numpad
        "numminus" => Ok(EncodedAction::Key(0x56)),
        "numplus" => Ok(EncodedAction::Key(0x57)),
        "numenter" => Ok(EncodedAction::Key(0x58)),
        "num1" => Ok(EncodedAction::Key(0x59)),
        "num2" => Ok(EncodedAction::Key(0x5a)),
        "num3" => Ok(EncodedAction::Key(0x5b)),
        "num4" => Ok(EncodedAction::Key(0x5c)),
        "num5" => Ok(EncodedAction::Key(0x5d)),
        "num6" => Ok(EncodedAction::Key(0x5e)),
        "num7" => Ok(EncodedAction::Key(0x5f)),
        "num8" => Ok(EncodedAction::Key(0x60)),
        "num9" => Ok(EncodedAction::Key(0x61)),
        "num0" => Ok(EncodedAction::Key(0x62)),

        other => Err(format!("Unsupported action '{other}'")),
    }
}

pub fn decode_mouse_action(ab: [u8; 4]) -> Option<String> {
    let s = match ab {
        [0x01, 0x01, 0x00, 0x53] => "left",
        [0x01, 0x02, 0x00, 0x52] => "right",
        [0x01, 0x04, 0x00, 0x50] => "middle",
        [0x01, 0x08, 0x00, 0x4c] => "backward",
        [0x01, 0x10, 0x00, 0x44] => "forward",
        [0x02, 0x01, 0x00, 0x52] => "dpi-cycle",
        [0x02, 0x02, 0x00, 0x51] => "dpi+",
        [0x02, 0x03, 0x00, 0x50] => "dpi-",
        [0x08, 0x00, 0x00, 0x4d] => "led_toggle",
        [0x00, 0x00, 0x00, 0x55] => "none",
        [0x04, _, _, _] => "fire",
        _ => return None,
    };
    Some(s.to_string())
}

pub fn decode_modifier_bit(bit: u8) -> &'static str {
    match bit {
        0x01 => "control",
        0x02 => "shift",
        0x04 => "option",
        0x08 => "command",
        0x10 => "control_r",
        0x20 => "shift_r",
        0x40 => "option_r",
        0x80 => "command_r",
        _ => "unknown",
    }
}

pub fn decode_hid_key(code: u8) -> String {
    match code {
        0x04..=0x1d => ((b'a' + (code - 0x04)) as char).to_string(),
        0x1e..=0x26 => ((b'1' + (code - 0x1e)) as char).to_string(),
        0x27 => "0".into(),
        0x28 => "enter".into(),
        0x29 => "esc".into(),
        0x2a => "backspace".into(),
        0x2b => "tab".into(),
        0x2c => "space".into(),
        0x2d => "minus".into(),
        0x2e => "equal".into(),
        0x2f => "lbracket".into(),
        0x30 => "rbracket".into(),
        0x31 => "backslash".into(),
        0x33 => "semicolon".into(),
        0x34 => "quote".into(),
        0x35 => "grave".into(),
        0x36 => "comma".into(),
        0x37 => "dot".into(),
        0x38 => "slash".into(),
        0x39 => "capslock".into(),
        0x3a..=0x45 => format!("f{}", code - 0x3a + 1),
        0x4a => "home".into(),
        0x4b => "pageup".into(),
        0x4c => "delete".into(),
        0x4d => "end".into(),
        0x4e => "pagedown".into(),
        0x4f => "arrow_right".into(),
        0x50 => "arrow_left".into(),
        0x51 => "arrow_down".into(),
        0x52 => "arrow_up".into(),
        0x56 => "numminus".into(),
        0x57 => "numplus".into(),
        0x58 => "numenter".into(),
        0x59..=0x61 => format!("num{}", code - 0x59 + 1),
        0x62 => "num0".into(),
        0x68..=0x6f => format!("f{}", code - 0x68 + 13),
        _ => format!("hid:0x{code:02x}"),
    }
}

pub fn decode_kb_list(buf: &[u8]) -> Option<String> {
    if buf.len() < 2 {
        return None;
    }
    let count = buf[0] as usize;
    if count == 0 || 1 + count * 3 + 1 > buf.len() {
        return None;
    }
    let sum: u16 = buf[..1 + count * 3].iter().map(|&b| b as u16).sum();
    let expected_inner = (0x55u16.wrapping_sub(sum) & 0xFF) as u8;
    if buf[1 + count * 3] != expected_inner {
        return None;
    }

    let mut parts = Vec::new();
    for i in 0..count {
        let ev_type = buf[1 + i * 3];
        let val = buf[2 + i * 3];
        match ev_type {
            0x80 => {
                for bit_idx in 0..8 {
                    let bit = 1u8 << bit_idx;
                    if (val & bit) != 0 {
                        parts.push(decode_modifier_bit(bit).to_string());
                    }
                }
            }
            0x81 => parts.push(decode_hid_key(val)),
            0x82 => parts.push(format!("media:0x{val:02x}")),
            0x40 | 0x41 | 0x42 => {}
            _ => return None,
        }
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("+"))
    }
}

pub fn factory_default_states() -> Vec<ButtonState> {
    ALL_BUTTONS
        .iter()
        .map(|&btn| {
            let def = btn.factory_default().to_string();
            ButtonState {
                id: btn.id().to_string(),
                label: btn.label().to_string(),
                action: def.clone(),
                default_action: def,
                is_modified: false,
            }
        })
        .collect()
}
