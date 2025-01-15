#![feature(slice_from_ptr_range)] // shader
use ui::{Result, xy, uint2, int2, image::{self, bgr}, Image};
use {std::sync::Arc, ui::vulkan::{Context, Commands, ImageView}};

ui::shader!{terrain, Terrain}

struct App(Image<Box<[u32]>>); 
impl ui::Widget for App {
fn paint(&mut self, context/*@Context{device, memory_allocator, ..}*/: &Context, commands: &mut Commands, target: Arc<ImageView>, _: uint2, _: int2) -> Result<()> {
	let terrain = Terrain::new(context)?;
	terrain.begin_rendering(context, commands, target.clone(), &[])?;
	unsafe{commands.draw(4, 1, 0, 0)}?;
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
	ui::run(&name, &mut App(image))
}