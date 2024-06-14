#import bevy_pbr::view_transformations::position_world_to_clip
#import bevy_pbr::mesh_functions

@group(2) @binding(0) var array_texture: texture_2d_array<f32>;
@group(2) @binding(1) var array_texture_sampler: sampler;
// 16x16 = 256
@group(2) @binding(2) var<storage> lightmap: array<f32>;

fn lightmap_index(x: u32, y: u32) -> u32 {
    return x | (y << 4);
}

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) texindex: u32,
};

struct Fragment {
    @builtin(position) position: vec4f,
    @location(0) uv: vec2<f32>,
    @location(1) texindex: u32,
    @location(2) localpos: vec4<f32>,
};

@vertex
fn vertex(input: Vertex) -> Fragment {
    var model = mesh_functions::get_model_matrix(input.instance_index);
    var output: Fragment;
    output.position = mesh_functions::mesh_position_local_to_clip(
        model,
        vec4<f32>(input.position, 1.0)
    );
    // output.localpos = mesh_functions::mesh_position_local_to_world(
    //     model,
    //     vec4<f32>(input.position, 1.0)
    // );
    output.localpos = vec4<f32>(input.position, 1.0);
    output.uv = input.uv;
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
        // vec2(fract(input.localpos.y), fract(input.localpos.z)),
        // vec2(input.uv.x, input.position.x % 1.0),
        uv,
        input.texindex
    );
    let x = u32(rem_euclid(input.localpos.x, 16.0));
    let y = u32(rem_euclid(input.localpos.z, 16.0));
    var index = lightmap_index(x, y);
    let light_level = lightmap[index];
    let output = vec4f(color.rgb * light_level, color.a);
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