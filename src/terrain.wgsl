struct Uniforms { yaw_sincos: vec2<f32>, pitch_sincos: vec2<f32>, view_position: vec2<f32>, aspect_ratio: f32, grid_size: u32 }
@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct Vertex {
	@location(0) height: f32,
	@location(1) NdotL: f32,
	@location(2) water: f32,
}

struct VertexOutput {
	@builtin(position) position: vec4<f32>,
	@location(0) color: vec3<f32>,
}

@vertex fn vertex(@builtin(vertex_index) vertex_index: u32, vertex: Vertex) -> VertexOutput {
	let xy = vec2(f32(vertex_index % uniforms.grid_size), f32(vertex_index / uniforms.grid_size)) / f32(uniforms.grid_size) * 2. - 1.;
	let p = vec3(xy, vertex.height);
	let p0 = p.xy - uniforms.view_position;
	let s = uniforms.yaw_sincos.x;
	let c = uniforms.yaw_sincos.y;
	let p1 = vec3(c*p0.x - s*p0.y, s*p0.x + c*p0.y, p.z);
	let ps = uniforms.pitch_sincos.x;
	let pc = uniforms.pitch_sincos.y;
	let p2 = vec3(p1.x, pc*p1.y - ps*p1.z, ps*p1.y + pc*p1.z);
	let p3 = vec3(p2.xy, (p2.z-1.)/2.);
	let n = 1./4.;
	let f = 1.;
	let zz = -f/(f-n);
	let z1 = -(f*n)/(f-n);
	let position = vec4(p3.x, uniforms.aspect_ratio*p3.y, zz*p3.z+z1, -p3.z);
	let color = mix(vec3(1./8.,1./6.,1./12.),vec3(0.,1./10.,1./5.),vertex.water)*vertex.NdotL;
	return VertexOutput(position, color);
}

@fragment fn fragment(vertex_output: VertexOutput) -> @location(0) vec4<f32> {
	return vec4<f32>(vertex_output.color, 1.);
}
