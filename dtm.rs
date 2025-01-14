#!/usr/bin/env -S cargo -Zscript
---cargo
package={edition='2024'}
dependencies={ui={git='https://github.com/Matthias-Fauconneau/ui'},bytemuck='*'}
patch.'https://github.com/Matthias-Fauconneau/ui'={ui={path='../ui'}}
[profile.dev]
opt-level = 1
---
#![feature(optimize_attribute)]
use ui::{Result, xy, image::{self, bgr}, Image};
struct App(Image<Box<[u32]>>); 
impl ui::Widget for App { 
	fn paint(&mut self, target: &mut ui::Target, _: ui::uint2, _: ui::int2) -> Result<()> {
		let ref source = self.0;
		for y in 0..target.size.y { 
			for x in 0..target.size.x {
				target[xy{x,y}] = source[xy{x: x*(source.size.x-1)/(target.size.x-1), y: y*(source.size.y-1)/(target.size.y-1)}]; 
			}
		}
		Ok(())
	}
}
#[optimize(speed)] fn main() -> Result {
	let dtm = std::env::args().skip(1).next().unwrap_or("data/DTM_R.tif.tif.exr".to_owned());
	let mmap = format!("{dtm}.f32");
	/*#[cfg(feature="exr")] if !std::fs::exists(&mmap)? {
		let start = std::time::Instant::now();
		let image = image::f32(dtm)?;
		println!("{}ms {}", start.elapsed().as_millis(), image.size);
		std::fs::write(&mmap, bytemuck::cast_slice(&image.data))?;
	}*/
	let mmap = std::fs::read(mmap)?;
	let image = Image::new(xy{x:4480, y:4240}, bytemuck::cast_slice::<_,f32>(&mmap));
	//let [Some(&min), Some(&max)] = [image.data.iter().filter(|&&v| v>=0.).min_by(|a,b| f32::total_cmp(a,b)), image.data.iter().max_by(|a,b| f32::total_cmp(a,b))] else {unreachable!()};
	let [min, max] = [341.97717f32, 863.59375f32];
	let oetf = &image::sRGB8_OETF12;
	//let start = std::time::Instant::now();
	let image = Image::from_iter(image.size, image.data.iter().map(|&v| {
		let v = image::oetf8_12(oetf, ((v-min)/(max-min)).clamp(0., 1.));
		bgr{b: v, g: v, r: v}.into()
	})); // 200ms
	//println!("{}ms", start.elapsed().as_millis());
	ui::run(&dtm, &mut App(image)) 
}
