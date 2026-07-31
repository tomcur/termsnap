use rio_vt::config::colors::{
    term::{TermColors, COUNT as COLOR_COUNT},
    AnsiColor, ColorRgb, NamedColor,
};

use std::collections::HashMap;

use crate::{Rgb, Screen};

pub(crate) struct Colors {
    colors: [Option<Rgb>; COLOR_COUNT],
}

impl Colors {
    pub fn to_rgb(&self, color: AnsiColor) -> Rgb {
        match color {
            AnsiColor::Named(named_color) => {
                self.colors[named_color as usize].expect("all colors should be defined")
            }
            AnsiColor::Indexed(idx) => {
                self.colors[usize::from(idx)].expect("all colors should be defined")
            }
            AnsiColor::Spec(rgb) => Rgb {
                r: rgb.r,
                g: rgb.g,
                b: rgb.b,
            },
        }
    }
}

impl Default for Colors {
    /// Generate a terminal color table
    fn default() -> Colors {
        let mut colors = [None; COLOR_COUNT];

        fill_named(&mut colors);
        fill_cube(&mut colors);
        fill_gray_ramp(&mut colors);

        Colors { colors }
    }
}

impl Colors {
    /// Overlay colors in `src` on `self`.
    ///
    /// This writes every `Some` entry in `src` into the corresponding slot of `self`. Entries where
    /// `src` is `None` are untouched. The terminal stores palette entries as f32 RGBA arrays; they
    /// are converted back to 8-bit sRGB here.
    pub(crate) fn overlay(&mut self, src: &TermColors) {
        for (idx, slot) in self.colors.iter_mut().enumerate() {
            if let Some(arr) = src[idx] {
                let ColorRgb { r, g, b } = ColorRgb::from_color_arr(arr);
                *slot = Some(Rgb { r, g, b });
            }
        }
    }
}

/// Parse a `#rrggbb` hex color.
fn hex(s: &str) -> Rgb {
    let v = u32::from_str_radix(s.trim_start_matches('#'), 16).expect("valid hex color");
    Rgb {
        r: (v >> 16) as u8,
        g: (v >> 8) as u8,
        b: v as u8,
    }
}

/// Fill named terminal colors with the solarized dark theme
fn fill_named(colors: &mut [Option<Rgb>; COLOR_COUNT]) {
    colors[NamedColor::Black as usize] = Some(hex("#073642"));
    colors[NamedColor::Red as usize] = Some(hex("#dc322f"));
    colors[NamedColor::Green as usize] = Some(hex("#859900"));
    colors[NamedColor::Yellow as usize] = Some(hex("#b58900"));
    colors[NamedColor::Blue as usize] = Some(hex("#268bd2"));
    colors[NamedColor::Magenta as usize] = Some(hex("#d33682"));
    colors[NamedColor::Cyan as usize] = Some(hex("#2aa198"));
    colors[NamedColor::White as usize] = Some(hex("#eee8d5"));
    colors[NamedColor::LightBlack as usize] = Some(hex("#002b36"));
    colors[NamedColor::LightRed as usize] = Some(hex("#cb4b16"));
    colors[NamedColor::LightGreen as usize] = Some(hex("#586e75"));
    colors[NamedColor::LightYellow as usize] = Some(hex("#657b83"));
    colors[NamedColor::LightBlue as usize] = Some(hex("#839496"));
    colors[NamedColor::LightMagenta as usize] = Some(hex("#6c71c4"));
    colors[NamedColor::LightCyan as usize] = Some(hex("#93a1a1"));
    colors[NamedColor::LightWhite as usize] = Some(hex("#fdf6e3"));
    colors[NamedColor::Foreground as usize] = Some(hex("#839496"));
    colors[NamedColor::Background as usize] = Some(hex("#002b36"));
    colors[NamedColor::Cursor as usize] = Some(hex("#839496"));
    colors[NamedColor::DimBlack as usize] = Some(hex("#073642"));
    colors[NamedColor::DimRed as usize] = Some(hex("#dc322f"));
    colors[NamedColor::DimGreen as usize] = Some(hex("#859900"));
    colors[NamedColor::DimYellow as usize] = Some(hex("#b58900"));
    colors[NamedColor::DimBlue as usize] = Some(hex("#268bd2"));
    colors[NamedColor::DimMagenta as usize] = Some(hex("#d33682"));
    colors[NamedColor::DimCyan as usize] = Some(hex("#2aa198"));
    colors[NamedColor::DimWhite as usize] = Some(hex("#eee8d5"));
    colors[NamedColor::DimForeground as usize] = Some(hex("#839496"));
    colors[NamedColor::LightForeground as usize] = Some(hex("#839496"));
}

fn fill_cube(colors: &mut [Option<Rgb>; COLOR_COUNT]) {
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

fn fill_gray_ramp(colors: &mut [Option<Rgb>; COLOR_COUNT]) {
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

pub(crate) fn most_common_color(screen: &Screen) -> Rgb {
    use std::hash::{Hash, Hasher};

    #[derive(PartialEq, Eq, Copy, Clone)]
    struct Rgb_(Rgb);

    impl Hash for Rgb_ {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            state.write_u32(
                (u32::from(self.0.r) << 16) + (u32::from(self.0.g) << 8) + u32::from(self.0.b),
            );
        }
    }

    #[derive(Default)]
    struct NoHashHasher(u64);

    impl Hasher for NoHashHasher {
        fn finish(&self) -> u64 {
            self.0
        }

        fn write(&mut self, bytes: &[u8]) {
            for byte in bytes {
                self.0 <<= 8;
                self.0 += u64::from(*byte);
            }
        }
    }

    let mut counts = HashMap::<Rgb_, u32, _>::with_capacity_and_hasher(
        16,
        std::hash::BuildHasherDefault::<NoHashHasher>::default(),
    );

    for idx in 0..screen.lines() * screen.columns() {
        let cell = &screen.cells[usize::from(idx)];
        let bg = &cell.bg;

        *counts.entry(Rgb_(*bg)).or_insert(0) += 1;
    }

    counts
        .iter()
        .max_by_key(|(_, count)| *count)
        .map(|(k, _)| k.0)
        // counts can be empty for 0x0 screens
        .unwrap_or(Rgb { r: 0, g: 0, b: 0 })
}
