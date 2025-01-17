#!/usr/bin/env -S cargo -Zscript
---cargo
package={edition='2024'}
---
#![allow(non_snake_case)]

fn main() {
	for [latitude/*φ*/, longitude/*λ*/] in [[47.319037,8.446886],[47.435143,8.627216]] {
		let to_arcseconds = |degrees| 3600.*degrees;
		let [latitude, longitude] = [latitude, longitude].map(to_arcseconds);
		// Differences of latitude and longitude relative to Bern in the unit 10000"
		let φ = (latitude-169028.66)/10000.;
		let λ = (longitude-26782.5)/10000.;
		let E = 2600000. + 72.37 + 211455.93 * λ - 10938.51 *λ*φ - 44.54 *λ*λ*λ - 0.36 * λ*φ*φ;
		let N = 1200000. + 147.07 + 308807.95 * φ + 3745.25 *λ*λ + 76.63 *φ*φ - 194.56 *λ*λ*φ + 119.79 *φ*φ*φ;
		println!("{E:.0} {N:.0}");
	}
}
