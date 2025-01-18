#!/usr/bin/env -S cargo -Zscript
---cargo
package={edition='2024'}
[dependencies]
bytemuck={version='*',features=['extern_crate_alloc']}
memmap={version='*', package='memmap2'}
vector={git='https://github.com/Matthias-Fauconneau/vector'}
image={git='https://github.com/Matthias-Fauconneau/image', features=['io']}
tiff={git='https://github.com/image-rs/image-tiff'}
[profile.dev]
opt-level = 3
debug = false
debug-assertions = false
overflow-checks = false
panic = 'unwind'
incremental = false
[patch.'https://github.com/image-rs/image-tiff']
tiff={path='../tiff'} 
[patch.'https://github.com/Matthias-Fauconneau/image']
image={path='../image'}
---
#![allow(non_snake_case)] 

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T=(), E=Error> = std::result::Result<T, E>;

use {vector::{vector, xy, vec2, int2, MinMax}, image::{Image, downsample8}};

fn tiff(path: impl AsRef<std::path::Path>, band: usize, cache: Option<impl AsRef<std::path::Path>>) -> Result<Image<Box<[u8]>>> {
	let tiff = unsafe{memmap::Mmap::map(&std::fs::File::open(path)?)?};
	let mut tiff = tiff::decoder::Decoder::new(std::io::Cursor::new(&*tiff))?;
	tiff.seek_to_image(4)?;
	let size = {let (x,y) = tiff.dimensions()?; xy{x: x as u32,y: y as _}};
	println!("{size}");
	let image = if cache.as_ref().is_some_and(|cache| std::fs::exists(cache).unwrap()) { std::fs::read(cache.unwrap())? }
	else {
		assert_eq!(band, 0);
		let tiff::decoder::DecodingResult::U8(image) = tiff.read_image()? else {unimplemented!()};
		if let Some(cache) = cache { std::fs::write(cache, bytemuck::cast_slice(&image))?; }
		image
	}.into_boxed_slice();
	Ok(Image::<Box<[u8]>>::new(size, image))
}

fn main() -> Result {
	for path in std::env::args().skip(1) {
		println!("{path}");
		let image = tiff(&path, 0, Some(format!("{path}.0")))?;
		println!("crop");
		vector!(2 LV95 T T, E N, E N);
		let vec2 = |p| vec2::from(<[f32;2]>::from(p));
		let MinMax{min, max} = MinMax{min: LV95{E: 76000f32, N: 41000.}, max: LV95{E: 89926.4, N: 54926.4}};
		let dtm = MinMax{min: LV95{E: 76224.253, N: 41584.5}, max: LV95{E: 89666.253, N: 54306.5}};
		let scale = vec2::from(image.size)/vec2(max-min);
		let image = image.crop(MinMax{min: int2::from(scale*vec2(dtm.min-min)), max: int2::from(scale*vec2(dtm.max-min))}); // /!\ rounding ~1m
		println!("downsample");
		let image = downsample8::<8>(image);
		println!("flip");
		let mut image = image;
		for y in 0..image.size.y/2 { for x in 0..image.size.x { image.data.swap(image.index(xy{x,y}).unwrap(), image.index(xy{x,y: image.size.y-1-y}).unwrap()) } }
		println!("export");
		image::save_u8(format!("{path}.png"), &image)?;
	}
	Ok(())
}
