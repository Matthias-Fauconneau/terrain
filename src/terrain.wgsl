struct Uniforms { view_position: vec2<f32>, yaw_sincos: vec2<f32>, aspect_ratio: f32 }
@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct Height {
 @location(0) height: f32,
}

struct VertexOutput {
	@builtin(position) position: vec4<f32>,
	@location(0) color: vec3<f32>,
}

@vertex fn vertex(@builtin(vertex_index) vertex_index: u32, vertex: Height) -> VertexOutput {
	let texture_coordinates = vec2(f32(vertex_index >> 1), f32(vertex_index & 1)) * 2.;
	return VertexOutput(vec4(texture_coordinates * vec2(2., -2.) + vec2(-1., 1.), 0., 1.), vec3(vertex.height));
}

@fragment fn fragment(vertex_output: VertexOutput) -> @location(0) vec4<f32> {
	return vec4<f32>(vertex_output.color, 1.);
}