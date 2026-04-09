// LUME prepends the stable shader contract before compiling this body.
// The prelude defines `LumeVertexInput`, `LumeVertexOutput`, `u`,
// `t_diffuse`, `t_font`, `s_diffuse`, and `s_font`.

@vertex
fn vs_main(in: LumeVertexInput) -> LumeVertexOutput {
    var out: LumeVertexOutput;
    out.clip_pos = vec4<f32>(in.position, 1.0);
    out.tex_coords = in.tex_coords;
    out.color = in.color;
    out.mode = in.mode;
    return out;
}

@fragment
fn fs_main(in: LumeVertexOutput) -> @location(0) vec4<f32> {
    var c = in.color;
    if in.mode > 2.5 {
        // Mode 3: font glyph — alpha-mask from font atlas
        let tex = textureSample(t_font, s_font, in.tex_coords);
        c.a *= tex.a;
    } else if in.mode > 0.5 {
        // Mode 1: textured — modulate by grid texture
        let tex = textureSample(t_diffuse, s_diffuse, in.tex_coords);
        c *= tex;
    }
    return c;
}
