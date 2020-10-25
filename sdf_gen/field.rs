use freetype::{
	self,
	face::LoadFlag,
	outline::Curve
};
use std::f64::consts::PI;
use crate::Glyph;

struct Vector { x: f64, y: f64 }

pub fn load_glyphs_and_generate_sdfs(font_file_path: &str, size: u32, spread: usize) -> (Vec<Glyph>, f32) {
	let spread_f64 = spread as f64;

	let char_codes = 33..127;
	let mut glyphs: Vec<Glyph> = Vec::with_capacity(char_codes.len());

	// Initialize freetype
	let library = freetype::Library::init().unwrap();
	let face = library.new_face(font_file_path, 0).unwrap();
	face.set_pixel_sizes(0, size).unwrap();

	// Get space advance
	face.load_char(32, LoadFlag::NO_HINTING).unwrap();
	let space_advance = face.glyph().metrics().horiAdvance as f32 / 64.0;

	// Iterate over characters
	println!("Generating {} signed distance fields", char_codes.len());
	for char_code in char_codes {
		// Load character and gather metrics
		face.load_char(char_code, LoadFlag::empty()).unwrap();
		let metrics = face.glyph().metrics();
		let width = metrics.width as usize / 64 + spread * 2;
		let height = metrics.height as usize / 64 + spread * 2;
		let left_edge_padded = metrics.horiBearingX as f64 / 64.0 - spread_f64;
		let top_edge_padded = metrics.horiBearingY as f64 / 64.0 + spread_f64;
		let outline = face.glyph().outline().unwrap();

		let mut field = Vec::with_capacity(height);
		
		// Iterate over texels in our distance field
		for row in 0..height {
			let mut field_row = Vec::with_capacity(width);

			for col in 0..width {
				let mut min_dist = f64::MAX;
				let mut total_cross_num = 0;
				let p = Vector {
					x: left_edge_padded + col as f64 + 0.5,
					y: top_edge_padded - row as f64 - 0.5
				};

				// Find the minimum distance from p to each curve
				for contour in outline.contours_iter() {
					let mut start = *contour.start();

					for curve in contour {
						let s = Vector {
							x: start.x as f64 / 64.0,
							y: start.y as f64 / 64.0
						};

						let (end, dist, cross_num) = match curve {
							Curve::Line(end) => {
								let e = Vector {
									x: end.x as f64 / 64.0,
									y: end.y as f64 / 64.0
								};

								let dist = find_dist_to_line(&p, &s, &e);
								let cross_num = find_cross_num_of_line(&p, &s, &e);

								(end, dist, cross_num)
							},
							Curve::Bezier2(control, end) => {
								let c = Vector {
									x: control.x as f64 / 64.0,
									y: control.y as f64 / 64.0
								};

								let e = Vector {
									x: end.x as f64 / 64.0,
									y: end.y as f64 / 64.0
								};
								
								let dist = find_dist_to_bezier(&p, &s, &c, &e);
								let cross_num = find_cross_num_of_bezier(&p, &s, &c, &e);
								
								(end, dist, cross_num)
							},
							Curve::Bezier3(_, _, _) => {
								panic!("cubic beziers not supported");
							}
						};

						if dist < min_dist {
							min_dist = dist;
						}

						total_cross_num += cross_num;
						start = end;
					}
				}

				// Clamp the signed distance to the spread and normalize it to a u8
				let dist_signed = if total_cross_num % 2 == 0 { -min_dist } else { min_dist };
				let dist_clamped = dist_signed.min(spread_f64).max(-spread_f64);
				let dist_positive = dist_clamped + spread_f64;
				let dist_scaled = (dist_positive * 255.0 / (spread_f64 * 2.0)).round() as u8;
				
				field_row.push(dist_scaled);
			}

			field.push(field_row);
		}
		
		glyphs.push(Glyph {
			char_code: char_code as u32,
			width: metrics.width as u32 / 64,
			height: metrics.height as u32 / 64,
			bearing_x: metrics.horiBearingX / 64,
			bearing_y: metrics.horiBearingY / 64,
			advance: metrics.horiAdvance / 64,
			field,
			position_x: 0,
			position_y: 0
		});
	}

	(glyphs, space_advance)
}

fn find_dist_to_line(p: &Vector, s: &Vector, e: &Vector) -> f64 {
	// Ignore if this is a point
	if s.x == e.x && s.y == e.y {
		return f64::MAX;
	}

	// Find the distance along the line the projected point lies
	let diff = Vector { x: e.x - s.x, y: e.y - s.y };
	let dot = (p.x - s.x) * diff.x + (p.y - s.y) * diff.y;
	let t = dot / (diff.x * diff.x + diff.y * diff.y);

	// Ignore if projected point is before start of line segment
	if t <= 0.0 {
		return f64::MAX;
	}

	// Project the point onto the line segment
	let t = t.min(1.0);
	let proj = Vector { x: s.x + t * diff.x, y: s.y + t * diff.y };

	// Find the distance
	let proj_diff = Vector { x: proj.x - p.x, y: proj.y - p.y };
	(proj_diff.x * proj_diff.x + proj_diff.y * proj_diff.y).sqrt()
}

fn find_cross_num_of_line(p: &Vector, s: &Vector, e: &Vector) -> u32 {
	let diff = Vector { x: e.x - s.x, y: e.y - s.y };

	// Ignore if line segment is horizontal
	if diff.y == 0.0 {
		return 0;
	}

	// Find the single crossing point
	let t = (p.y - s.y) / diff.y;
	let x = s.x + t * diff.x;

	// Count if crossing point is to the right and if one of the following is true where its
	// - Between the endpoints
	// - At the start of an upward line segment
	// - At the end of a downward line segment
	if x > p.x && ((t > 0.0 && t < 1.0) || (t == 0.0 && diff.y.is_sign_positive()) || (t == 1.0 && diff.y.is_sign_negative())) {
		1
	}
	else {
		0
	}
}

fn find_dist_to_bezier(p: &Vector, s: &Vector, c: &Vector, e: &Vector) -> f64 {
	let sc = Vector { x: c.x - s.x, y: c.y - s.y };
	let ps = Vector { x: s.x - p.x, y: s.y - p.y };

	// Find cubic polynomial coefficients
	let a = Vector { x: s.x - 2.0 * c.x + e.x, y: s.y - 2.0 * c.y + e.y };
	let d4 = a.x * a.x + a.y * a.y;
	let d3 = a.x * sc.x + a.y * sc.y;
	let d2 = a.x * ps.x + a.y * ps.y + 2.0 * sc.x * sc.x + 2.0 * sc.y * sc.y;
	let d1 = sc.x * ps.x + sc.y * ps.y;
	let d0 = ps.x * ps.x + ps.y * ps.y;

	// Find depressed cubic coefficients
	let dp = (d4 * d2 - 3.0 * d3 * d3) / (d4 * d4);
	let dq = (2.0 * d3 * d3 * d3 - d4 * d3 * d2 + d4 * d4 * d1) / (d4 * d4 * d4);

	// Find roots of depressed cubic
	let discriminant = 4.0 * dp * dp * dp + 27.0 * dq * dq;
	let depressed_roots = if dp == 0.0 && dq == 0.0 {
		// 0 is the single solution
		vec![0.0]
	}
	else if discriminant > 0.0 {
		// Find the single solution using Cardano's formula
		let a = -dq / 2.0;
		let b = (discriminant / 108.0).sqrt();
		vec![(a + b).cbrt() + (a - b).cbrt()]
	}
	else if discriminant < 0.0 {
		// Find the three solutions using trigonometry
		let a = 2.0 * (-dp / 3.0).sqrt();
		let b = 1.0 / 3.0 * ((3.0 * dq) / (2.0 * dp) * (-3.0 / dp).sqrt()).acos();
		(0..3).into_iter().map(|k| a * (b - (2.0 * PI * k as f64 / 3.0)).cos()).collect()
	}
	else {
		// Find the two solutions
		let a = 3.0 * dq;
		vec![a / dp, -a / (2.0 * dp)]
	};

	// Find minimum distance using the roots
	let mut min_dist = f64::MAX;
	for root in depressed_roots {
		// Map depressed root to a root of the original cubic polynomial
		let t = root - d3 / d4;

		// Ignore if point is before start of curve
		if t <= 0.0 {
			continue;
		}

		// Find distance using the 4th degree polynomial
		let t = t.min(1.0);
		let dist = (d4 * t.powf(4.0) + 4.0 * d3 * t.powf(3.0) + 2.0 * d2 * t.powf(2.0) + 4.0 * d1 * t + d0).sqrt();

		// Compare with current minimal distance
		if dist < min_dist {
			min_dist = dist;
		}
	}

	min_dist
}

fn find_cross_num_of_bezier(p: &Vector, s: &Vector, c: &Vector, e: &Vector) -> u32 {
	let u = s.y - 2.0 * c.y + e.y;

	// Determine if y varies linearly
	if u == 0.0 {
		// Find single crossing point
		let diff = e.y - s.y;
		let t = (p.y - s.y) / diff;
		let a = 1.0 - t;
		let x = a * a * s.x + 2.0 * a * t * c.x + t * t * e.x;

		// Count if crossing point is to the right and if one of the following is true where its
		// - Between the endpoints
		// - At the start of an upward line segment
		// - At the end of a downward line segment
		return if x > p.x && ((t > 0.0 && t < 1.0) || (t == 0.0 && diff.is_sign_positive()) || (t == 1.0 && diff.is_sign_negative())) {
			1
		}
		else {
			0
		}
	}
	
	let w = p.y * s.y - 2.0 * p.y * c.y + p.y * e.y - s.y * e.y + c.y * c.y;

	// If w is negative, there are no solutions
	if w.is_sign_negative() {
		return 0;
	}

	// Find two crossing points
	let w = w.sqrt();
	let v = s.y - c.y;

	let t1 = (v + w) / u;
	let a1 = 1.0 - t1;
	let x1 = a1 * a1 * s.x + 2.0 * a1 * t1 * c.x + t1 * t1 * e.x;

	let t2 = (v - w) / u;
	let a2 = 1.0 - t2;
	let x2 = a2 * a2 * s.x + 2.0 * a2 * t2 * c.x + t2 * t2 * e.x;

	// Find curve direction
	let s_dir = if s.y == c.y { e.y - s.y } else { c.y - s.y };
	let e_dir = if c.y == e.y { e.y - s.y } else { e.y - c.y };

	// Find crossing number
	if t1 == t2 {
		// There are two equal solutions, count if crossing point is to the right and if one of the following is true where its
		// - At the start of an upward curve
		// - At the end of a downward curve
		if x1 > p.x && ((t1 == 0.0 && s_dir.is_sign_positive()) || (t1 == 1.0 && e_dir.is_sign_negative())) {
			1
		}
		else {
			0
		}
	}
	else {
		// There are two distinct solutions, count if crossing point is to the right and if one of the following is true where its
		// - Between the endpoints
		// - At the start of an upward curve
		// - At the end of a downward curve
		let mut count = 0;

		if x1 > p.x && ((t1 > 0.0 && t1 < 1.0) || (t1 == 0.0 && s_dir.is_sign_positive()) || (t1 == 1.0 && e_dir.is_sign_negative())) {
			count += 1;
		}

		if x2 > p.x && ((t2 > 0.0 && t2 < 1.0) || (t2 == 0.0 && s_dir.is_sign_positive()) || (t2 == 1.0 && e_dir.is_sign_negative())) {
			count += 1;
		}

		count
	}
}