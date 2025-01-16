#!/usr/bin/env -S cargo -Zscript
---cargo
package={edition='2024'}
[dependencies]
memmap={version='*', package='memmap2'}
tiff='*'
bytemuck={version='*',features=['extern_crate_alloc']}
image={git='https://github.com/Matthias-Fauconneau/image', features=['exr']}
[profile.dev]
opt-level = 3
debug = false
debug-assertions = false
overflow-checks = false
panic = 'unwind'
incremental = false
---

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T=(), E=Error> = std::result::Result<T, E>;

use image::{Image, xy};

fn tiff(path: impl AsRef<std::path::Path>, cache: Option<impl AsRef<std::path::Path>>) -> Result<Image<Box<[f32]>>> {
	let tiff = unsafe{memmap::Mmap::map(&std::fs::File::open(path)?)?};
	let mut tiff = tiff::decoder::Decoder::new(std::io::Cursor::new(&*tiff))?.with_limits(tiff::decoder::Limits::unlimited());    
	let size = {let (x,y) = tiff.dimensions()?; xy{x: x as u32,y: y as _}};
	println!("{size}");
	let image = if cache.as_ref().is_some_and(|cache| std::fs::exists(cache).unwrap()) { bytemuck::pod_collect_to_vec(&std::fs::read(cache.unwrap())?).into_boxed_slice() }
	else {
		let stride = size.x;
		let strip_offsets = tiff.find_tag(tiff::tags::Tag::StripOffsets)?.unwrap().into_u64_vec()?;
		let strip_height = tiff.chunk_dimensions().1;
		let mut image = vec![0f32; (((size.y+strip_height-1)/strip_height*strip_height)*stride) as usize].into_boxed_slice();
		for (strip_index, y) in (0..strip_offsets.len()).zip(std::iter::successors(Some(0), |&y| Some(y+strip_height))) {
			print!("."); std::io::Write::flush(&mut std::io::stdout())?;
			tiff.read_chunk_to_buffer(tiff::decoder::DecodingBuffer::F32(&mut image[(y*stride) as usize..]), strip_index as u32, size.x as usize)?;
		}
		if let Some(cache) = cache { std::fs::write(cache, bytemuck::cast_slice(&image))?; }
		image
	};
	Ok(Image::<Box<[f32]>>::new(size, image))
}

pub fn downsample<T: Copy+Into<f32>, D: core::ops::Deref<Target=[T]>, const FACTOR: u32>(ref source: &Image<D>) -> Image<Box<[f32]>> {
	Image::from_xy(source.size/FACTOR, |xy{x,y}|{
			let (count, sum) = (0..FACTOR).map(|dy| (0..FACTOR).map(move |dx| source[xy{x:x*FACTOR+dx,y:y*FACTOR+dy}].into()))
				.flatten().filter(|&value| value > f32::MIN).fold((0, 0f32), |(count, sum), value| (count+1, sum+value));
			if count > 0 { sum / (count as f32) } else { f32::MIN }
	})
}

fn main() -> Result {
	for path in std::env::args().skip(1) {
		let image = tiff(&arg, Some(format!("{path}.f32")))?;
		let size = 8192.into();
		let image = image.slice((image.size-size)/2, size);
		println!("downsample");
		let image = downsample::<_,_,8>(&image);
		println!("flip");
		let mut image = image;
		for y in 0..image.size.y/2 { for x in 0..image.size.x { image.data.swap(image.index(xy{x,y}).unwrap(), image.index(xy{x,y: image.size.y-1-y}).unwrap()) } }
		println!("export");
		image::save_exr(format!("{path}.exr"), "Altitude", &image)?;
	}
	Ok(())
}
