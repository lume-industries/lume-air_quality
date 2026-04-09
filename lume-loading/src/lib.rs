mod world_assets;

use std::sync::LazyLock;

use vzglyd_slide::{Limits, SceneSpace, ShaderSources, SlideSpec, WorldLighting};
use world_assets::Vertex;

const WIRE_VERSION: u8 = 1;

fn loading_slide_spec() -> SlideSpec<Vertex> {
    SlideSpec {
        name: "loading_scene".into(),
        limits: Limits::pi4(),
        scene_space: SceneSpace::World3D,
        camera_path: None,
        shaders: Some(ShaderSources {
            vertex_wgsl: None,
            fragment_wgsl: Some(include_str!("loading_shader.wgsl").to_string()),
        }),
        overlay: None,
        font: None,
        textures_used: 0,
        textures: vec![],
        static_meshes: vec![],
        dynamic_meshes: vec![],
        draws: vec![],
        lighting: Some(WorldLighting::new([1.0, 1.0, 1.0], 0.08, None)),
    }
}

static SPEC_BYTES: LazyLock<Vec<u8>> = LazyLock::new(|| {
    let mut bytes = vec![WIRE_VERSION];
    bytes.extend(postcard::to_stdvec(&loading_slide_spec()).expect("serialize loading slide spec"));
    bytes
});

pub fn serialized_spec() -> &'static [u8] {
    &SPEC_BYTES
}

#[cfg(target_arch = "wasm32")]
vzglyd_slide::params_buf!(64);

#[cfg(target_arch = "wasm32")]
fn slide_configure(_len: i32) -> i32 {
    // Loading slide has no configurable parameters.
    0
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
fn slide_init() -> i32 {
    0
}

#[cfg(target_arch = "wasm32")]
fn slide_update(_dt: f32) -> i32 {
    0
}

#[cfg(target_arch = "wasm32")]
vzglyd_slide::export_traced_entrypoints! {
    configure = slide_configure,
    init = slide_init,
    update = slide_update,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_valid() {
        loading_slide_spec().validate().unwrap();
    }

    #[test]
    fn loading_slide_is_minimal_world_scene() {
        let spec = loading_slide_spec();
        assert_eq!(spec.scene_space, SceneSpace::World3D);
        assert!(spec.static_meshes.is_empty());
        assert!(spec.dynamic_meshes.is_empty());
        assert!(spec.draws.is_empty());
        assert!(spec.textures.is_empty());
        assert!(spec.shaders.is_some());
        assert!(spec.lighting.is_some());
    }
}
