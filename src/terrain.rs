fn minmax(values: &[f32]) -> [f32; 2] {
	let [mut min, mut max] = [f32::INFINITY, -f32::INFINITY];
	for &value in values { if value > f32::MIN && value < min { min = value; } if value > max { max = value; } }
	[min, max]
}

use ui::{vulkan, shader};
shader!{terrain} // z: f32, NdotL: f32, water: f32,
use vulkan::Subbuffer;
pub struct Terrain {
	pass: terrain::Pass,
	vertex_grid_size_x: u32,
	grid: Subbuffer::<[u32]>,
	vertices: Subbuffer::<[terrain::Vertex]>,
}

use {ui::{Error, Result, throws}, std::cmp::min, num::sq, vector::{xy, xyz, dot, normalize, cross, mat4}, image::f32};
use vulkan::{Context, buffer, BufferUsage, Commands, Arc, ImageView};
impl Terrain {
	pub fn new(context: &Context) -> Result<Self> {
		let ground = f32(std::env::args().skip(1).next().unwrap_or("data/DTM_R.tif.tif.exr".to_owned()))?;
		let water = f32(std::env::args().skip(2).next().unwrap_or("data/DTM_GEWAESSER_R.tif.tif.exr".to_owned()))?;
		
		let ref height = water; // Single terrain for water/ground (`water` has ground altitude for points without water)
		let size = height.size;
		let meters_per_pixel = 8.; // 8x downsample from 1m resolution original = 8m/px
		let size_in_meters = {assert_eq!(size.x, size.y); size.x as f32}*meters_per_pixel;
		let meters_to_normalized = 2./size_in_meters; // shader scales xy integer coordinates [0..`size`] to screen width (=2). i.e: = i/size*2-1
		let [min_height, _max] = minmax(&height.data);
		let z = |height| meters_to_normalized*(height-min_height);
		
		let vertex_grid_size_x = {assert_eq!(size.x, size.y); size.x};
		let vertex_stride = vertex_grid_size_x;
		let vertices = buffer(context, BufferUsage::VERTEX_BUFFER, height.data.len())?;
		{
			let mut vertices = vertices.write()?;
			for y in 1..size.y-1 { for x in 1..size.x-1 {
				let dx_z = (height[xy{x: x+1, y}]-height[xy{x: x-1, y}])/(2.*meters_per_pixel);
				let dy_z = (height[xy{x, y: y+1}]-height[xy{x, y: y-1}])/(2.*meters_per_pixel);
				let n = normalize(cross(xyz{x: 1., y: 0., z: dx_z}, xyz{x: 0., y: 1., z: dy_z}));
				vertices[(y*vertex_stride+x) as usize] = terrain::Vertex{
					z: z(height[xy{x,y}]),
					NdotL: dot(n, xyz{x: 0., y: 0., z: 1.}),
					water: if water[xy{x,y}] > ground[xy{x,y}] { 1. } else { 0. }
				};
			}}
		}
		let mut cell_count = 0;
		let center = size.signed()/2;
		let radius2 = sq(min(size.x, size.y)/2);
		for y in 0..size.y-2 { for x in 0..size.x-2 {
			let r2 = vector::sq(xy{x,y}.signed()-center) as u32;
			if r2 >= radius2 { continue; }
			let i0 = y*vertex_stride+x;
			if [0, 1, vertex_stride, vertex_stride+1].iter().all(|di| height[(i0+di) as usize] > f32::MIN) {} else {continue;}
			cell_count += 1;
		}}
		let grid = buffer(context, BufferUsage::INDEX_BUFFER, (cell_count*6) as usize)?;
		{
			let mut grid = grid.write()?;
			let mut target = 0;
			for y in 0..size.y-2 { for x in 0..size.x-2 {
				let r2 = vector::sq(xy{x,y}.signed()-center) as u32;
				if r2 >= radius2 { continue; }
				let i0 = y*vertex_stride+x;
				if [0, 1, vertex_stride, vertex_stride+1].iter().all(|di| height[(i0+di) as usize] > f32::MIN) {} else {continue;}
				grid[target+0] = i0;
				grid[target+1] = i0+1;
				grid[target+2] = i0+vertex_stride+1;
				grid[target+3] = i0;
				grid[target+4] = i0+vertex_stride+1;
				grid[target+5] = i0+vertex_stride;
				target += 6;
			}}
			assert!(target == grid.len());
		}
		Ok(Self{
			pass: terrain::Pass::new(context)?,
			vertex_grid_size_x,
			grid,
			vertices
		})
	}
	#[throws] pub fn render(&self, context: &Context, commands: &mut Commands, color: Arc<ImageView>, depth: Arc<ImageView>, view_projection: mat4) {
		let Self{pass, vertex_grid_size_x, grid, vertices} = self;
		pass.begin_rendering(context, commands, color, depth, &terrain::Uniforms{
			view_projection: view_projection.map(|column| column.into()).into(), 
			vertex_grid_size_x: *vertex_grid_size_x
		})?;
		commands.bind_index_buffer(grid.clone())?;
		commands.bind_vertex_buffers(0, vertices.clone())?;
		unsafe{commands.draw_indexed(grid.len() as _, 1, 0, 0, 0)}?;
		commands.end_rendering()?;
	}
}