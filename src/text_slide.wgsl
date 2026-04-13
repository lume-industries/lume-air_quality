// VZGLYD prepends the stable shader contract before compiling this body.
// The prelude defines `VzglydVertexInput`, `VzglydVertexOutput`, `u`,
// `t_diffuse`, `t_font`, `t_detail`, `t_lookup`, `s_diffuse`, and `s_font`.

@vertex
fn vs_main(in: VzglydVertexInput) -> VzglydVertexOutput {
    var out: VzglydVertexOutput;
    out.clip_pos = vec4<f32>(in.position, 1.0);
    out.tex_coords = in.tex_coords;
    out.color = in.color;
    out.mode = in.mode;
    return out;
}

@fragment
fn fs_main(in: VzglydVertexOutput) -> @location(0) vec4<f32> {
    var c = in.color;
    if in.mode > 2.5 {
        let tex = textureSample(t_font, s_font, in.tex_coords);
        c.a *= tex.a;
    }
    return c;
}
