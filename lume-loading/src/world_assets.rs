#![allow(dead_code, unused_imports)]

pub const MAX_FPS_CHARS: usize = 8;

pub use vzglyd_slide::WorldVertex as Vertex;
pub use vzglyd_slide::make_font_atlas;

pub fn make_noise_texture() -> Vec<u8> {
    use std::f32::consts::TAU;

    const SIZE: u32 = 128;
    let mut buf = vec![0u8; (SIZE * SIZE * 4) as usize];
    for y in 0..SIZE {
        for x in 0..SIZE {
            let nx = x as f32 / SIZE as f32;
            let ny = y as f32 / SIZE as f32;
            let r = 0.5
                + 0.5
                    * (0.55 * (nx * TAU * 2.0).sin() * (ny * TAU).cos()
                        + 0.30 * (nx * TAU * 4.0 + ny * TAU * 3.0).sin()
                        + 0.15 * (nx * TAU * 7.0 - ny * TAU * 5.0).cos());
            let g = 0.5
                + 0.5
                    * (0.55 * (nx * TAU * 2.0 + ny * TAU).cos()
                        + 0.30 * (nx * TAU - ny * TAU * 5.0).sin()
                        + 0.15 * (nx * TAU * 6.0 + ny * TAU * 7.0).sin());
            let i = ((y * SIZE + x) * 4) as usize;
            buf[i] = (r.clamp(0.0, 1.0) * 255.0) as u8;
            buf[i + 1] = (g.clamp(0.0, 1.0) * 255.0) as u8;
            buf[i + 2] = buf[i];
            buf[i + 3] = 255;
        }
    }
    buf
}

pub fn append_stl_mesh(verts: &mut Vec<Vertex>, idx: &mut Vec<u16>, bytes: &[u8], color: [f32; 4]) {
    assert!(bytes.len() >= 84, "binary STL too short (< 84 bytes)");
    let triangle_count = u32::from_le_bytes(bytes[80..84].try_into().unwrap()) as usize;
    assert!(
        bytes.len() >= 84 + triangle_count * 50,
        "binary STL truncated"
    );

    let read_f32 =
        |offset: usize| f32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());

    for triangle in 0..triangle_count {
        let offset = 84 + triangle * 50;
        let normal = [read_f32(offset), read_f32(offset + 4), read_f32(offset + 8)];
        let base = verts.len() as u16;
        for vertex in 0..3usize {
            let vertex_offset = offset + 12 + vertex * 12;
            verts.push(Vertex {
                position: [
                    read_f32(vertex_offset),
                    read_f32(vertex_offset + 4),
                    read_f32(vertex_offset + 8),
                ],
                normal,
                color,
                mode: 0.0,
            });
        }
        idx.extend_from_slice(&[base, base + 1, base + 2]);
    }
}
