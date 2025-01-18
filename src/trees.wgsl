struct Uniforms { view_projection: mat4x4<f32>, tree_size: f32 }
@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var<storage> positions : array<vec3<f32>>;

struct VertexOutput {
	@builtin(position) position: vec4<f32>,
	@location(0) color: vec3<f32>,
}

const vertices = array(vec2(-1.,-1.), vec2(1.,-1.), vec2(1.,1.), vec2(-1.,-1.), vec2(1.,1.), vec2(-1.,1.));
@vertex fn vertex(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
	let center = positions[vertex_index/6];
	let position = uniforms.view_projection * vec4(center+vec3(uniforms.tree_size*vertices[vertex_index%6], 0.), 1.);
	return VertexOutput(position, vec3(0.,1.,0.));
}

@fragment fn fragment(vertex_output: VertexOutput) -> @location(0) vec4<f32> {
	return vec4<f32>(vertex_output.color, 1.);
}
