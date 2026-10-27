
struct Camera { offset: vec2<f32>, scale: f32, aspect: f32 }
@group(0) @binding(0) var<uniform> cam: Camera;
struct VsOut { @builtin(position) pos: vec4<f32>, @location(0) color: vec4<f32> }
@vertex fn vs(@location(0) pos: vec2<f32>, @location(1) color: vec4<f32>) -> VsOut {
    var out: VsOut;
    let p = (pos + cam.offset) * cam.scale;
    out.pos = vec4<f32>(p.x / cam.aspect, p.y, 0.0, 1.0);
    out.color = color;
    return out;
}
@fragment fn fs(in: VsOut) -> @location(0) vec4<f32> { return in.color; }
