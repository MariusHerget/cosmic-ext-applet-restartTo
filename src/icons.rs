// SPDX-License-Identifier: MPL-2.0

use cosmic::widget::icon;
use cosmic::Core;
use fontawesome_free_pack;
use simpleicons_rs::slug_colored;

/// Convert SVG string to 'static byte slice for use with from_svg_bytes
/// This is safe because these icons are processed once and reused
fn svg_to_static_bytes(svg: &str) -> &'static [u8] {
    Box::leak(svg.to_string().into_bytes().into_boxed_slice())
}

/// Convert cosmic theme Alpha<Rgb, f32> color to hex string format (#RRGGBB)
fn color_to_hex(color: &cosmic::cosmic_theme::palette::Alpha<cosmic::cosmic_theme::palette::rgb::Rgb, f32>) -> String {
    // Extract RGB values from the Alpha wrapper
    let rgb = &color.color;
    let r = (rgb.red * 255.0) as u8;
    let g = (rgb.green * 255.0) as u8;
    let b = (rgb.blue * 255.0) as u8;
    format!("#{:02X}{:02X}{:02X}", r, g, b)
}

/// Get the accent color from the cosmic theme and convert it to hex
fn get_accent_color_hex(_core: &Core) -> String {
    // Get the active theme and extract the accent color
    let theme = cosmic::theme::active();
    let accent_color = theme.cosmic().accent.base;
    color_to_hex(&accent_color)
}

/// Get a subtle/muted color from the cosmic theme (secondary color) and convert it to hex
/// This is useful for icons that should be less prominent than accent-colored icons
#[allow(dead_code)]
pub fn get_subtle_color_hex(_core: &Core) -> String {
    // Get the active theme and extract the secondary color (more subtle than accent)
    let theme = cosmic::theme::active();
    let secondary_color = theme.cosmic().secondary.base;
    color_to_hex(&secondary_color)
}


/// Process Font Awesome SVG to use accent color for theme integration
/// Uses simple string replacement (no regex) to replace fill attributes with accent color
fn make_fontawesome_theme_aware(svg: &str, accent_hex: &str) -> &'static [u8] {
    // Font Awesome SVGs may have fill="#..." attributes
    // Strategy: Replace fill="#..." patterns and fill="currentColor" with accent color, preserving only fill="none"
    
    let mut result = svg.to_string();
    
    // Split by fill=" to process each fill attribute
    let parts: Vec<&str> = result.split("fill=\"").collect();
    if parts.len() > 1 {
        let mut modified = String::new();
        modified.push_str(parts[0]); // Part before first fill="
        
        for part in parts.iter().skip(1) {
            // Find the closing quote
            if let Some(quote_pos) = part.find('"') {
                let fill_value = &part[..quote_pos];
                let rest = &part[quote_pos+1..];
                
                // Preserve fill="none" only (currentColor should be replaced with accent color)
                if fill_value == "none" {
                    modified.push_str("fill=\"");
                    modified.push_str(fill_value);
                    modified.push('"');
                    modified.push_str(rest);
                } else if fill_value == "currentColor" || fill_value.starts_with('#') || 
                          fill_value == "black" || fill_value == "white" ||
                          fill_value == "#000" || fill_value == "#fff" ||
                          fill_value == "#000000" || fill_value == "#ffffff" {
                    // Replace with accent color
                    modified.push_str("fill=\"");
                    modified.push_str(accent_hex);
                    modified.push('"');
                    modified.push_str(rest);
                } else {
                    // Unknown fill value, keep as-is but replace with accent color for safety
                    modified.push_str("fill=\"");
                    modified.push_str(accent_hex);
                    modified.push('"');
                    modified.push_str(rest);
                }
            } else {
                // No closing quote found, keep as-is
                modified.push_str("fill=\"");
                modified.push_str(part);
            }
        }
        result = modified;
    }
    
    svg_to_static_bytes(&result)
}

/// Get the icon handle for a boot entry based on its description
/// Returns an icon handle that can be used with cosmic widgets
pub fn get_boot_entry_icon(description: &str, core: &Core) -> icon::Handle {
    let desc_lower = description.to_lowercase();
    let accent_hex = get_accent_color_hex(core);

    // Check for Windows entries
    if desc_lower.contains("windows") || desc_lower.contains("win") || desc_lower.contains("microsoft") {
        // Use Font Awesome Windows icon SVG with theme accent color
        let svg_bytes = make_fontawesome_theme_aware(fontawesome_free_pack::BRANDS_WINDOWS.svg, &accent_hex);
        return icon::from_svg_bytes(svg_bytes);
    }

    // Check for Linux distributions using simpleicons
    // Use slug_colored() with accent color hex for theme-aware icons
    if desc_lower.contains("pop!_os") || desc_lower.contains("pop os") || desc_lower.contains("popos") {
        if let Some(icon) = slug_colored("popos", &accent_hex) {
            let svg_bytes = svg_to_static_bytes(icon.svg);
            return icon::from_svg_bytes(svg_bytes);
        }
    }
    if desc_lower.contains("ubuntu") {
        if let Some(icon) = slug_colored("ubuntu", &accent_hex) {
            let svg_bytes = svg_to_static_bytes(icon.svg);
            return icon::from_svg_bytes(svg_bytes);
        }
    }
    if desc_lower.contains("fedora") {
        if let Some(icon) = slug_colored("fedora", &accent_hex) {
            let svg_bytes = svg_to_static_bytes(icon.svg);
            return icon::from_svg_bytes(svg_bytes);
        }
    }
    if desc_lower.contains("archlinux") || desc_lower.contains("arch linux") {
        if let Some(icon) = slug_colored("archlinux", &accent_hex) {
            let svg_bytes = svg_to_static_bytes(icon.svg);
            return icon::from_svg_bytes(svg_bytes);
        }
    }
    if desc_lower.contains("debian") {
        if let Some(icon) = slug_colored("debian", &accent_hex) {
            let svg_bytes = svg_to_static_bytes(icon.svg);
            return icon::from_svg_bytes(svg_bytes);
        }
    }
    if desc_lower.contains("opensuse") || desc_lower.contains("suse") {
        if let Some(icon) = slug_colored("opensuse", &accent_hex) {
            let svg_bytes = svg_to_static_bytes(icon.svg);
            return icon::from_svg_bytes(svg_bytes);
        }
    }
    if desc_lower.contains("gentoo") {
        if let Some(icon) = slug_colored("gentoo", &accent_hex) {
            let svg_bytes = svg_to_static_bytes(icon.svg);
            return icon::from_svg_bytes(svg_bytes);
        }
    }
    if desc_lower.contains("manjaro") {
        if let Some(icon) = slug_colored("manjaro", &accent_hex) {
            let svg_bytes = svg_to_static_bytes(icon.svg);
            return icon::from_svg_bytes(svg_bytes);
        }
    }
    if desc_lower.contains("mint") || desc_lower.contains("linuxmint") {
        if let Some(icon) = slug_colored("linuxmint", &accent_hex) {
            let svg_bytes = svg_to_static_bytes(icon.svg);
            return icon::from_svg_bytes(svg_bytes);
        }
    }
    if desc_lower.contains("elementary") {
        if let Some(icon) = slug_colored("elementary", &accent_hex) {
            let svg_bytes = svg_to_static_bytes(icon.svg);
            return icon::from_svg_bytes(svg_bytes);
        }
    }
    if desc_lower.contains("kali") || desc_lower.contains("kalilinux") {
        if let Some(icon) = slug_colored("kalilinux", &accent_hex) {
            let svg_bytes = svg_to_static_bytes(icon.svg);
            return icon::from_svg_bytes(svg_bytes);
        }
    }
    if desc_lower.contains("centos") {
        if let Some(icon) = slug_colored("centos", &accent_hex) {
            let svg_bytes = svg_to_static_bytes(icon.svg);
            return icon::from_svg_bytes(svg_bytes);
        }
    }
    if desc_lower.contains("rhel") || desc_lower.contains("red hat") || desc_lower.contains("redhat") {
        if let Some(icon) = slug_colored("redhat", &accent_hex) {
            let svg_bytes = svg_to_static_bytes(icon.svg);
            return icon::from_svg_bytes(svg_bytes);
        }
    }
    if desc_lower.contains("rocky") {
        if let Some(icon) = slug_colored("rockylinux", &accent_hex) {
            let svg_bytes = svg_to_static_bytes(icon.svg);
            return icon::from_svg_bytes(svg_bytes);
        }
    }
    if desc_lower.contains("alpine") {
        if let Some(icon) = slug_colored("alpinelinux", &accent_hex) {
            let svg_bytes = svg_to_static_bytes(icon.svg);
            return icon::from_svg_bytes(svg_bytes);
        }
    }
    if desc_lower.contains("nixos") {
        if let Some(icon) = slug_colored("nixos", &accent_hex) {
            let svg_bytes = svg_to_static_bytes(icon.svg);
            return icon::from_svg_bytes(svg_bytes);
        }
    }
    if desc_lower.contains("void") {
        if let Some(icon) = slug_colored("voidlinux", &accent_hex) {
            let svg_bytes = svg_to_static_bytes(icon.svg);
            return icon::from_svg_bytes(svg_bytes);
        }
    }
    if desc_lower.contains("endeavour") {
        if let Some(icon) = slug_colored("endeavouros", &accent_hex) {
            let svg_bytes = svg_to_static_bytes(icon.svg);
            return icon::from_svg_bytes(svg_bytes);
        }
    }

    // Check for hardware/storage entries using cosmic-icons (system icon theme)
    if desc_lower.contains("network") || desc_lower.contains("pxe") || desc_lower.contains("netboot") || desc_lower.contains("nic") || desc_lower.contains("ethernet") {
        return icon::from_name("network-wired-symbolic").into();
    }
    if desc_lower.contains("ssd") || desc_lower.contains("solid state") || desc_lower.contains("nvme") || desc_lower.contains("m.2") {
        return icon::from_name("drive-harddisk-solidstate-symbolic").into();
    }
    if desc_lower.contains("hdd") || desc_lower.contains("hard disk") || desc_lower.contains("harddisk") {
        return icon::from_name("drive-harddisk-solidstate-symbolic").into();
    }
    if desc_lower.contains("usb") || desc_lower.contains("removable") || desc_lower.contains("usb-storage") {
        return icon::from_name("drive-harddisk-usb-symbolic").into();
    }
    if desc_lower.contains("cd") || desc_lower.contains("dvd") || desc_lower.contains("optical") {
        return icon::from_name("media-optical-symbolic").into();
    }

    // Default fallback: Linux icon from simpleicons with theme accent color
    if let Some(icon) = slug_colored("linux", &accent_hex) {
        let svg_bytes = svg_to_static_bytes(icon.svg);
        return icon::from_svg_bytes(svg_bytes);
    }
    
    // Ultimate fallback: system reboot icon
    icon::from_name("system-reboot-symbolic").into()
}
