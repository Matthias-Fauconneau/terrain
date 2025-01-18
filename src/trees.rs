struct DropGuard<T, F: Fn(&T)> { value: T, guard: F }
impl<T, F: Fn(&T)> Drop for DropGuard<T, F> { fn drop(&mut self) { (self.guard)(&self.value) } }
impl<T, F: Fn(&T)> std::ops::Deref for DropGuard<T, F>  { type Target = T; fn deref(&self) -> &Self::Target { &self.value } }
impl<T, F: Fn(&T)> std::ops::DerefMut for DropGuard<T, F>  { fn deref_mut(&mut self) -> &mut Self::Target { &mut self.value } }

use ui::{vulkan, shader};
shader!{trees} // position: vec3
use vulkan::Subbuffer;
pub struct Trees {
	pass: trees::Pass,
	vertices: Subbuffer::<[[f32; /*3*/4/*/!\ alignment*/]]>,
	tree_size: f32,
}

use {ui::Result, vector::{xy, uint2, vec2, mat4, vector, MinMax}, image::{Image, bilinear_sample}};
use vulkan::{Context, from_iter, BufferUsage, Commands, Arc, ImageView, WriteDescriptorSet};
impl Trees {
	pub fn new(context: &Context, ground: &Image<impl std::ops::Deref<Target=[f32]>>, meters_to_normalized: f32, z: impl Fn(f32)->f32) -> Result<Self> {
		let trees = std::fs::read(std::env::args().skip(3).next().unwrap_or("data/rtree_afs_3000vchr.baumbestand_p_geom.csv.f32".to_owned()))?;
		vector!(2 LV95 T T, E N, E N);
		let trees = bytemuck::cast_slice::<_, LV95<f32>>(&trees);
		let vec2 = |p| vec2::from( <[f32;2]>::from(p) );
		let min = LV95{E: 78849.25, N: 43849.5};
		let MinMax{min, max} = MinMax{min, max: min+LV95::from(8192.)};
		let mut plot = DropGuard{
			value: Image::<Box<[f32]>>::zero(xy{x: 1024, y: 1024}),
			guard: |value| image::save_exr("output/plot.exr", "Value", value).unwrap()
		};
		Ok(Self{
			pass: trees::Pass::new(context, true)?,
			vertices: from_iter(context, BufferUsage::STORAGE_BUFFER, trees.iter().map(|p| {
				let normalized_cooordinates = vec2((p-min)/(max-min));
				{let p = uint2::from(normalized_cooordinates*vec2::from(plot.size)); if let Some(pixel) = plot.get_mut(p) { *pixel += 1f32; }}
				let xy{x,y} = 2.* normalized_cooordinates - vec2::from(1.);
				[x, y, z(bilinear_sample(ground, normalized_cooordinates*vec2::from(ground.size-uint2::from(1)))+1.), /*pad:*/0.]
			}))?,
			tree_size: 1.*meters_to_normalized,
		})
	}
	pub fn render(&self, context: &Context, commands: &mut Commands, color: Arc<ImageView>, depth: Arc<ImageView>, 
		view_projection: mat4) -> vulkan::Result {
		let Self{pass, vertices, tree_size} = self;
		pass.begin_rendering(context, commands, color, Some(depth), false, &trees::Uniforms{
			view_projection: view_projection.map(|column| column.into()).into(), 
			tree_size: *tree_size
		}, &[WriteDescriptorSet::buffer(1, vertices.clone())])?;
		unsafe{commands.draw((vertices.len()*6) as _, 1, 0, 0)}?;
		commands.end_rendering()?;
		Ok(())
	}
}
