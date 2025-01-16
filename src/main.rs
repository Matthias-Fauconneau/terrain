#![feature(slice_from_ptr_range)] // shader
#![allow(incomplete_features)]#![feature(inherent_associated_types)] // shader uniforms
#![allow(non_snake_case)] // NdotL
use {ui::{default, Result, time}, std::cmp::min, num::sq, vector::{xy, size, int2, vec2, xyz, cross, normalize, dot}, image::{Image, f32}};
use std::sync::Arc;
use ui::vulkan::{self, Context, Commands, Format, ImageUsage, ImageCreateInfo, ImageView, buffer, Subbuffer, BufferUsage, BufferContents, Vertex};

fn minmax(values: &[f32]) -> [f32; 2] {
	let [mut min, mut max] = [f32::INFINITY, -f32::INFINITY];
	for &value in values { if value > f32::MIN && value < min { min = value; } if value > max { max = value; } }
	[min, max]
}

#[derive(Clone, Copy, BufferContents, Vertex)] #[repr(C)] pub struct TerrainVertex { 
	#[format(R32_SFLOAT)] pub height: f32,
	#[format(R32_SFLOAT)] pub NdotL: f32,
}

ui::shader!{terrain, TerrainVertex, Terrain}

struct App {
	terrain: Terrain,
	size: size,
	grid: Subbuffer::<[u32]>,
	vertices: Subbuffer::<[TerrainVertex]>,
	view_position: vec2,
	yaw: f32,
}

impl App {
	fn new(context: &Context, height: Image<impl AsRef<[f32]>>) -> Result<Self> {
		let height = height.as_ref();
		
		let size = height.size;
		let vertex_stride = size.x;
		let vertices = buffer(context, BufferUsage::VERTEX_BUFFER, height.data.len())?;
		{
			let [min, max] = minmax(height.data);

			let mut vertices = vertices.write()?;
			for y in 1..size.y-1 { for x in 1..size.x-1 {
				let dx_z = (height[xy{x: x+1, y}]-height[xy{x: x-1, y}])/2.;
				let dy_z = (height[xy{x, y: y+1}]-height[xy{x, y: y-1}])/2.;
				let n = normalize(cross(xyz{x: 1., y: 0., z: dx_z}, xyz{x: 0., y: 1., z: dy_z}));
				vertices[(y*vertex_stride+x) as usize] = TerrainVertex{
					height: (height[xy{x,y}]-min)/(max-min)/2.,
					NdotL: dot(n, xyz{x: 0., y: 0., z: 1.})
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
			if [0, 1, vertex_stride, vertex_stride+1].iter().all(|di| height[(i0+di) as usize] > f32::MIN) { cell_count += 1; }
		}}
		let grid = buffer(context, BufferUsage::INDEX_BUFFER, (cell_count*6) as usize)?;
		{
			let mut grid = grid.write()?;
			let mut target = 0;
			for y in 0..size.y-2 { for x in 0..size.x-2 {
				let r2 = vector::sq(xy{x,y}.signed()-center) as u32;
				if r2 >= radius2 { continue; }
				let i0 = y*vertex_stride+x;
				if [0, 1, vertex_stride, vertex_stride+1].iter().all(|di| height[(i0+di) as usize] > f32::MIN) {
					grid[target+0] = i0;
					grid[target+1] = i0+1;
					grid[target+2] = i0+vertex_stride+1;
					grid[target+3] = i0;
					grid[target+4] = i0+vertex_stride+1;
					grid[target+5] = i0+vertex_stride;
					target += 6;
				}
			}}
			assert!(target == grid.len());
		}
		Ok(Self{
			terrain: Terrain::new(context)?,
			size,
			grid,
			vertices,
			view_position: xy{x: 0., y: 0.}, yaw: 0.
		})
	}
}

impl ui::Widget for App {
fn paint(&mut self, context@Context{memory_allocator, ..}: &Context, commands: &mut Commands, target: Arc<ImageView>, _: size, _: int2) -> Result<()> {
	let Self{terrain, size, grid, vertices, view_position, yaw} = self;
	//*view_position += rotate(-*yaw, control);
	let image_size = {let [x,y,_] = target.image().extent(); xy{x,y}};
	let depth = vulkan::Image::new(memory_allocator.clone(), ImageCreateInfo{
		format: Format::D16_UNORM,
		extent: target.image().extent(),
		usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT,
		..default()
	}, default())?;
	terrain.begin_rendering(context, commands, target.clone(), ImageView::new_default(depth)?, &Terrain::Uniforms{
		grid_size: (*size).into(),
		pitch_sincos: xy::from((std::f32::consts::PI/4.).sin_cos()).into(),
		yaw_sincos: xy::from(yaw.sin_cos()).into(),
		view_position: (*view_position).into(),
		aspect_ratio: image_size.x as f32/image_size.y as f32,
	})?;
	commands.bind_index_buffer(grid.clone())?;
	commands.bind_vertex_buffers(0, vertices.clone())?;
	unsafe{commands.draw_indexed(grid.len() as _, 1, 0, 0, 0)}?;
	commands.end_rendering()?;
	*yaw += std::f32::consts::PI/60.;
	Ok(())
}
fn event(&mut self, _size: size, _context: &mut ui::EventContext, _event: &ui::Event) -> Result<bool> { Ok(true/*Keep repainting*/) }
}

fn main() -> Result {
	let path = std::env::args().skip(1).next().unwrap_or("data/DTM_R.tif.tif.exr".to_owned());
	ui::run(&path.clone(), Box::new(move |context| Ok(Box::new(time("init", || App::new(context, f32(path)?))?))))
}
