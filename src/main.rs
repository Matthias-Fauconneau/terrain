#![feature(slice_from_ptr_range)] // shader
#![allow(incomplete_features)]#![feature(inherent_associated_types)] // shader uniforms

use {ui::{Error, throws, Result, xy, uint2, int2, image::{self, bgr}, Image}, vector::vec2};
use {std::sync::Arc, ui::vulkan::{Context, Commands, ImageView, buffer, Subbuffer, BufferUsage, BufferContents, Vertex}};

#[derive(Clone, Copy, BufferContents, Vertex)] #[repr(C)] pub struct Height {
	#[format(R32_SFLOAT)] pub height: f32
}

ui::shader!{terrain, Height, Terrain}

struct App {
	#[allow(dead_code)] height: Image<Box<[u32]>>,
	view_position: vec2,
	yaw: f32,
}

impl App {
	#[throws] fn new(_context: &Context, height: Image<Box<[u32]>>) -> Self { Self{height, view_position: xy{x: 0., y: 0.}, yaw: 0.} }
}

impl ui::Widget for App {
fn paint(&mut self, context/*@Context{device, memory_allocator, ..}*/: &Context, commands: &mut Commands, target: Arc<ImageView>, _: uint2, _: int2) -> Result<()> {
	let Self{height: _, view_position, yaw} = self;
	
	let terrain = Terrain::new(context)?;
	let grid = vec![0,1,2,3];
	let grid = buffer::<u32>(context, BufferUsage::INDEX_BUFFER, grid.len() as _, grid)?;
	
	let height = [Height{height: 1./4.},Height{height: 2./4.},Height{height: 3./4.},Height{height: 1.}];
	let height : Subbuffer::<[Height]> = buffer(context, BufferUsage::VERTEX_BUFFER, height.len() as _, height)?;
	
	//*view_position += rotate(-*yaw, control);
	let size = {let [x,y,_] = target.image().extent(); xy{x,y}};
			
	terrain.begin_rendering(context, commands, target.clone(), &Terrain::Uniforms{
					aspect_ratio: size.x as f32/size.y as f32,
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
	let data = std::env::current_dir()?.ancestors().find_map(|root| std::fs::read(root.join(&name)).ok() ).unwrap();
	let image = Image::new(xy{x:4480, y:4240}, bytemuck::cast_slice::<_,f32>(&data));
	//let [Some(&min), Some(&max)] = [image.data.iter().filter(|&&v| v>=0.).min_by(|a,b| f32::total_cmp(a,b)), image.data.iter().max_by(|a,b| f32::total_cmp(a,b))] else {unreachable!()};
	let [min, max] = [341.97717f32, 863.59375f32];
	let oetf = &image::sRGB8_OETF12;
	//let start = std::time::Instant::now();
	let image = Image::from_iter(image.size, image.data.iter().map(|&v| {
		let v = image::oetf8_12(oetf, ((v-min)/(max-min)).clamp(0., 1.));
		bgr{b: v, g: v, r: v}.into()
	})); // 200ms
	//println!("{}ms", start.elapsed().as_millis());
	ui::run(&name, Box::new(move |context| Ok(Box::new(App::new(context, image)?))))
}