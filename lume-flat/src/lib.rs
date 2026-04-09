use once_cell::sync::Lazy;
use vzglyd_slide::{
    CameraKeyframe, CameraPath, DrawSource, DrawSpec, FilterMode, FontAtlas, GlyphInfo, Limits,
    PipelineKind, RuntimeOverlay, SceneSpace, SlideSpec, StaticMesh, TextureDesc, TextureFormat,
    WrapMode,
};

pub use vzglyd_slide::ScreenVertex as Vertex;

// Minimal 5×7 bitmap font — digits, colon, and space.
pub fn glyph(c: u8) -> [u8; 7] {
    match c {
        b' ' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        b'0' => [0x0E, 0x11, 0x13, 0x15, 0x19, 0x11, 0x0E],
        b'1' => [0x04, 0x0C, 0x04, 0x04, 0x04, 0x04, 0x0E],
        b'2' => [0x0E, 0x11, 0x01, 0x02, 0x04, 0x08, 0x1F],
        b'3' => [0x0E, 0x11, 0x01, 0x06, 0x01, 0x11, 0x0E],
        b'4' => [0x02, 0x06, 0x0A, 0x12, 0x1F, 0x02, 0x02],
        b'5' => [0x1F, 0x10, 0x1E, 0x01, 0x01, 0x11, 0x0E],
        b'6' => [0x0E, 0x10, 0x10, 0x1E, 0x11, 0x11, 0x0E],
        b'7' => [0x1F, 0x01, 0x02, 0x04, 0x04, 0x04, 0x04],
        b'8' => [0x0E, 0x11, 0x11, 0x0E, 0x11, 0x11, 0x0E],
        b'9' => [0x0E, 0x11, 0x11, 0x0F, 0x01, 0x11, 0x0E],
        b':' => [0x00, 0x04, 0x00, 0x00, 0x00, 0x04, 0x00],
        _ => [0x00; 7],
    }
}

fn make_font_atlas() -> (FontAtlas, std::collections::HashMap<char, GlyphInfo>) {
    const AW: usize = 256;
    const AH: usize = 8;
    let mut buf = vec![0u8; AW * AH * 4];
    let chars = b" 0123456789:AMP";
    let mut map = std::collections::HashMap::new();
    for (ci, &c) in chars.iter().enumerate() {
        let rows = glyph(c);
        let xb = ci * 6;
        for (row, &byte) in rows.iter().enumerate() {
            for col in 0..5usize {
                if (byte >> (4 - col)) & 1 == 1 {
                    let i = (row * AW + xb + col) * 4;
                    buf[i] = 255;
                    buf[i + 1] = 255;
                    buf[i + 2] = 255;
                    buf[i + 3] = 255;
                }
            }
        }
        let u0 = xb as f32 / AW as f32;
        let u1 = (xb + 5) as f32 / AW as f32;
        let v0 = 0.0;
        let v1 = 7.0 / AH as f32;
        map.insert(
            c as char,
            GlyphInfo {
                codepoint: c as u32,
                u0,
                v0,
                u1,
                v1,
            },
        );
    }
    (
        FontAtlas {
            width: AW as u32,
            height: AH as u32,
            pixels: buf,
            glyphs: map.values().cloned().collect(),
        },
        map,
    )
}

fn quad(
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    z: f32,
    uv0: [f32; 2],
    uv1: [f32; 2],
    color: [f32; 4],
    mode: f32,
) -> [Vertex; 4] {
    [
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
    ]
}

pub fn make_grid_texture_data() -> Vec<u8> {
    const SIZE: u32 = 32;
    let mut texels = vec![0u8; (SIZE * SIZE * 4) as usize];
    for y in 0..SIZE {
        for x in 0..SIZE {
            let i = ((y * SIZE + x) * 4) as usize;
            let on = (x + y) % 8 == 0;
            if on {
                texels[i] = 220;
                texels[i + 1] = 120;
                texels[i + 2] = 255;
                texels[i + 3] = 255;
            } else {
                texels[i] = 20;
                texels[i + 1] = 10;
                texels[i + 2] = 40;
                texels[i + 3] = 255;
            }
        }
    }
    texels
}

fn build_scene() -> (Vec<Vertex>, Vec<u16>) {
    let mut verts = Vec::new();
    // Dark background
    verts.extend_from_slice(&quad(
        -1.0,
        -1.0,
        1.0,
        1.0,
        0.9,
        [0.0, 0.0],
        [1.0, 1.0],
        [0.04, 0.05, 0.08, 1.0],
        0.0,
    ));
    // Textured panel
    verts.extend_from_slice(&quad(
        -0.8,
        -0.6,
        0.8,
        0.6,
        0.5,
        [0.0, 0.0],
        [4.0, 3.0],
        [1.0, 1.0, 1.0, 1.0],
        1.0,
    ));
    // Accent bar — top
    verts.extend_from_slice(&quad(
        -0.8,
        0.55,
        0.8,
        0.6,
        0.4,
        [0.0, 0.0],
        [1.0, 1.0],
        [0.2, 0.9, 0.7, 0.8],
        0.0,
    ));
    // Accent bar — bottom
    verts.extend_from_slice(&quad(
        -0.8,
        -0.6,
        0.8,
        -0.55,
        0.4,
        [0.0, 0.0],
        [1.0, 1.0],
        [0.9, 0.4, 0.6, 0.8],
        0.0,
    ));
    let num_quads = (verts.len() / 4) as u16;
    let indices: Vec<u16> = (0..num_quads)
        .flat_map(|q| {
            let b = q * 4;
            [b, b + 1, b + 2, b, b + 2, b + 3]
        })
        .collect();
    (verts, indices)
}

fn build_camera() -> CameraPath {
    CameraPath {
        looped: true,
        keyframes: vec![
            CameraKeyframe {
                time: 0.0,
                position: [0.0, 0.0, 1.0],
                target: [0.0, 0.0, 0.0],
                up: [0.0, 1.0, 0.0],
                fov_y_deg: 60.0,
            },
            CameraKeyframe {
                time: 5.0,
                position: [0.0, 0.0, 1.0],
                target: [0.0, 0.0, 0.0],
                up: [0.0, 1.0, 0.0],
                fov_y_deg: 60.0,
            },
        ],
    }
}

/// Build the dynamic clock overlay for elapsed time `t` (seconds since scene start).
pub fn build_overlay(t: f32) -> RuntimeOverlay<Vertex> {
    use core::fmt::Write;
    let secs = t as u64;
    let h = (secs / 3600) % 24;
    let m = (secs / 60) % 60;
    let s = secs % 60;
    let mut buf = heapless::String::<16>::new();
    let _ = write!(buf, "{:02}:{:02}:{:02}", h, m, s);

    let (_atlas, glyph_map) = make_font_atlas();

    // Centre the clock near the top of the screen.
    let cw = 6.0 * 2.0 / 640.0;
    let ch = 7.0 * 2.0 / 480.0;
    let total_w = cw * buf.len() as f32;
    let start_x = -total_w / 2.0;
    let y0 = 0.85;

    let mut verts: Vec<Vertex> = Vec::new();
    let mut idx: Vec<u16> = Vec::new();
    for (i, ch_char) in buf.chars().enumerate() {
        if let Some(g) = glyph_map.get(&ch_char) {
            let x0 = start_x + i as f32 * cw;
            let x1 = x0 + cw;
            let y1 = y0 + ch;
            let base = verts.len() as u16;
            verts.extend_from_slice(&[
                Vertex {
                    position: [x0, y0, 0.1],
                    tex_coords: [g.u0, g.v1],
                    color: [1.0, 1.0, 1.0, 1.0],
                    mode: 3.0,
                },
                Vertex {
                    position: [x1, y0, 0.1],
                    tex_coords: [g.u1, g.v1],
                    color: [1.0, 1.0, 1.0, 1.0],
                    mode: 3.0,
                },
                Vertex {
                    position: [x1, y1, 0.1],
                    tex_coords: [g.u1, g.v0],
                    color: [1.0, 1.0, 1.0, 1.0],
                    mode: 3.0,
                },
                Vertex {
                    position: [x0, y1, 0.1],
                    tex_coords: [g.u0, g.v0],
                    color: [1.0, 1.0, 1.0, 1.0],
                    mode: 3.0,
                },
            ]);
            idx.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
    }
    RuntimeOverlay {
        vertices: verts,
        indices: idx,
    }
}

pub fn flat_slide_spec() -> SlideSpec<Vertex> {
    let (verts, indices) = build_scene();
    let mesh = StaticMesh {
        label: "flat_scene".into(),
        vertices: verts,
        indices,
    };
    let overlay = build_overlay(0.0);
    SlideSpec {
        name: "flat_scene".into(),
        limits: Limits::pi4(),
        scene_space: SceneSpace::Screen2D,
        camera_path: Some(build_camera()),
        shaders: Some(vzglyd_slide::ShaderSources {
            vertex_wgsl: Some(include_str!("flat_shader.wgsl").to_string()),
            fragment_wgsl: Some(include_str!("flat_shader.wgsl").to_string()),
        }),
        font: Some(make_font_atlas().0),
        overlay: Some(overlay),
        textures_used: 1,
        textures: vec![TextureDesc {
            label: "grid".into(),
            width: 32,
            height: 32,
            format: TextureFormat::Rgba8Unorm,
            wrap_u: WrapMode::Repeat,
            wrap_v: WrapMode::Repeat,
            wrap_w: WrapMode::Repeat,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Linear,
            mip_filter: FilterMode::Nearest,
            data: make_grid_texture_data(),
        }],
        static_meshes: vec![mesh.clone()],
        dynamic_meshes: vec![],
        draws: vec![DrawSpec {
            label: "flat_pass".into(),
            source: DrawSource::Static(0),
            pipeline: PipelineKind::Opaque,
            index_range: 0..(mesh.indices.len() as u32),
        }],
        lighting: None,
    }
}

// ── Wire format ───────────────────────────────────────────────────────────────

const WIRE_VERSION: u8 = 1;

static SPEC_BYTES: Lazy<Vec<u8>> = Lazy::new(|| {
    let spec = flat_slide_spec();
    let mut buf = vec![WIRE_VERSION];
    buf.append(&mut postcard::to_stdvec(&spec).expect("serialize flat slide spec"));
    buf
});

pub fn serialized_spec() -> &'static [u8] {
    &SPEC_BYTES
}

// ── WASM exports ─────────────────────────────────────────────────────────────


#[cfg(target_arch = "wasm32")]
vzglyd_slide::export_traced_entrypoints! {
    init = slide_init,
    update = slide_update,
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn vzglyd_spec_ptr() -> *const u8 {
    SPEC_BYTES.as_ptr()
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn vzglyd_spec_len() -> u32 {
    SPEC_BYTES.len() as u32
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn vzglyd_abi_version() -> u32 {
    vzglyd_slide::ABI_VERSION
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
fn slide_init() -> i32 {
    let mut state = runtime_state::state();
    state.elapsed = 0.0;
    state.refresh_overlay();
    0
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
fn slide_update(dt: f32) -> i32 {
    let mut state = runtime_state::state();
    state.elapsed += dt.max(0.0);
    state.refresh_overlay();
    1
}

// ── WASM overlay exports ──────────────────────────────────────────────────────
//
// The host drives this state by calling `vzglyd_update(dt)` every frame. After a
// successful update, it reads the postcard-encoded overlay through the
// pointer/length exports below.

#[cfg(target_arch = "wasm32")]
mod runtime_state {
    use std::sync::{Mutex, MutexGuard, OnceLock};

    pub struct RuntimeState {
        pub elapsed: f32,
        pub overlay_bytes: Vec<u8>,
    }

    impl RuntimeState {
        fn new() -> Self {
            let mut state = Self {
                elapsed: 0.0,
                overlay_bytes: Vec::new(),
            };
            state.refresh_overlay();
            state
        }

        pub fn refresh_overlay(&mut self) {
            self.overlay_bytes =
                postcard::to_stdvec(&super::build_overlay(self.elapsed)).expect("encode overlay");
        }
    }

    static STATE: OnceLock<Mutex<RuntimeState>> = OnceLock::new();

    pub fn state() -> MutexGuard<'static, RuntimeState> {
        STATE
            .get_or_init(|| Mutex::new(RuntimeState::new()))
            .lock()
            .unwrap()
    }
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn vzglyd_overlay_ptr() -> *const u8 {
    runtime_state::state().overlay_bytes.as_ptr()
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn vzglyd_overlay_len() -> u32 {
    runtime_state::state().overlay_bytes.len() as u32
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_valid() {
        flat_slide_spec().validate().unwrap();
    }

    #[test]
    fn overlay_increments() {
        let ov0 = build_overlay(0.0);
        let ov1 = build_overlay(61.0); // 00:01:01
        // Both should produce exactly 8 glyphs (HH:MM:SS)
        assert_eq!(ov0.vertices.len(), 8 * 4);
        assert_eq!(ov1.vertices.len(), 8 * 4);
        // The UV coordinates must differ (different digits rendered)
        let v0_uv: Vec<_> = ov0.vertices.iter().map(|v| v.tex_coords[0]).collect();
        let v1_uv: Vec<_> = ov1.vertices.iter().map(|v| v.tex_coords[0]).collect();
        assert_ne!(
            v0_uv, v1_uv,
            "overlay should produce different UVs for different times"
        );
    }
}
