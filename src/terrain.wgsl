struct Uniforms { grid_size: vec2<u32>,  yaw_sincos: vec2<f32>, pitch_sincos: vec2<f32>, view_position: vec2<f32>, aspect_ratio: f32 }
@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct Vertex {
	@location(0) height: f32,
	@location(1) NdotL: f32,
}

struct VertexOutput {
	@builtin(position) position: vec4<f32>,
	@location(0) color: vec3<f32>,
}

@vertex fn vertex(@builtin(vertex_index) vertex_index: u32, vertex: Vertex) -> VertexOutput {
	let xy = vec2(f32(vertex_index % uniforms.grid_size.x), f32(vertex_index / uniforms.grid_size.x)) / vec2<f32>(uniforms.grid_size);
	let p = vec3(vec3(xy, 1.-vertex.height) * 2. - 1.);
	let p0 = p.xy - uniforms.view_position;
	let s = uniforms.yaw_sincos.x;
	let c = uniforms.yaw_sincos.y;
	let p1 = vec3(c*p0.x - s*p0.y, s*p0.x + c*p0.y, p.z);
	let ps = uniforms.pitch_sincos.x;
	let pc = uniforms.pitch_sincos.y;
	let p2 = vec3(p1.x, pc*p1.y - ps*p1.z, ps*p1.y + pc*p1.z);
	return VertexOutput(vec4(p2.xy, (p2.z+1.)/2. /*Vulkan clips z/w < 0*/, 1.), vec3(/*vertex.height**/vertex.NdotL));
}

@fragment fn fragment(vertex_output: VertexOutput) -> @location(0) vec4<f32> {
	return vec4<f32>(vertex_output.color, 1.);
}
