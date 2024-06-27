use alacritty_terminal::{
    term::color::Colors,
    vte::ansi::{NamedColor, Rgb},
};

/// Generate a terminal color table
pub fn colors() -> Colors {
    let mut colors = Colors::default();

    fill_named(&mut colors);
    fill_cube(&mut colors);
    fill_gray_ramp(&mut colors);

    colors
}

/// Fill named terminal colors with the solarized dark theme
fn fill_named(colors: &mut Colors) {
    colors[NamedColor::Black as usize] = Some("#073642".parse().unwrap());
    colors[NamedColor::Black] = Some("#073642".parse().unwrap());
    colors[NamedColor::Red] = Some("#dc322f".parse().unwrap());
    colors[NamedColor::Green] = Some("#859900".parse().unwrap());
    colors[NamedColor::Yellow] = Some("#b58900".parse().unwrap());
    colors[NamedColor::Blue] = Some("#268bd2".parse().unwrap());
    colors[NamedColor::Magenta] = Some("#d33682".parse().unwrap());
    colors[NamedColor::Cyan] = Some("#2aa198".parse().unwrap());
    colors[NamedColor::White] = Some("#eee8d5".parse().unwrap());
    colors[NamedColor::BrightBlack] = Some("#002b36".parse().unwrap());
    colors[NamedColor::BrightRed] = Some("#cb4b16".parse().unwrap());
    colors[NamedColor::BrightGreen] = Some("#586e75".parse().unwrap());
    colors[NamedColor::BrightYellow] = Some("#657b83".parse().unwrap());
    colors[NamedColor::BrightBlue] = Some("#839496".parse().unwrap());
    colors[NamedColor::BrightMagenta] = Some("#6c71c4".parse().unwrap());
    colors[NamedColor::BrightCyan] = Some("#93a1a1".parse().unwrap());
    colors[NamedColor::BrightWhite] = Some("#fdf6e3".parse().unwrap());
    colors[NamedColor::Foreground] = Some("#839496".parse().unwrap());
    colors[NamedColor::Background] = Some("#002b36".parse().unwrap());
    colors[NamedColor::Cursor] = Some("#839496".parse().unwrap());
    colors[NamedColor::DimBlack] = Some("#073642".parse().unwrap());
    colors[NamedColor::DimRed] = Some("#dc322f".parse().unwrap());
    colors[NamedColor::DimGreen] = Some("#859900".parse().unwrap());
    colors[NamedColor::DimYellow] = Some("#b58900".parse().unwrap());
    colors[NamedColor::DimBlue] = Some("#268bd2".parse().unwrap());
    colors[NamedColor::DimMagenta] = Some("#d33682".parse().unwrap());
    colors[NamedColor::DimCyan] = Some("#2aa198".parse().unwrap());
    colors[NamedColor::DimWhite] = Some("#eee8d5".parse().unwrap());
    colors[NamedColor::DimForeground] = Some("#839496".parse().unwrap());
    colors[NamedColor::BrightForeground] = Some("#839496".parse().unwrap());
}

fn fill_cube(colors: &mut Colors) {
    // adapted from: https://github.com/alacritty/alacritty/blob/da554e41f3a91ed6cc5db66b23bf65c58529db83/alacritty/src/display/color.rs#L91-L115
    let mut index = 16usize;

    // Build colors.
    for r in 0..6 {
        for g in 0..6 {
            for b in 0..6 {
                // Override colors 16..232 with the config (if present).
                colors[index] = Some(Rgb {
                    r: if r == 0 { 0 } else { r * 40 + 55 },
                    g: if g == 0 { 0 } else { g * 40 + 55 },
                    b: if b == 0 { 0 } else { b * 40 + 55 },
                });
                index += 1;
            }
        }
    }

    debug_assert!(index == 232);
}

fn fill_gray_ramp(colors: &mut Colors) {
    // adapted from: https://github.com/alacritty/alacritty/blob/da554e41f3a91ed6cc5db66b23bf65c58529db83/alacritty/src/display/color.rs#L118-L139
    let mut index: usize = 232;

    // Build colors.
    for i in 0..24 {
        let value = i * 10 + 8;
        colors[index] = Some(Rgb {
            r: value,
            g: value,
            b: value,
        });
        index += 1;
    }

    debug_assert!(index == 256);
}