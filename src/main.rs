#![feature(slice_from_ptr_range)] // shader
#![allow(incomplete_features)]#![feature(inherent_associated_types)] // shader uniforms
#![allow(non_snake_case)] // NdotL
use {ui::{default, Result, time}, vector::{xy, size, int2, vec2, xyz, cross, normalize, dot}, ui::Image};
use std::sync::Arc;
use ui::vulkan::{self, Context, Commands, Format, ImageUsage, ImageCreateInfo, ImageView, buffer, Subbuffer, BufferUsage, BufferContents, Vertex};

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
	fn new(context: &Context, height: Image<&[f32]>) -> Result<Self> {
		//let [Some(&min), Some(&max)] = [height.iter().filter(|&&v| v>=0.).min_by(|a,b| f32::total_cmp(a,b)), height.iter().max_by(|a,b| f32::total_cmp(a,b))] else {unreachable!()};
		let [min, max] = [341.97717f32, 863.59375f32];
		
		let size = height.size;
		let vertex_stride = size.x;
		let vertices = buffer(context, BufferUsage::VERTEX_BUFFER, height.data.len())?;
		{
			let mut vertices = vertices.write()?;
			time("vertex", || 
				for y in 1..size.y-1 { for x in 1..size.x-1 {
					let dx_z = (height[xy{x: x+1, y}]-height[xy{x: x-1, y}])/2.;
					let dy_z = (height[xy{x, y: y+1}]-height[xy{x, y: y-1}])/2.;
					let n = normalize(cross(xyz{x: 1., y: 0., z: dx_z}, xyz{x: 0., y: 1., z: dy_z}));
					vertices[(y*vertex_stride+x) as usize] = TerrainVertex{
						height: (height[xy{x,y}]-min)/(max-min)/2.,
						NdotL: dot(n, xyz{x: 0., y: 0., z: 1.})
					};
				}}
			);
		}
		let skip = 4;
		let mut cell_count = 0;
		for y in 0..size.y/skip-1 { for x in 0..size.x/skip-1 {
			let i0 = y*skip*vertex_stride+x*skip;
			if [0, skip, vertex_stride*skip, vertex_stride*skip+skip].iter().all(|di| height[(i0+di) as usize] > 0.) { cell_count += 1; }
		}}
		let grid = buffer(context, BufferUsage::INDEX_BUFFER, (cell_count*6) as usize)?;
		{
			let mut grid = grid.write()?;
			let mut target = 0;
			for y in 0..size.y/skip-1 { for x in 0..size.x/skip-1 {
				let i0 = y*skip*vertex_stride+x*skip;
				if [0, skip, vertex_stride*skip, vertex_stride*skip+skip].iter().all(|di| height[(i0+di) as usize] > 0.) {
					grid[target+0] = i0;
					grid[target+1] = i0+skip;
					grid[target+2] = i0+vertex_stride*skip+skip;
					grid[target+3] = i0;
					grid[target+4] = i0+vertex_stride*skip+skip;
					grid[target+5] = i0+vertex_stride*skip;
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
	let name = format!("{}.f32", std::env::args().skip(1).next().unwrap_or("data/DTM_R.tif.tif.exr".to_owned()));
	ui::run(&name.clone(), Box::new(move |context| {
		let height = std::env::current_dir()?.ancestors().find_map(|root| ui::time!(std::fs::read(root.join(&name))).ok() ).expect(&name);
		Ok(Box::new(time("init", || App::new(context, Image::new(xy{x:4480, y:4240}, bytemuck::cast_slice(&height))))?))
	}))
}
