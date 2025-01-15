struct Uniforms { grid_size: vec2<u32>, view_position: vec2<f32>, yaw_sincos: vec2<f32>, aspect_ratio: f32 }
@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexOutput {
	@builtin(position) position: vec4<f32>,
	@location(0) color: vec3<f32>,
}

@vertex fn vertex(@builtin(vertex_index) vertex_index: u32, @location(0) height: f32) -> VertexOutput {
	let zero_one = vec2(f32(vertex_index % uniforms.grid_size.x), f32(vertex_index / uniforms.grid_size.x)) / vec2<f32>(uniforms.grid_size);
	return VertexOutput(vec4(zero_one * 2. - 1., 0., 1.), vec3(height));
}

@fragment fn fragment(vertex_output: VertexOutput) -> @location(0) vec4<f32> {
	return vec4<f32>(vertex_output.color, 1.);
}
