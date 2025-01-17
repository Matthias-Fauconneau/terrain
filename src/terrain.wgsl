struct Uniforms { view_projection: mat4x4<f32>, vertex_grid_size_x: u32 }
@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct Vertex {
	@location(0) z: f32,
	@location(1) NdotL: f32,
	@location(2) water: f32,
}

struct VertexOutput {
	@builtin(position) position: vec4<f32>,
	@location(0) color: vec3<f32>,
}

@vertex fn vertex(@builtin(vertex_index) vertex_index: u32, vertex: Vertex) -> VertexOutput {
	let xy = vec2(f32(vertex_index % uniforms.vertex_grid_size_x), f32(vertex_index / uniforms.vertex_grid_size_x)) / f32(uniforms.vertex_grid_size_x) * 2. - 1.;
	let position = uniforms.view_projection * vec4(xy, vertex.z, 1.);
	let color = mix(vec3(1./8.,1./6.,1./12.),vec3(0.,1./10.,1./5.),vertex.water)*vertex.NdotL;
	return VertexOutput(position, color);
}

@fragment fn fragment(vertex_output: VertexOutput) -> @location(0) vec4<f32> {
	return vec4<f32>(vertex_output.color, 1.);
}
