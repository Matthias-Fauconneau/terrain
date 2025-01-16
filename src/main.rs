#![feature(slice_from_ptr_range)] // shader
#![allow(incomplete_features)]#![feature(inherent_associated_types)] // shader uniforms
use {ui::{Result, xy, size, int2}, vector::vec2};
use {std::sync::Arc, ui::vulkan::{Context, Commands, ImageView, buffer, from_iter, Subbuffer, BufferUsage, BufferContents, Vertex}};

#[derive(Clone, Copy, BufferContents, Vertex)] #[repr(C)] pub struct Height { #[format(R32_SFLOAT)] pub height: f32 }

ui::shader!{terrain, Height, Terrain}

struct App {
	terrain: Terrain,
	size: size,
	grid: Subbuffer::<[u32]>,
	height: Subbuffer::<[Height]>,
	view_position: vec2,
	yaw: f32,
}

impl App {
	fn new(context: &Context, height: &[f32]) -> Result<Self> {
		let size : size = xy{x:4480, y:4240};
		let vertex_stride = size.x;
		let skip = 4;
		let grid = buffer(context, BufferUsage::INDEX_BUFFER, ((size.y/skip-1)*(size.x/skip-1)*6) as usize)?;
		{
			let mut grid = grid.write()?;
			let index_stride = size.x/skip-1;
			let _start = std::time::Instant::now();
			for y in 0..size.y/skip-1 { for x in 0..size.x/skip-1 {
				let target = ((y*index_stride+x)*6) as usize;
				let i0 = y*skip*vertex_stride+x*skip;
				grid[target+0] = i0;
				grid[target+1] = i0+skip;
				grid[target+2] = i0+vertex_stride*skip+skip;
				grid[target+3] = i0;
				grid[target+4] = i0+vertex_stride*skip+skip;
				grid[target+5] = i0+vertex_stride*skip;
			}}
			//println!("{}ms", _start.elapsed().as_millis());
		}
		//let [Some(&min), Some(&max)] = [height.iter().filter(|&&v| v>=0.).min_by(|a,b| f32::total_cmp(a,b)), height.iter().max_by(|a,b| f32::total_cmp(a,b))] else {unreachable!()};
		let [min, max] = [341.97717f32, 863.59375f32];
		//println!("{}ms", start.elapsed().as_millis());
		Ok(Self{
			terrain: Terrain::new(context)?,
			size,
			grid,
			height: from_iter(context, BufferUsage::VERTEX_BUFFER, height.iter().map(|h| Height{height: (h-min)/(max-min)}))?,
			view_position: xy{x: 0., y: 0.}, yaw: 0.
		})
	}
}

impl ui::Widget for App {
fn paint(&mut self, context/*@Context{device, memory_allocator, ..}*/: &Context, commands: &mut Commands, target: Arc<ImageView>, _: size, _: int2) -> Result<()> {
	let Self{terrain, size, grid, height, view_position, yaw} = self;
	//*view_position += rotate(-*yaw, control);
	let image_size = {let [x,y,_] = target.image().extent(); xy{x,y}};
	terrain.begin_rendering(context, commands, target.clone(), &Terrain::Uniforms{
		grid_size: (*size).into(),
		pitch_sincos: xy::from((std::f32::consts::PI/3.).sin_cos()).into(),
		yaw_sincos: xy::from(yaw.sin_cos()).into(),
		view_position: (*view_position).into(),
		aspect_ratio: image_size.x as f32/image_size.y as f32,
	})?;
	commands.bind_index_buffer(grid.clone())?;
	commands.bind_vertex_buffers(0, height.clone())?;
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
		Ok(Box::new(ui::time!(App::new(context, bytemuck::cast_slice(&height)))?))
	}))
}
