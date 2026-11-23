mod device;
mod protocol;

use std::collections::BTreeMap;
use std::env;

use device::M913Device;
use protocol::{
    build_commit_packet, build_key_kb_packet, build_modifier_kb_packet, build_read_packet,
    build_status_packet, build_write_packet, decode_kb_list, decode_mouse_action, hex_packet,
    ALL_BUTTONS, ADDR_BTN_BASE, BTN_STRIDE, Button, KB_LIST_MARKER,
};

fn read_memory(dev: &M913Device, addrs: &[u16]) -> Result<BTreeMap<u16, u8>, String> {
    let mut mem = BTreeMap::new();
    for &addr in addrs {
        let req = build_read_packet(addr);
        let reply = dev.transact(&req)?;
        for i in 0..10 {
            mem.insert(addr + i as u16, reply[6 + i]);
        }
    }
    Ok(mem)
}

fn read_buttons(dev: &M913Device) -> Result<(), String> {
    // 1. Ping status first to confirm wireless link is awake
    let status_req = build_status_packet();
    let status_rep = dev.transact(&status_req)?;
    println!("Status reply: {}", hex_packet(&status_rep));

    // 2. Read button action table 0x0060..0x009F (64 bytes in 10-byte chunks)
    let btn_addrs = [0x0060, 0x006a, 0x0074, 0x007e, 0x0088, 0x0092, 0x009c];
    let mut mem = read_memory(dev, &btn_addrs)?;

    // 3. For any button pointing at KB_LIST_MARKER, read its keyboard event list (20 bytes)
    let mut kb_addrs = Vec::new();
    for btn in ALL_BUTTONS {
        let base = ADDR_BTN_BASE + (btn as u16) * BTN_STRIDE;
        let ab = [mem[&base], mem[&(base + 1)], mem[&(base + 2)], mem[&(base + 3)]];
        if ab == KB_LIST_MARKER {
            let kb_base = btn.kb_addr();
            kb_addrs.push(kb_base);
            kb_addrs.push(kb_base + 0x0a);
        }
    }
    if !kb_addrs.is_empty() {
        let kb_mem = read_memory(dev, &kb_addrs)?;
        mem.extend(kb_mem);
    }

    println!("\nCurrent onboard button mappings:");
    println!("{:<8} {:<14} {}", "Button", "Raw Action", "Decoded");
    println!("{:-<48}", "");
    for btn in ALL_BUTTONS {
        let base = ADDR_BTN_BASE + (btn as u16) * BTN_STRIDE;
        let ab = [mem[&base], mem[&(base + 1)], mem[&(base + 2)], mem[&(base + 3)]];
        let decoded = if ab == KB_LIST_MARKER {
            let kb_base = btn.kb_addr();
            let kb_bytes: Vec<u8> = (0..20).map(|i| mem[&(kb_base + i)]).collect();
            decode_kb_list(&kb_bytes)
                .map(|s| format!("key: {s}"))
                .unwrap_or_else(|| format!("kb raw: {}", hex_packet(&kb_bytes[..10])))
        } else if let Some(m) = decode_mouse_action(ab) {
            format!("mouse: {m}")
        } else {
            "unknown".to_string()
        };
        println!(
            "{:<8} {:02x} {:02x} {:02x} {:02x}    {}",
            btn.name(),
            ab[0],
            ab[1],
            ab[2],
            ab[3],
            decoded
        );
    }

    Ok(())
}

fn parse_target_action(spec: &str) -> Result<TargetAction, String> {
    match spec.to_ascii_lowercase().as_str() {
        "ctrl" | "control" | "ctrl_l" => Ok(TargetAction::Modifier(0x01)),
        "shift" | "shift_l" => Ok(TargetAction::Modifier(0x02)),
        "option" | "opt" | "alt" | "alt_l" => Ok(TargetAction::Modifier(0x04)),
        "command" | "cmd" | "super" | "meta" => Ok(TargetAction::Modifier(0x08)),
        "minus" | "-" => Ok(TargetAction::Key(0x2d)),
        "equal" | "=" => Ok(TargetAction::Key(0x2e)),
        "numminus" => Ok(TargetAction::Key(0x56)),
        "numplus" => Ok(TargetAction::Key(0x57)),
        "1" => Ok(TargetAction::Key(0x1e)),
        "2" => Ok(TargetAction::Key(0x1f)),
        "3" => Ok(TargetAction::Key(0x20)),
        "4" => Ok(TargetAction::Key(0x21)),
        "5" => Ok(TargetAction::Key(0x22)),
        "6" => Ok(TargetAction::Key(0x23)),
        "7" => Ok(TargetAction::Key(0x24)),
        "8" => Ok(TargetAction::Key(0x25)),
        "9" => Ok(TargetAction::Key(0x26)),
        "0" => Ok(TargetAction::Key(0x27)),
        "none" | "disable" => Ok(TargetAction::Raw([0x00, 0x00, 0x00, 0x55])),
        "left" => Ok(TargetAction::Raw([0x01, 0x01, 0x00, 0x53])),
        "right" => Ok(TargetAction::Raw([0x01, 0x02, 0x00, 0x52])),
        "middle" => Ok(TargetAction::Raw([0x01, 0x04, 0x00, 0x50])),
        "fire" => Ok(TargetAction::Raw([0x04, 0x3a, 0x03, 0x14])),
        s if s.starts_with("hid:0x") => {
            let code = u8::from_str_radix(&s[6..], 16)
                .map_err(|_| format!("Invalid hex HID keycode '{s}'"))?;
            Ok(TargetAction::Key(code))
        }
        other => Err(format!("Unsupported action '{other}' in PoC")),
    }
}

enum TargetAction {
    Modifier(u8),
    Key(u8),
    Raw([u8; 4]),
}

fn remap_buttons(dev: &M913Device, pairs: &[(Button, TargetAction)]) -> Result<(), String> {
    // 1. Read the current 64-byte button table so untouched buttons stay 100% intact!
    let btn_addrs = [0x0060, 0x006a, 0x0074, 0x007e, 0x0088, 0x0092, 0x009c];
    let mem = read_memory(dev, &btn_addrs)?;

    let mut table = [[0u8; 4]; 16];
    for (idx, slot) in table.iter_mut().enumerate() {
        let base = ADDR_BTN_BASE + (idx as u16) * BTN_STRIDE;
        *slot = [mem[&base], mem[&(base + 1)], mem[&(base + 2)], mem[&(base + 3)]];
    }

    // 2. Write keyboard sub-packets for any keyboard/modifier remaps and update table
    for (btn, action) in pairs {
        let idx = *btn as usize;
        match action {
            TargetAction::Modifier(mod_bit) => {
                let pkt = build_modifier_kb_packet(*btn, *mod_bit);
                println!("Writing modifier KB packet for {}: {}", btn.name(), hex_packet(&pkt));
                dev.transact(&pkt)?;
                table[idx] = KB_LIST_MARKER;
            }
            TargetAction::Key(hid_key) => {
                let pkt = build_key_kb_packet(*btn, *hid_key);
                println!("Writing key KB packet for {}: {}", btn.name(), hex_packet(&pkt));
                dev.transact(&pkt)?;
                table[idx] = KB_LIST_MARKER;
            }
            TargetAction::Raw(ab) => {
                table[idx] = *ab;
            }
        }
    }

    // 3. Write the 8 button-table packets (2 buttons = 8 bytes per packet)
    for pair_idx in 0..8 {
        let addr = ADDR_BTN_BASE + (pair_idx as u16) * 8;
        let mut payload = [0u8; 8];
        payload[0..4].copy_from_slice(&table[pair_idx * 2]);
        payload[4..8].copy_from_slice(&table[pair_idx * 2 + 1]);
        let pkt = build_write_packet(addr, &payload);
        dev.transact(&pkt)?;
    }

    // 4. Commit to onboard flash
    let commit = build_commit_packet();
    println!("Committing to onboard memory: {}", hex_packet(&commit));
    dev.transact(&commit)?;
    println!("Done! Settings committed to mouse.");

    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let cmd = args.first().map(|s| s.as_str()).unwrap_or("read");

    let res = match cmd {
        "list" => M913Device::list_interfaces(),
        "read" => M913Device::open().and_then(|d| read_buttons(&d)),
        "watch" => M913Device::open().and_then(|d| d.watch()),
        "remap" => {
            if args.len() < 2 {
                Err("Usage: m913-mac remap <btn>=<action> [<btn>=<action> ...]\nExample: m913-mac remap side11=option side12=control".into())
            } else {
                let mut pairs = Vec::new();
                for arg in &args[1..] {
                    let Some((b_str, a_str)) = arg.split_once('=') else {
                        eprintln!("Invalid mapping '{arg}', expected button=action");
                        std::process::exit(1);
                    };
                    let btn = Button::parse(b_str)
                        .ok_or_else(|| format!("Unknown button '{b_str}'"))
                        .unwrap_or_else(|e| {
                            eprintln!("{e}");
                            std::process::exit(1);
                        });
                    let act = parse_target_action(a_str).unwrap_or_else(|e| {
                        eprintln!("{e}");
                        std::process::exit(1);
                    });
                    pairs.push((btn, act));
                }
                M913Device::open().and_then(|d| remap_buttons(&d, &pairs))
            }
        }
        other => Err(format!("Unknown command '{other}'. Use: list | read | remap")),
    };

    if let Err(e) = res {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
