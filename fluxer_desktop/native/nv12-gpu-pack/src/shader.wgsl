struct PackUniforms {
    width: u32,
    height: u32,
    stride_y: u32,
    stride_uv: u32,
};

@group(0) @binding(0) var source_tex: texture_2d<f32>;
@group(0) @binding(1) var<storage, read_write> y_plane: array<atomic<u32>>;
@group(0) @binding(2) var<storage, read_write> uv_plane: array<atomic<u32>>;
@group(0) @binding(3) var<uniform> dims: PackUniforms;

fn rgb_to_y_full(r: f32, g: f32, b: f32) -> f32 {
    return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

fn limited_y(y_full: f32) -> u32 {
    let scaled = round(y_full * 219.0) + 16.0;
    return u32(clamp(scaled, 16.0, 235.0));
}

fn limited_u(b: f32, y_full: f32) -> u32 {
    let centered = (b - y_full) / 1.8556;
    let scaled = round(centered * 224.0) + 128.0;
    return u32(clamp(scaled, 16.0, 240.0));
}

fn limited_v(r: f32, y_full: f32) -> u32 {
    let centered = (r - y_full) / 1.5748;
    let scaled = round(centered * 224.0) + 128.0;
    return u32(clamp(scaled, 16.0, 240.0));
}

fn write_y_byte(byte_offset: u32, value: u32) {
    let word_index = byte_offset / 4u;
    let shift = (byte_offset % 4u) * 8u;
    let payload = (value & 0xFFu) << shift;
    atomicOr(&y_plane[word_index], payload);
}

fn write_uv_byte(byte_offset: u32, value: u32) {
    let word_index = byte_offset / 4u;
    let shift = (byte_offset % 4u) * 8u;
    let payload = (value & 0xFFu) << shift;
    atomicOr(&uv_plane[word_index], payload);
}

@compute @workgroup_size(8, 8, 1)
fn pack_nv12(@builtin(global_invocation_id) gid: vec3<u32>) {
    let block_x = gid.x;
    let block_y = gid.y;
    let pixel_x = block_x * 2u;
    let pixel_y = block_y * 2u;
    if (pixel_x + 1u >= dims.width) {
        return;
    }
    if (pixel_y + 1u >= dims.height) {
        return;
    }
    let p00 = textureLoad(source_tex, vec2<i32>(i32(pixel_x), i32(pixel_y)), 0);
    let p10 = textureLoad(source_tex, vec2<i32>(i32(pixel_x + 1u), i32(pixel_y)), 0);
    let p01 = textureLoad(source_tex, vec2<i32>(i32(pixel_x), i32(pixel_y + 1u)), 0);
    let p11 = textureLoad(source_tex, vec2<i32>(i32(pixel_x + 1u), i32(pixel_y + 1u)), 0);
    let y00_full = rgb_to_y_full(p00.r, p00.g, p00.b);
    let y10_full = rgb_to_y_full(p10.r, p10.g, p10.b);
    let y01_full = rgb_to_y_full(p01.r, p01.g, p01.b);
    let y11_full = rgb_to_y_full(p11.r, p11.g, p11.b);
    let y00 = limited_y(y00_full);
    let y10 = limited_y(y10_full);
    let y01 = limited_y(y01_full);
    let y11 = limited_y(y11_full);
    let row0 = pixel_y * dims.stride_y;
    let row1 = (pixel_y + 1u) * dims.stride_y;
    write_y_byte(row0 + pixel_x, y00);
    write_y_byte(row0 + pixel_x + 1u, y10);
    write_y_byte(row1 + pixel_x, y01);
    write_y_byte(row1 + pixel_x + 1u, y11);
    let r_avg = (p00.r + p10.r + p01.r + p11.r) * 0.25;
    let g_avg = (p00.g + p10.g + p01.g + p11.g) * 0.25;
    let b_avg = (p00.b + p10.b + p01.b + p11.b) * 0.25;
    let y_full_avg = rgb_to_y_full(r_avg, g_avg, b_avg);
    let u = limited_u(b_avg, y_full_avg);
    let v = limited_v(r_avg, y_full_avg);
    let uv_row = block_y * dims.stride_uv;
    let uv_offset = uv_row + block_x * 2u;
    write_uv_byte(uv_offset, u);
    write_uv_byte(uv_offset + 1u, v);
}
