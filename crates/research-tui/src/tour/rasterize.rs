//! Buffer → PNG rasterizer + buffer → text serializer.
//!
//! The PNG path uses `ab_glyph` with a vendored IBM Plex Mono Regular font.
//! The text path is style-free — used by snapshot tests for diff-friendly
//! comparison.

use std::path::Path;

use ab_glyph::{Font, FontRef, PxScale, ScaleFont};
use image::{Rgba, RgbaImage};
use ratatui::buffer::Buffer;
use ratatui::style::{Color, Modifier};

use crate::palette;

const FONT_REGULAR: &[u8] = include_bytes!("../../assets/IBMPlexMono-Regular.ttf");
const FONT_BOLD: &[u8] = include_bytes!("../../assets/IBMPlexMono-Bold.ttf");
const FALLBACK_REGULAR: &[u8] = include_bytes!("../../assets/DejaVuSansMono.ttf");
const FALLBACK_BOLD: &[u8] = include_bytes!("../../assets/DejaVuSansMono-Bold.ttf");

const CELL_W: u32 = 9;
const CELL_H: u32 = 18;
const FONT_PX: f32 = 14.5;
const CENTER_X: u32 = 4;
const CENTER_Y: u32 = 8;

/// Pure text dump of a buffer — for snapshot tests.
pub fn buffer_to_text(buffer: &Buffer) -> String {
    let area = buffer.area;
    let mut out = String::with_capacity((area.width as usize + 1) * area.height as usize);
    for y in area.y..area.y + area.height {
        let mut line = String::new();
        for x in area.x..area.x + area.width {
            let cell = &buffer[(x, y)];
            line.push_str(cell.symbol());
        }
        out.push_str(line.trim_end());
        out.push('\n');
    }
    out
}

/// Render a buffer to a PNG file.
pub fn render_buffer_to_png(buffer: &Buffer, path: &Path) -> std::io::Result<()> {
    let area = buffer.area;
    let img_w = area.width as u32 * CELL_W;
    let img_h = area.height as u32 * CELL_H;
    let mut img = RgbaImage::from_pixel(img_w, img_h, color_to_rgba(palette::DEFAULT_BG));

    let regular = FontRef::try_from_slice(FONT_REGULAR)
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "regular font"))?;
    let bold = FontRef::try_from_slice(FONT_BOLD)
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "bold font"))?;
    let fallback_regular = FontRef::try_from_slice(FALLBACK_REGULAR)
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "fallback regular"))?;
    let fallback_bold = FontRef::try_from_slice(FALLBACK_BOLD)
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "fallback bold"))?;

    for y in area.y..area.y + area.height {
        for x in area.x..area.x + area.width {
            let cell = &buffer[(x, y)];
            let modifiers = cell.modifier;
            let mut fg = resolve(cell.fg, palette::DEFAULT_FG);
            let mut bg = resolve(cell.bg, palette::DEFAULT_BG);
            if modifiers.contains(Modifier::REVERSED) {
                std::mem::swap(&mut fg, &mut bg);
            }
            let px = (x as u32 - area.x as u32) * CELL_W;
            let py = (y as u32 - area.y as u32) * CELL_H;
            paint_rect(&mut img, px, py, CELL_W, CELL_H, color_to_rgba(bg));
            let symbol = cell.symbol();
            if !symbol.is_empty() && symbol != " " {
                let drawn = single_char(symbol)
                    .map(|ch| try_draw_box_drawing(&mut img, ch, px, py, color_to_rgba(fg)))
                    .unwrap_or(false);
                if !drawn {
                    let primary = if modifiers.contains(Modifier::BOLD) {
                        &bold
                    } else {
                        &regular
                    };
                    let fallback = if modifiers.contains(Modifier::BOLD) {
                        &fallback_bold
                    } else {
                        &fallback_regular
                    };
                    draw_glyph(
                        &mut img,
                        primary,
                        fallback,
                        symbol,
                        px,
                        py,
                        color_to_rgba(fg),
                    );
                }
            }
            if modifiers.contains(Modifier::UNDERLINED) {
                let uy = py + CELL_H - 2;
                paint_rect(&mut img, px, uy, CELL_W, 1, color_to_rgba(fg));
            }
        }
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    img.save(path).map_err(std::io::Error::other)?;
    Ok(())
}

fn paint_rect(img: &mut RgbaImage, px: u32, py: u32, w: u32, h: u32, color: Rgba<u8>) {
    for j in 0..h {
        for i in 0..w {
            let x = px + i;
            let y = py + j;
            if x < img.width() && y < img.height() {
                img.put_pixel(x, y, color);
            }
        }
    }
}

fn draw_glyph(
    img: &mut RgbaImage,
    primary: &FontRef<'_>,
    fallback: &FontRef<'_>,
    text: &str,
    px: u32,
    py: u32,
    fg: Rgba<u8>,
) {
    let scale = PxScale::from(FONT_PX);
    let primary_scaled = primary.as_scaled(scale);
    let fallback_scaled = fallback.as_scaled(scale);
    let mut x_offset = 0.0f32;
    for ch in text.chars() {
        let primary_id = primary.glyph_id(ch);
        let use_fallback = primary_id.0 == 0;
        let scaled: &dyn Scaler = if use_fallback {
            &fallback_scaled
        } else {
            &primary_scaled
        };
        let glyph_id = if use_fallback {
            fallback.glyph_id(ch)
        } else {
            primary_id
        };
        let h_advance = scaled.h_advance(glyph_id);
        let scaled_glyph = ab_glyph::Glyph {
            id: glyph_id,
            scale,
            position: ab_glyph::point(0.0, 0.0),
        };
        if let Some(outline) = scaled.outline_glyph(scaled_glyph) {
            let bounds = outline.px_bounds();
            let baseline = py as f32 + ScaleFont::ascent(&primary_scaled);
            outline.draw(|gx, gy, alpha| {
                let dx = px as f32 + x_offset + bounds.min.x + gx as f32;
                let dy = baseline + bounds.min.y + gy as f32;
                if dx < 0.0 || dy < 0.0 {
                    return;
                }
                let dx = dx as u32;
                let dy = dy as u32;
                if dx >= img.width() || dy >= img.height() {
                    return;
                }
                blend_pixel(img, dx, dy, fg, alpha);
            });
        }
        x_offset += h_advance;
    }
}

trait Scaler {
    fn h_advance(&self, id: ab_glyph::GlyphId) -> f32;
    fn outline_glyph(&self, g: ab_glyph::Glyph) -> Option<ab_glyph::OutlinedGlyph>;
}

impl<F: Font> Scaler for ab_glyph::PxScaleFont<F> {
    fn h_advance(&self, id: ab_glyph::GlyphId) -> f32 {
        ScaleFont::h_advance(self, id)
    }
    fn outline_glyph(&self, g: ab_glyph::Glyph) -> Option<ab_glyph::OutlinedGlyph> {
        ScaleFont::outline_glyph(self, g)
    }
}

fn blend_pixel(img: &mut RgbaImage, x: u32, y: u32, fg: Rgba<u8>, alpha: f32) {
    let alpha = alpha.clamp(0.0, 1.0);
    if alpha <= 0.005 {
        return;
    }
    let existing = img.get_pixel(x, y);
    let inv = 1.0 - alpha;
    let r = (fg[0] as f32 * alpha + existing[0] as f32 * inv) as u8;
    let g = (fg[1] as f32 * alpha + existing[1] as f32 * inv) as u8;
    let b = (fg[2] as f32 * alpha + existing[2] as f32 * inv) as u8;
    img.put_pixel(x, y, Rgba([r, g, b, 255]));
}

fn resolve(color: Color, fallback: Color) -> Color {
    match color {
        Color::Reset => fallback,
        c => c,
    }
}

/// One of three Unicode line weights; `Skip` means the side has no stroke.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Stroke {
    Skip,
    Light,
    Heavy,
    Double,
}

/// Orientation of a dashed run.
#[derive(Clone, Copy)]
enum Axis {
    Horizontal,
    Vertical,
}

/// Return the single `char` if `s` contains exactly one; else `None`.
#[must_use]
fn single_char(s: &str) -> Option<char> {
    let mut it = s.chars();
    let first = it.next()?;
    if it.next().is_some() {
        return None;
    }
    Some(first)
}

/// Try to paint `ch` as box-drawing or block geometry. Returns true on hit.
#[must_use]
fn try_draw_box_drawing(img: &mut RgbaImage, ch: char, px: u32, py: u32, fg: Rgba<u8>) -> bool {
    if let Some(strokes) = strokes_for(ch) {
        paint_strokes(img, strokes, px, py, fg);
        return true;
    }
    if let Some((axis, stroke, dashes)) = dashed_for(ch) {
        paint_dashed(img, axis, stroke, dashes, px, py, fg);
        return true;
    }
    if try_draw_block(img, ch, px, py, fg) {
        return true;
    }
    false
}

/// Paint the four half-segments of a 4-sided box-drawing glyph.
fn paint_strokes(
    img: &mut RgbaImage,
    strokes: (Stroke, Stroke, Stroke, Stroke),
    px: u32,
    py: u32,
    fg: Rgba<u8>,
) {
    let (top, bottom, left, right) = strokes;
    paint_v_stroke(img, px, py, 0, CENTER_Y, top, fg);
    paint_v_stroke(img, px, py, CENTER_Y, CELL_H - 1, bottom, fg);
    paint_h_stroke(img, px, py, 0, CENTER_X, left, fg);
    paint_h_stroke(img, px, py, CENTER_X, CELL_W - 1, right, fg);
}

/// Paint a vertical stroke segment between inclusive rows `[r0, r1]`.
fn paint_v_stroke(
    img: &mut RgbaImage,
    px: u32,
    py: u32,
    r0: u32,
    r1: u32,
    stroke: Stroke,
    fg: Rgba<u8>,
) {
    let h = r1 - r0 + 1;
    match stroke {
        Stroke::Skip => {}
        Stroke::Light => paint_rect(img, px + CENTER_X, py + r0, 1, h, fg),
        Stroke::Heavy => paint_rect(img, px + CENTER_X - 1, py + r0, 3, h, fg),
        Stroke::Double => {
            paint_rect(img, px + CENTER_X - 1, py + r0, 1, h, fg);
            paint_rect(img, px + CENTER_X + 1, py + r0, 1, h, fg);
        }
    }
}

/// Paint a horizontal stroke segment between inclusive cols `[c0, c1]`.
fn paint_h_stroke(
    img: &mut RgbaImage,
    px: u32,
    py: u32,
    c0: u32,
    c1: u32,
    stroke: Stroke,
    fg: Rgba<u8>,
) {
    let w = c1 - c0 + 1;
    match stroke {
        Stroke::Skip => {}
        Stroke::Light => paint_rect(img, px + c0, py + CENTER_Y, w, 1, fg),
        Stroke::Heavy => paint_rect(img, px + c0, py + CENTER_Y - 1, w, 3, fg),
        Stroke::Double => {
            paint_rect(img, px + c0, py + CENTER_Y - 1, w, 1, fg);
            paint_rect(img, px + c0, py + CENTER_Y + 1, w, 1, fg);
        }
    }
}

/// Paint a dashed stroke run across the cell.
fn paint_dashed(
    img: &mut RgbaImage,
    axis: Axis,
    stroke: Stroke,
    dashes: u32,
    px: u32,
    py: u32,
    fg: Rgba<u8>,
) {
    let extent = match axis {
        Axis::Horizontal => CELL_W,
        Axis::Vertical => CELL_H,
    };
    let gaps = dashes - 1;
    let dash_total = extent.saturating_sub(gaps);
    let base = dash_total / dashes;
    let extra = dash_total % dashes;
    let mut start: u32 = 0;
    for i in 0..dashes {
        let len = base + if i < extra { 1 } else { 0 };
        if len == 0 {
            start += 1;
            continue;
        }
        let end = start + len - 1;
        match axis {
            Axis::Horizontal => paint_h_stroke(img, px, py, start, end, stroke, fg),
            Axis::Vertical => paint_v_stroke(img, px, py, start, end, stroke, fg),
        }
        start += len + 1;
    }
}

/// Try to paint `ch` as a U+2580..U+259F block / shade / quadrant. Returns true on hit.
#[must_use]
fn try_draw_block(img: &mut RgbaImage, ch: char, px: u32, py: u32, fg: Rgba<u8>) -> bool {
    let w = CELL_W;
    let h = CELL_H;
    let hw = w / 2;
    let hh = h / 2;
    match ch {
        '\u{2580}' => paint_rect(img, px, py, w, hh, fg),
        '\u{2581}' => paint_rect(img, px, py + h - h / 8, w, h / 8, fg),
        '\u{2582}' => paint_rect(img, px, py + h - h / 4, w, h / 4, fg),
        '\u{2583}' => paint_rect(img, px, py + h - 3 * h / 8, w, 3 * h / 8, fg),
        '\u{2584}' => paint_rect(img, px, py + hh, w, h - hh, fg),
        '\u{2585}' => paint_rect(img, px, py + h - 5 * h / 8, w, 5 * h / 8, fg),
        '\u{2586}' => paint_rect(img, px, py + h - 3 * h / 4, w, 3 * h / 4, fg),
        '\u{2587}' => paint_rect(img, px, py + h - 7 * h / 8, w, 7 * h / 8, fg),
        '\u{2588}' => paint_rect(img, px, py, w, h, fg),
        '\u{2589}' => paint_rect(img, px, py, 7 * w / 8, h, fg),
        '\u{258A}' => paint_rect(img, px, py, 3 * w / 4, h, fg),
        '\u{258B}' => paint_rect(img, px, py, 5 * w / 8, h, fg),
        '\u{258C}' => paint_rect(img, px, py, hw, h, fg),
        '\u{258D}' => paint_rect(img, px, py, 3 * w / 8, h, fg),
        '\u{258E}' => paint_rect(img, px, py, w / 4, h, fg),
        '\u{258F}' => paint_rect(img, px, py, w / 8, h, fg),
        '\u{2590}' => paint_rect(img, px + hw, py, w - hw, h, fg),
        '\u{2591}' => paint_shade(img, px, py, w, h, fg, 0.25),
        '\u{2592}' => paint_shade(img, px, py, w, h, fg, 0.50),
        '\u{2593}' => paint_shade(img, px, py, w, h, fg, 0.75),
        '\u{2594}' => paint_rect(img, px, py, w, h / 8, fg),
        '\u{2595}' => paint_rect(img, px + w - w / 8, py, w / 8, h, fg),
        '\u{2596}' => paint_rect(img, px, py + hh, hw, h - hh, fg),
        '\u{2597}' => paint_rect(img, px + hw, py + hh, w - hw, h - hh, fg),
        '\u{2598}' => paint_rect(img, px, py, hw, hh, fg),
        '\u{2599}' => {
            paint_rect(img, px, py, hw, h, fg);
            paint_rect(img, px, py + hh, w, h - hh, fg);
        }
        '\u{259A}' => {
            paint_rect(img, px, py, hw, hh, fg);
            paint_rect(img, px + hw, py + hh, w - hw, h - hh, fg);
        }
        '\u{259B}' => {
            paint_rect(img, px, py, w, hh, fg);
            paint_rect(img, px, py + hh, hw, h - hh, fg);
        }
        '\u{259C}' => {
            paint_rect(img, px, py, w, hh, fg);
            paint_rect(img, px + hw, py + hh, w - hw, h - hh, fg);
        }
        '\u{259D}' => paint_rect(img, px + hw, py, w - hw, hh, fg),
        '\u{259E}' => {
            paint_rect(img, px + hw, py, w - hw, hh, fg);
            paint_rect(img, px, py + hh, hw, h - hh, fg);
        }
        '\u{259F}' => {
            paint_rect(img, px + hw, py, w - hw, hh, fg);
            paint_rect(img, px, py + hh, w, h - hh, fg);
        }
        _ => return false,
    }
    true
}

/// Blend `fg` over the cell with the given alpha density (for shade blocks).
fn paint_shade(img: &mut RgbaImage, px: u32, py: u32, w: u32, h: u32, fg: Rgba<u8>, density: f32) {
    for j in 0..h {
        for i in 0..w {
            blend_pixel(img, px + i, py + j, fg, density);
        }
    }
}

/// Table of stroke patterns for U+2500..U+257F non-dashed glyphs.
#[must_use]
fn strokes_for(ch: char) -> Option<(Stroke, Stroke, Stroke, Stroke)> {
    use Stroke::{Double as D, Heavy as H, Light as L, Skip as S};
    let s = match ch {
        '\u{2500}' => (S, S, L, L),
        '\u{2501}' => (S, S, H, H),
        '\u{2502}' => (L, L, S, S),
        '\u{2503}' => (H, H, S, S),
        '\u{250C}' => (S, L, S, L),
        '\u{250D}' => (S, L, S, H),
        '\u{250E}' => (S, H, S, L),
        '\u{250F}' => (S, H, S, H),
        '\u{2510}' => (S, L, L, S),
        '\u{2511}' => (S, L, H, S),
        '\u{2512}' => (S, H, L, S),
        '\u{2513}' => (S, H, H, S),
        '\u{2514}' => (L, S, S, L),
        '\u{2515}' => (L, S, S, H),
        '\u{2516}' => (H, S, S, L),
        '\u{2517}' => (H, S, S, H),
        '\u{2518}' => (L, S, L, S),
        '\u{2519}' => (L, S, H, S),
        '\u{251A}' => (H, S, L, S),
        '\u{251B}' => (H, S, H, S),
        '\u{251C}' => (L, L, S, L),
        '\u{251D}' => (L, L, S, H),
        '\u{251E}' => (H, L, S, L),
        '\u{251F}' => (L, H, S, L),
        '\u{2520}' => (H, H, S, L),
        '\u{2521}' => (H, L, S, H),
        '\u{2522}' => (L, H, S, H),
        '\u{2523}' => (H, H, S, H),
        '\u{2524}' => (L, L, L, S),
        '\u{2525}' => (L, L, H, S),
        '\u{2526}' => (H, L, L, S),
        '\u{2527}' => (L, H, L, S),
        '\u{2528}' => (H, H, L, S),
        '\u{2529}' => (H, L, H, S),
        '\u{252A}' => (L, H, H, S),
        '\u{252B}' => (H, H, H, S),
        '\u{252C}' => (S, L, L, L),
        '\u{252D}' => (S, L, H, L),
        '\u{252E}' => (S, L, L, H),
        '\u{252F}' => (S, L, H, H),
        '\u{2530}' => (S, H, L, L),
        '\u{2531}' => (S, H, H, L),
        '\u{2532}' => (S, H, L, H),
        '\u{2533}' => (S, H, H, H),
        '\u{2534}' => (L, S, L, L),
        '\u{2535}' => (L, S, H, L),
        '\u{2536}' => (L, S, L, H),
        '\u{2537}' => (L, S, H, H),
        '\u{2538}' => (H, S, L, L),
        '\u{2539}' => (H, S, H, L),
        '\u{253A}' => (H, S, L, H),
        '\u{253B}' => (H, S, H, H),
        '\u{253C}' => (L, L, L, L),
        '\u{253D}' => (L, L, H, L),
        '\u{253E}' => (L, L, L, H),
        '\u{253F}' => (L, L, H, H),
        '\u{2540}' => (H, L, L, L),
        '\u{2541}' => (L, H, L, L),
        '\u{2542}' => (H, H, L, L),
        '\u{2543}' => (H, L, H, L),
        '\u{2544}' => (H, L, L, H),
        '\u{2545}' => (L, H, H, L),
        '\u{2546}' => (L, H, L, H),
        '\u{2547}' => (H, L, H, H),
        '\u{2548}' => (L, H, H, H),
        '\u{2549}' => (H, H, H, L),
        '\u{254A}' => (H, H, L, H),
        '\u{254B}' => (H, H, H, H),
        '\u{2550}' => (S, S, D, D),
        '\u{2551}' => (D, D, S, S),
        '\u{2552}' => (S, L, S, D),
        '\u{2553}' => (S, D, S, L),
        '\u{2554}' => (S, D, S, D),
        '\u{2555}' => (S, L, D, S),
        '\u{2556}' => (S, D, L, S),
        '\u{2557}' => (S, D, D, S),
        '\u{2558}' => (L, S, S, D),
        '\u{2559}' => (D, S, S, L),
        '\u{255A}' => (D, S, S, D),
        '\u{255B}' => (L, S, D, S),
        '\u{255C}' => (D, S, L, S),
        '\u{255D}' => (D, S, D, S),
        '\u{255E}' => (L, L, S, D),
        '\u{255F}' => (D, D, S, L),
        '\u{2560}' => (D, D, S, D),
        '\u{2561}' => (L, L, D, S),
        '\u{2562}' => (D, D, L, S),
        '\u{2563}' => (D, D, D, S),
        '\u{2564}' => (S, L, D, D),
        '\u{2565}' => (S, D, L, L),
        '\u{2566}' => (S, D, D, D),
        '\u{2567}' => (L, S, D, D),
        '\u{2568}' => (D, S, L, L),
        '\u{2569}' => (D, S, D, D),
        '\u{256A}' => (L, L, D, D),
        '\u{256B}' => (D, D, L, L),
        '\u{256C}' => (D, D, D, D),
        '\u{256D}' => (S, L, S, L),
        '\u{256E}' => (S, L, L, S),
        '\u{256F}' => (L, S, L, S),
        '\u{2570}' => (L, S, S, L),
        '\u{2574}' => (S, S, L, S),
        '\u{2575}' => (L, S, S, S),
        '\u{2576}' => (S, S, S, L),
        '\u{2577}' => (S, L, S, S),
        '\u{2578}' => (S, S, H, S),
        '\u{2579}' => (H, S, S, S),
        '\u{257A}' => (S, S, S, H),
        '\u{257B}' => (S, H, S, S),
        '\u{257C}' => (S, S, L, H),
        '\u{257D}' => (L, H, S, S),
        '\u{257E}' => (S, S, H, L),
        '\u{257F}' => (H, L, S, S),
        _ => return None,
    };
    Some(s)
}

/// Table of dashed glyphs in U+2504..U+250B and U+254C..U+254F.
#[must_use]
fn dashed_for(ch: char) -> Option<(Axis, Stroke, u32)> {
    let v = match ch {
        '\u{2504}' => (Axis::Horizontal, Stroke::Light, 3),
        '\u{2505}' => (Axis::Horizontal, Stroke::Heavy, 3),
        '\u{2506}' => (Axis::Vertical, Stroke::Light, 3),
        '\u{2507}' => (Axis::Vertical, Stroke::Heavy, 3),
        '\u{2508}' => (Axis::Horizontal, Stroke::Light, 4),
        '\u{2509}' => (Axis::Horizontal, Stroke::Heavy, 4),
        '\u{250A}' => (Axis::Vertical, Stroke::Light, 4),
        '\u{250B}' => (Axis::Vertical, Stroke::Heavy, 4),
        '\u{254C}' => (Axis::Horizontal, Stroke::Light, 2),
        '\u{254D}' => (Axis::Horizontal, Stroke::Heavy, 2),
        '\u{254E}' => (Axis::Vertical, Stroke::Light, 2),
        '\u{254F}' => (Axis::Vertical, Stroke::Heavy, 2),
        _ => return None,
    };
    Some(v)
}

fn color_to_rgba(color: Color) -> Rgba<u8> {
    match color {
        Color::Rgb(r, g, b) => Rgba([r, g, b, 255]),
        Color::Reset => color_to_rgba(palette::DEFAULT_FG),
        Color::Black => Rgba([0, 0, 0, 255]),
        Color::White => Rgba([255, 255, 255, 255]),
        Color::Red => color_to_rgba(palette::OCHRE),
        Color::Green => color_to_rgba(palette::SUCCESS),
        Color::Yellow => color_to_rgba(palette::OCHRE),
        Color::Blue => color_to_rgba(palette::WAVE),
        Color::Magenta => color_to_rgba(palette::OCHRE),
        Color::Cyan => color_to_rgba(palette::WAVE_SOFT),
        Color::Gray => color_to_rgba(palette::PAPER_DIM),
        Color::DarkGray => color_to_rgba(palette::MUTED),
        Color::LightRed => color_to_rgba(palette::OCHRE),
        Color::LightGreen => color_to_rgba(palette::SUCCESS),
        Color::LightYellow => color_to_rgba(palette::OCHRE),
        Color::LightBlue => color_to_rgba(palette::WAVE_SOFT),
        Color::LightMagenta => color_to_rgba(palette::OCHRE),
        Color::LightCyan => color_to_rgba(palette::WAVE_SOFT),
        Color::Indexed(_) => color_to_rgba(palette::DEFAULT_FG),
    }
}
