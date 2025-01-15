#![feature(slice_from_ptr_range)] // shader
#![allow(incomplete_features)]#![feature(inherent_associated_types)] // shader uniforms

use {ui::{Error, throws, Result, xy, size, int2}, vector::vec2};
use {std::sync::Arc, ui::vulkan::{Context, Commands, ImageView, buffer, Subbuffer, BufferUsage, BufferContents, Vertex}};

#[derive(Clone, Copy, BufferContents, Vertex)] #[repr(C)] pub struct Height { #[format(R32_SFLOAT)] pub height: f32 }

ui::shader!{terrain, Height, Terrain}

struct App {
	size: size,
	height: Subbuffer::<[Height]>,
	view_position: vec2,
	yaw: f32,
}

impl App {
	#[throws] fn new(context: &Context, height: &[f32]) -> Self { 
		//let [Some(&min), Some(&max)] = [height.iter().filter(|&&v| v>=0.).min_by(|a,b| f32::total_cmp(a,b)), height.iter().max_by(|a,b| f32::total_cmp(a,b))] else {unreachable!()};
		let [min, max] = [341.97717f32, 863.59375f32];
		//println!("{}ms", start.elapsed().as_millis());
		Self{
			size: xy{x:4480, y:4240},
			height: buffer(context, BufferUsage::VERTEX_BUFFER, height.len() as _, height.iter().map(|h| Height{height: (h-min)/(max-min)}))?,
			view_position: xy{x: 0., y: 0.}, yaw: 0.
		}
	}
}

impl ui::Widget for App {
fn paint(&mut self, context/*@Context{device, memory_allocator, ..}*/: &Context, commands: &mut Commands, target: Arc<ImageView>, _: size, _: int2) -> Result<()> {
	let Self{size, height, view_position, yaw} = self;
	
	let terrain = Terrain::new(context)?;
	let stride = size.x;
	let grid = buffer(context, BufferUsage::INDEX_BUFFER, (size.y-1)*(size.x-1)*6, 
		(0..size.y-1).map(|y| (0..size.x-1).map(move |x| {
			let i0 = y*stride+x;
			[i0, i0+1, i0+stride+1, i0, i0+stride+1, i0+stride].into_iter()
		})).flatten().flatten()
	)?;

	//*view_position += rotate(-*yaw, control);
	let image_size = {let [x,y,_] = target.image().extent(); xy{x,y}};
			
	terrain.begin_rendering(context, commands, target.clone(), &Terrain::Uniforms{
		grid_size: (*size).into(),
		aspect_ratio: image_size.x as f32/image_size.y as f32,
		view_position: (*view_position).into(),
		yaw_sincos: xy::from(yaw.sin_cos()).into()
	})?;
	commands.bind_vertex_buffers(0, height.clone())?;
	commands.bind_index_buffer(grid.clone())?;
	unsafe{commands.draw_indexed(grid.len() as _, 1, 0, 0, 0)}?;
	commands.end_rendering()?;
	Ok(())
}
}

fn main() -> Result {
	let name = format!("{}.f32", std::env::args().skip(1).next().unwrap_or("data/DTM_R.tif.tif.exr".to_owned()));
	ui::run(&name.clone(), Box::new(move |context| {
			let height = std::env::current_dir()?.ancestors().find_map(|root| std::fs::read(root.join(&name)).ok() ).unwrap();
			Ok(Box::new(App::new(context, bytemuck::cast_slice(&height))?))
	}))
}