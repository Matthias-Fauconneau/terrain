#![feature(slice_from_ptr_range)] // shader
#![allow(incomplete_features)]#![feature(inherent_associated_types)] // shader uniforms
#![allow(non_snake_case)] // NdotL
mod terrain; use terrain::Terrain;
mod trees; use trees::Trees;

use {ui::{Result, time}, vector::{xy, size, int2, vec2, xyz, vec3, rotate, xyzw, vec4, mat4}};
use ui::vulkan::{Context, Commands, Arc, ImageView, Image, default, ImageCreateInfo, Format, ImageUsage};

struct App {
	terrain: Terrain,
	trees: Trees,
	view_position: vec2,
	yaw: f32,
}

impl App {
	fn new(context: &Context) -> Result<Self> {
		use image::f32;
		let ref ground = f32(std::env::args().skip(1).next().unwrap_or("data/DTM_R.tif.tif.exr".to_owned()))?;
		let ref water = f32(std::env::args().skip(2).next().unwrap_or("data/DTM_GEWAESSER_R.tif.tif.exr".to_owned()))?;
		let meters_per_pixel = 8.; // 8x downsample from 1m resolution original = 8m/px
		let vertex_grid_size_x = {assert_eq!(ground.size.x, ground.size.y); ground.size.x};
		let size_in_meters = vertex_grid_size_x as f32 * meters_per_pixel;
		let meters_to_normalized = 2./size_in_meters; // shader scales xy integer coordinates [0..`size`] to screen width (=2). i.e: = i/size*2-1
		fn minmax(values: &[f32]) -> [f32; 2] {
			let [mut min, mut max] = [f32::INFINITY, -f32::INFINITY];
			for &value in values { if value > f32::MIN && value < min { min = value; } if value > max { max = value; } }
			[min, max]
		}
		let [min_height, _max] = minmax(&water.data);
		let z = |height| meters_to_normalized*(height-min_height);
		Ok(Self{
			terrain: Terrain::new(context, ground, water, meters_per_pixel, z)?,
			trees: Trees::new(context, /*ground*/water, meters_to_normalized, z)?,
			view_position: xy{x: 0., y: 0.}, yaw: 0.
		})
	}
}

impl ui::Widget for App {
fn paint(&mut self, context@Context{memory_allocator, ..}: &Context, commands: &mut Commands, target: Arc<ImageView>, _: size, _: int2) -> Result<()> {
	let Self{terrain, trees, view_position, yaw} = self;
	//*view_position += rotate(-*yaw, control);
	let image_size = {let [x,y,_] = target.image().extent(); xy{x,y}};
	let aspect_ratio = image_size.x as f32/image_size.y as f32;
	
	let view_projection = |xyz{x,y,z}:vec3| {
		let xy{x,y} = rotate(*yaw, xy{x,y} - *view_position);
		let xy{x: y, y: z} = rotate(-std::f32::consts::PI/3., xy{x: y, y: z});
		let z = (z-1.)/2.;
		let n = 1./4.;
		let f = 1.;
		if true { xyzw{x, y: aspect_ratio*y, z: -z, w: 1.}  } else {
			let zz = -f/(f-n);
			let z1 = -(f*n)/(f-n);
			xyzw{x, y: aspect_ratio*y, z: zz*z+z1, w: -z}
		}
	};
	fn from_linear(linear: impl Fn(vec3)->vec4) -> mat4 {
		let w = linear(xyz{x:0.,y:0.,z:0.});
		let xyz{x,y,z} = xyz{x: xyz{x:1.,y:0.,z:0.}, y:xyz{x:0.,y:1.,z:0.}, z: xyz{x:0.,y:0.,z:1.}}.map(|e| linear(e)-w);
		xyzw{x,y,z,w}
	}
	let view_projection = from_linear(view_projection);
	
	let depth = ImageView::new_default(Image::new(memory_allocator.clone(), ImageCreateInfo{
		format: Format::D16_UNORM,
		extent: target.image().extent(),
		usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT,
		..default()
	}, default())?)?;
	
	terrain.render(context, commands, target.clone(), depth.clone(), view_projection)?;
	trees.render(context, commands, target.clone(), depth.clone(), view_projection)?;
	
	*yaw += std::f32::consts::PI/6./60.;
	Ok(())
}
fn event(&mut self, _size: size, _context: &mut ui::EventContext, _event: &ui::Event) -> Result<bool> { Ok(true/*Keep repainting*/) }
}

fn main() -> Result { ui::run("terrain", Box::new(move |context,_commands| Ok(Box::new(time("init", || App::new(context))?)))) }
