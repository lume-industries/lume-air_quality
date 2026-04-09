use once_cell::sync::Lazy;
use vzglyd_slide::{
    DrawSource, DrawSpec, FilterMode, Limits, PipelineKind, SceneSpace, ShaderSources, SlideSpec,
    StaticMesh, TextureDesc, TextureFormat, WrapMode,
};

pub use vzglyd_slide::ScreenVertex as Vertex;

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

const SOLID: f32 = 0.0;
const TEXTURED: f32 = 1.0;
const PULSING: f32 = 2.0;

fn build_dashboard_geometry() -> (Vec<Vertex>, Vec<u16>) {
    let mut verts: Vec<Vertex> = Vec::new();

    verts.extend_from_slice(&quad(
        -1.0,
        -1.0,
        1.0,
        1.0,
        0.95,
        [0.0, 0.0],
        [1.0, 1.0],
        [0.03, 0.05, 0.10, 1.0],
        SOLID,
    ));

    verts.extend_from_slice(&quad(
        -0.78,
        -0.82,
        0.78,
        0.82,
        0.78,
        [0.0, 0.0],
        [8.0, 6.0],
        [1.0, 1.0, 1.0, 1.0],
        TEXTURED,
    ));

    verts.extend_from_slice(&quad(
        -1.0,
        0.82,
        1.0,
        1.0,
        0.40,
        [0.0, 0.0],
        [1.0, 1.0],
        [0.10, 0.28, 0.60, 0.92],
        SOLID,
    ));

    verts.extend_from_slice(&quad(
        -1.0,
        -1.0,
        1.0,
        -0.82,
        0.40,
        [0.0, 0.0],
        [1.0, 1.0],
        [0.10, 0.28, 0.60, 0.92],
        SOLID,
    ));

    verts.extend_from_slice(&quad(
        -1.0,
        -0.82,
        -0.78,
        0.82,
        0.50,
        [0.0, 0.0],
        [1.0, 1.0],
        [0.05, 0.12, 0.25, 0.82],
        SOLID,
    ));

    verts.extend_from_slice(&quad(
        0.78,
        -0.82,
        1.0,
        0.82,
        0.50,
        [0.0, 0.0],
        [1.0, 1.0],
        [0.05, 0.12, 0.25, 0.82],
        SOLID,
    ));

    let accent = [0.15, 0.75, 1.00, 1.0];
    let cz = 0.30;
    verts.extend_from_slice(&quad(
        -1.0,
        0.88,
        -0.90,
        1.0,
        cz,
        [0.0, 0.0],
        [1.0, 1.0],
        accent,
        SOLID,
    ));
    verts.extend_from_slice(&quad(
        0.90,
        0.88,
        1.0,
        1.0,
        cz,
        [0.0, 0.0],
        [1.0, 1.0],
        accent,
        SOLID,
    ));
    verts.extend_from_slice(&quad(
        -1.0,
        -1.00,
        -0.90,
        -0.88,
        cz,
        [0.0, 0.0],
        [1.0, 1.0],
        accent,
        SOLID,
    ));
    verts.extend_from_slice(&quad(
        0.90,
        -1.00,
        1.0,
        -0.88,
        cz,
        [0.0, 0.0],
        [1.0, 1.0],
        accent,
        SOLID,
    ));

    verts.extend_from_slice(&quad(
        -0.28,
        -0.13,
        0.28,
        0.13,
        0.20,
        [0.0, 0.0],
        [1.0, 1.0],
        [0.65, 0.88, 1.0, 0.85],
        PULSING,
    ));

    let num_quads = (verts.len() / 4) as u16;
    let indices: Vec<u16> = (0..num_quads)
        .flat_map(|q| {
            let base = q * 4;
            [base, base + 1, base + 2, base, base + 2, base + 3]
        })
        .collect();

    (verts, indices)
}

fn make_grid_texture_data() -> Vec<u8> {
    const SIZE: u32 = 64;
    const CELL: u32 = 8;

    let mut texels = vec![0u8; (SIZE * SIZE * 4) as usize];
    for y in 0..SIZE {
        for x in 0..SIZE {
            let i = ((y * SIZE + x) * 4) as usize;
            let on_line = x % CELL == 0 || y % CELL == 0;
            let on_major = x % (CELL * 4) == 0 || y % (CELL * 4) == 0;
            if on_major {
                texels[i] = 30;
                texels[i + 1] = 110;
                texels[i + 2] = 180;
                texels[i + 3] = 255;
            } else if on_line {
                texels[i] = 15;
                texels[i + 1] = 55;
                texels[i + 2] = 100;
                texels[i + 3] = 255;
            } else {
                texels[i] = 6;
                texels[i + 1] = 14;
                texels[i + 2] = 28;
                texels[i + 3] = 255;
            }
        }
    }
    texels
}

pub fn dashboard_slide_spec() -> SlideSpec<Vertex> {
    let (verts, indices) = build_dashboard_geometry();
    let mesh = StaticMesh {
        label: "dashboard".into(),
        vertices: verts,
        indices,
    };

    SlideSpec {
        name: "dashboard_scene".into(),
        limits: Limits::pi4(),
        scene_space: SceneSpace::Screen2D,
        camera_path: None,
        shaders: Some(ShaderSources {
            vertex_wgsl: None,
            fragment_wgsl: Some(include_str!("dashboard_shader.wgsl").to_string()),
        }),
        overlay: None,
        font: None,
        textures_used: 1,
        textures: vec![TextureDesc {
            label: "grid_texture".into(),
            width: 64,
            height: 64,
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
            label: "dashboard_main".into(),
            source: DrawSource::Static(0),
            pipeline: PipelineKind::Opaque,
            index_range: 0..(mesh.indices.len() as u32),
        }],
        lighting: None,
    }
}

const WIRE_VERSION: u8 = 1;

static SPEC_BYTES: Lazy<Vec<u8>> = Lazy::new(|| {
    let spec = dashboard_slide_spec();
    let mut buf = vec![WIRE_VERSION];
    let mut body = postcard::to_stdvec(&spec).expect("serialize dashboard slide spec");
    buf.append(&mut body);
    buf
});

pub fn serialized_spec() -> &'static [u8] {
    &SPEC_BYTES
}


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
    0
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
fn slide_update(_dt: f32) -> i32 {
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vertex_is_40_bytes() {
        assert_eq!(std::mem::size_of::<Vertex>(), 40);
    }

    #[test]
    fn geometry_indices_match_quad_count() {
        let (verts, indices) = build_dashboard_geometry();
        assert_eq!(indices.len(), (verts.len() / 4) * 6);
    }

    #[test]
    fn geometry_indices_are_in_bounds() {
        let (verts, indices) = build_dashboard_geometry();
        let max_index = verts.len() as u16;
        for idx in indices {
            assert!(idx < max_index, "index {idx} out of bounds");
        }
    }

    #[test]
    fn spec_validates() {
        dashboard_slide_spec().validate().unwrap();
    }
}
