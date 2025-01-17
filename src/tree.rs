ui::shader!{tree} // position: vec3
pub struct Trees {
			pass: tree::Pass,
			vertices: Subbuffer::<[tree::Vertex]>,
}
impl Trees {
	fn new(context: &Context) {
		let trees = std::fs::read(std::env::args().skip(2).next().unwrap_or("data/rtree_afs_3000vchr.baumbestand_p_geom.csv.f32".to_owned()))?;
		vector!(2 LV95 T T, E N, E N);
		let trees = bytemuck::cast_slice::<_, LV95<f32>>(&trees);
		let vec2 = |p| vec2::from( <[f32;2]>::from(p) );
		let MinMax{min, max} = MinMax{min: LV95{E: 2676227., N: 1241397.}, max: LV95{E: 2689664., N: 1254495.}};
		let trees = from_iter(context, BufferUsage::VERTEX_BUFFER, Box::from_iter(trees.iter().map(|p| {
			let normalized_cooordinates = vec2((p-min)/(max-min));
			let xy{x,y} = 2.* normalized_cooordinates - vec2::from(1.);
			tree::Vertex{position: xyz{x,y, z: z(bilinear_sample(&ground, normalized_cooordinates*vec2::from(ground.size-uint2::from(1))))}}
		})))?;
	}
}
