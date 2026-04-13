#![allow(dead_code)]

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub use VRX_64_slide::{
    DrawSource, DrawSpec, FilterMode, FontAtlas, GlyphInfo, Limits, PipelineKind,
    RuntimeOverlay, SceneSpace, ShaderSources, SlideSpec, StaticMesh, TextureDesc, TextureFormat,
    WrapMode,
};

pub use VRX_64_slide::ScreenVertex as Vertex;

pub const WIRE_VERSION: u8 = 1;
pub const VIRTUAL_WIDTH: f32 = 320.0;
pub const VIRTUAL_HEIGHT: f32 = 240.0;

const MODE_SOLID: f32 = 0.0;
const MODE_FONT: f32 = 3.0;
const WHITE_RGBA: [u8; 4] = [255, 255, 255, 255];

#[derive(Clone, Copy)]
pub struct Palette {
    pub background: [f32; 4],
    pub panel: [f32; 4],
    pub accent: [f32; 4],
    pub accent_soft: [f32; 4],
}

#[derive(Clone, Copy)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

pub struct TextBlock<'a> {
    pub text: &'a str,
    pub x: f32,
    pub y: f32,
    pub scale: f32,
    pub color: [f32; 4],
    pub align: TextAlign,
    pub wrap_cols: Option<usize>,
}

/// Font data needed for text composition.
/// Returned by `font_catalog::make_font()` and passed into `compose_overlay`.
pub struct FontAssets {
    pub atlas: FontAtlas,
    pub glyphs: HashMap<char, GlyphInfo>,
    pub advance_width: f32,
}

pub fn normalize_text(input: &str) -> String {
    let mut normalized = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '\u{2018}' | '\u{2019}' => normalized.push('\''),
            '\u{201C}' | '\u{201D}' => normalized.push('"'),
            '\u{2013}' | '\u{2014}' | '\u{2212}' => normalized.push('-'),
            '\u{2026}' => normalized.push_str("..."),
            '\n' | '\r' | '\t' => normalized.push(' '),
            ch if ch.is_ascii() && !ch.is_ascii_control() => normalized.push(ch),
            _ => normalized.push('?'),
        }
    }
    normalized
}

/// Compose text blocks into a runtime overlay using the provided font.
pub fn compose_overlay(blocks: &[TextBlock<'_>], font: &FontAssets) -> RuntimeOverlay<Vertex> {
    let glyphs = &font.glyphs;
    let advance_base = font.advance_width;
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for block in blocks {
        let lines = wrap_text(block.text, block.wrap_cols);
        let advance = advance_base * block.scale;
        let line_height = advance * 1.25;

        for (line_idx, line) in lines.iter().enumerate() {
            let line_width = line.chars().count() as f32 * advance;
            let line_x = match block.align {
                TextAlign::Left => block.x,
                TextAlign::Center => block.x - line_width * 0.5,
                TextAlign::Right => block.x - line_width,
            };
            let line_y = block.y + line_idx as f32 * line_height;

            for (glyph_idx, ch) in line.chars().enumerate() {
                let glyph_char = if glyphs.contains_key(&ch) { ch } else { '?' };
                let glyph = glyphs
                    .get(&glyph_char)
                    .expect("font atlas should include fallback glyphs");
                let x0 = line_x + glyph_idx as f32 * advance;
                let y0 = line_y;
                let x1 = x0 + advance;
                let y1 = y0 + advance;
                push_quad(
                    &mut vertices,
                    &mut indices,
                    rect_to_ndc(x0, y0, x1, y1),
                    [glyph.u0, glyph.v0],
                    [glyph.u1, glyph.v1],
                    block.color,
                    MODE_FONT,
                    0.12,
                );
            }
        }
    }

    RuntimeOverlay { vertices, indices }
}

/// Build a default slide spec with a background panel, text overlay, and font atlas.
pub fn default_panel_spec(
    name: &str,
    overlay: RuntimeOverlay<Vertex>,
    palette: Palette,
    font: FontAtlas,
) -> SlideSpec<Vertex> {
    let mesh = background_mesh(palette);

    SlideSpec {
        name: name.into(),
        limits: Limits::pi4(),
        scene_space: SceneSpace::Screen2D,
        camera_path: None,
        shaders: Some(ShaderSources {
            vertex_wgsl: Some(include_str!("text_slide.wgsl").to_string()),
            fragment_wgsl: Some(include_str!("text_slide.wgsl").to_string()),
        }),
        overlay: Some(overlay),
        font: Some(font),
        textures_used: 1,
        textures: vec![TextureDesc {
            label: "blank".into(),
            width: 1,
            height: 1,
            format: TextureFormat::Rgba8Unorm,
            wrap_u: WrapMode::ClampToEdge,
            wrap_v: WrapMode::ClampToEdge,
            wrap_w: WrapMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mip_filter: FilterMode::Nearest,
            data: WHITE_RGBA.to_vec(),
        }],
        static_meshes: vec![mesh.clone()],
        dynamic_meshes: vec![],
        draws: vec![DrawSpec {
            label: format!("{name}_panel"),
            source: DrawSource::Static(0),
            pipeline: PipelineKind::Transparent,
            index_range: 0..mesh.indices.len() as u32,
        }],
        lighting: None,
    }
}

pub fn serialize_spec(spec: &SlideSpec<Vertex>) -> Vec<u8> {
    let mut buf = vec![WIRE_VERSION];
    buf.append(&mut postcard::to_stdvec(spec).expect("serialize text slide spec"));
    buf
}

pub fn serialize_overlay(overlay: &RuntimeOverlay<Vertex>) -> Vec<u8> {
    postcard::to_stdvec(overlay).expect("serialize text overlay")
}

pub fn now_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn background_mesh(palette: Palette) -> StaticMesh<Vertex> {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let scrim = with_alpha(palette.background, 0.08);
    let panel = with_alpha(palette.panel, 0.72);
    let accent = with_alpha(palette.accent, 0.86);
    let accent_soft = with_alpha(palette.accent_soft, 0.42);
    push_quad(
        &mut vertices,
        &mut indices,
        [-1.0, -1.0, 1.0, 1.0],
        [0.0, 0.0],
        [1.0, 1.0],
        scrim,
        MODE_SOLID,
        0.97,
    );
    push_quad(
        &mut vertices,
        &mut indices,
        [-0.88, -0.80, 0.88, 0.80],
        [0.0, 0.0],
        [1.0, 1.0],
        panel,
        MODE_SOLID,
        0.62,
    );
    push_quad(
        &mut vertices,
        &mut indices,
        [-0.88, 0.72, 0.88, 0.80],
        [0.0, 0.0],
        [1.0, 1.0],
        accent,
        MODE_SOLID,
        0.56,
    );
    push_quad(
        &mut vertices,
        &mut indices,
        [-0.88, -0.80, 0.88, -0.77],
        [0.0, 0.0],
        [1.0, 1.0],
        accent_soft,
        MODE_SOLID,
        0.56,
    );
    push_quad(
        &mut vertices,
        &mut indices,
        [-0.70, 0.08, 0.70, 0.16],
        [0.0, 0.0],
        [1.0, 1.0],
        [1.0, 1.0, 1.0, 0.05],
        MODE_SOLID,
        0.32,
    );

    StaticMesh {
        label: "text_panel".into(),
        vertices,
        indices,
    }
}

fn rect_to_ndc(x0: f32, y0: f32, x1: f32, y1: f32) -> [f32; 4] {
    [
        px_to_ndc_x(x0),
        px_to_ndc_y(y1),
        px_to_ndc_x(x1),
        px_to_ndc_y(y0),
    ]
}

fn px_to_ndc_x(px: f32) -> f32 {
    px / (VIRTUAL_WIDTH * 0.5) - 1.0
}

fn px_to_ndc_y(py: f32) -> f32 {
    1.0 - py / (VIRTUAL_HEIGHT * 0.5)
}

fn with_alpha(mut color: [f32; 4], alpha: f32) -> [f32; 4] {
    color[3] = alpha;
    color
}

fn push_quad(
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u16>,
    rect: [f32; 4],
    uv0: [f32; 2],
    uv1: [f32; 2],
    color: [f32; 4],
    mode: f32,
    z: f32,
) {
    let base = vertices.len() as u16;
    let [x0, y0, x1, y1] = rect;
    vertices.extend_from_slice(&[
        Vertex {
            position: [x0, y0, z],
            tex_coords: [uv0[0], uv1[1]],
            color,
            mode,
        },
        Vertex {
            position: [x1, y0, z],
            tex_coords: [uv1[0], uv1[1]],
            color,
            mode,
        },
        Vertex {
            position: [x1, y1, z],
            tex_coords: [uv1[0], uv0[1]],
            color,
            mode,
        },
        Vertex {
            position: [x0, y1, z],
            tex_coords: [uv0[0], uv0[1]],
            color,
            mode,
        },
    ]);
    indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}

fn wrap_text(text: &str, max_cols: Option<usize>) -> Vec<String> {
    let normalized = normalize_text(text);
    let Some(max_cols) = max_cols else {
        return normalized
            .split('\n')
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect();
    };

    let mut wrapped = Vec::new();
    for paragraph in normalized.split('\n') {
        let mut current = String::new();
        for word in paragraph.split_whitespace() {
            if current.is_empty() {
                current.push_str(word);
                continue;
            }

            let proposed_len = current.chars().count() + 1 + word.chars().count();
            if proposed_len > max_cols {
                wrapped.push(current);
                current = word.to_string();
            } else {
                current.push(' ');
                current.push_str(word);
            }
        }

        if !current.is_empty() {
            wrapped.push(current);
        }
    }

    if wrapped.is_empty() {
        wrapped.push(String::new());
    }
    wrapped
}
