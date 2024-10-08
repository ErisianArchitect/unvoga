#import bevy_pbr::view_transformations::position_world_to_clip
#import bevy_pbr::mesh_functions

@group(2) @binding(0) var array_texture: texture_2d_array<f32>;
@group(2) @binding(1) var array_texture_sampler: sampler;
// The global light level.
@group(2) @binding(2) var<uniform> light_level: f32;
// 16x16x16 = 4096
@group(2) @binding(3) var<storage> lightmap: array<f32>;
@group(2) @binding(4) var<storage> lightmap_pad_pos_x: array<f32>;
@group(2) @binding(5) var<storage> lightmap_pad_neg_x: array<f32>;
@group(2) @binding(6) var<storage> lightmap_pad_pos_y: array<f32>;
@group(2) @binding(7) var<storage> lightmap_pad_neg_y: array<f32>;
@group(2) @binding(8) var<storage> lightmap_pad_pos_z: array<f32>;
@group(2) @binding(9) var<storage> lightmap_pad_neg_z: array<f32>;

const MIN_LIGHT: f32 = 0.025;

const LIGHT_DIR: vec3f = vec3f(
    -0.37139067, -0.74278134, -0.557086
);
const LIGHT_COLOR: vec3f = vec3f(
    1.0,
    1.0,
    1.0,
);
const DIFFUSE_MIN: f32 = 0.2;
const DIFFUSE_MAX: f32 = 1.0;

fn calc_diffuse(diff: f32) -> f32 {
    let diff_diff = DIFFUSE_MAX - DIFFUSE_MIN;
    return diff * diff_diff + DIFFUSE_MIN;
}

fn lightmap_index(x: u32, y: u32, z: u32) -> u32 {
    return x | (z << 4) | (y << 8);
}

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) texindex: u32,
};

struct Fragment {
    @builtin(position) position: vec4f,
    @location(0) uv: vec2<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) texindex: u32,
    @location(3) localpos: vec4<f32>,
};

@vertex
fn vertex(input: Vertex) -> Fragment {
    var model = mesh_functions::get_model_matrix(input.instance_index);
    var output: Fragment;
    output.position = mesh_functions::mesh_position_local_to_clip(
        model,
        vec4<f32>(input.position, 1.0)
    );
    output.localpos = vec4<f32>(input.position, 1.0);
    output.uv = input.uv;
    output.normal = input.normal;
    output.texindex = input.texindex;
    return output;
}

@fragment
fn fragment(input: Fragment) -> @location(0) vec4<f32> {
    let uv = vec2(
        fract(input.uv.x),
        fract(input.uv.y)
    );
    let color = textureSample(
        array_texture,
        array_texture_sampler,
        uv,
        input.texindex
    );
    let norm = input.normal;
    let diff = calc_diffuse(max(dot(norm, -LIGHT_DIR), 0.0));
    let diffuse = diff * LIGHT_COLOR;
    // color.rgb = color.rgb * diffuse;
    let rgb = color.rgb * diffuse;
    let output = vec4f(rgb, color.a);
    // Light level adjustment. I have to rewrite this for 3D.
    // let x = u32(rem_euclid(input.localpos.x, 16.0));
    // let y = u32(rem_euclid(input.localpos.z, 16.0));
    // var index = lightmap_index(x, y);
    // let light = lightmap[index];
    // let min_light = (1.0 - MIN_LIGHT) * light + MIN_LIGHT;
    // let adj_light_level = (1.0 - light_level) * min_light + light_level;
    // let adj_rgb = color.rgb * adj_light_level;
    // let output = vec4f(adj_rgb, color.a);
    // let output = vec4f(color.rgb, color.a);
    return output;
}

fn rem_euclid(a: f32, b: f32) -> f32 {
    let r = a % b;
    if r < 0.0 {
        return r + b;
    } else {
        return r;
    }
}