// Dashboard slide shader body. LUME prepends the stable Screen2D shader
// contract before compiling this source.

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
    var color = in.color;

    if in.mode >= 2.5 {
        let tex = textureSample(t_font, s_font, in.tex_coords);
        color *= tex;
    } else if in.mode >= 1.5 {
        let pulse = 0.55 + 0.45 * sin(u.time * 2.2);
        color.a *= pulse;
    } else if in.mode >= 0.5 {
        let tex = textureSample(t_diffuse, s_diffuse, in.tex_coords);
        color *= tex;
    }

    let screen_y = in.clip_pos.y / 480.0;
    let scan_pos = fract(u.time * 0.3);
    let scan_dist = abs(screen_y - scan_pos);
    let glow = max(0.0, 1.0 - scan_dist * 35.0) * 0.18;
    color = vec4<f32>(color.rgb + vec3<f32>(glow), color.a);

    return color;
}
