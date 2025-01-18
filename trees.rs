#!/usr/bin/env -S cargo -Zscript
---cargo
package={edition='2024'}
[dependencies]
vector={ git='https://github.com/Matthias-Fauconneau/vector' }
bytemuck={version='*',features=['extern_crate_alloc']}
[profile.dev]
opt-level = 1
---
#![feature(iter_next_chunk)]//, array_try_map, iterator_try_collect)]

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T=(), E=Error> = std::result::Result<T, E>;

fn from_iter<I: Iterator,  const N: usize>(mut iter: I) -> [I::Item; N] where I::Item: std::fmt::Debug { 
	let chunk = iter.next_chunk().unwrap(); 
	assert!(iter.next().is_none()); 
	chunk 
}

use vector::vector;

fn main() -> Result {
	for path in std::env::args().skip(1) {
		println!("read");
		let trees = std::fs::read(&path)?;
		println!("parse {}", trees.len());
		vector!(2 LV95 T T, E N, E N);
		let trees = Box::from_iter(std::str::from_utf8(&trees)?.lines().map(|line| {
			let [_id, min_x, max_x, min_y, max_y] = from_iter(line.split('\t'));
			let [min_x, max_x, min_y, max_y] = [min_x, max_x, min_y, max_y].map(|value| value.parse::<f32>().unwrap());
			LV95{E: (min_x+max_x)/2., N: (min_y+max_y)/2.}
		}));
		println!("write {}", trees.len());
		std::fs::write(format!("{path}.f32"), bytemuck::cast_slice(&trees))?;
	}
	Ok(())
}
