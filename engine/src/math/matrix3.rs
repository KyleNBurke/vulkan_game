use super::{Vector2, ApproxEq};
use auto_ops::impl_op_ex;

const IDENTITY: [[f32; 3]; 3] = [
	[1.0, 0.0, 0.0],
	[0.0, 1.0, 0.0],
	[0.0, 0.0, 1.0]
];

#[derive(Default, Copy, Clone, Debug, PartialEq)]
pub struct Matrix3 {
	pub elements: [[f32; 3]; 3]
}

impl Matrix3 {
	pub fn new() -> Self {
		Self { elements: IDENTITY }
	}

	pub fn from(elements: [[f32; 3]; 3]) -> Self {
		Self { elements }
	}

	pub fn set(&mut self, elements: [[f32; 3]; 3]) {
		self.elements = elements;
	}

	pub fn to_padded_array(&self) -> [[f32; 4]; 3] {
		let e = &self.elements;

		[
			[e[0][0], e[0][1], e[0][2], 0.0],
			[e[1][0], e[1][1], e[1][2], 0.0],
			[e[2][0], e[2][1], e[2][2], 0.0],
		]
	}

	pub fn compose(&mut self, position: &Vector2, rotation: f32, scale: &Vector2) {
		self.elements = [
			[rotation.cos() * scale.x, -rotation.sin(), position.x],
			[rotation.sin(), rotation.cos() * scale.y, position.y],
			[0.0, 0.0, 1.0]
		];
	}
}

impl_op_ex!(+ |a: &Matrix3, b: &Matrix3| -> Matrix3 {
	let mut r = *a;
	r += b;
	r
});

impl_op_ex!(- |a: &Matrix3, b: &Matrix3| -> Matrix3 {
	let mut r = *a;
	r -= b;
	r
});

impl_op_ex!(* |a: &Matrix3, b: &Matrix3| -> Matrix3 {
	let mut r = *a;
	r *= b;
	r
});

impl_op_ex!(+= |a: &mut Matrix3, b: &Matrix3| {
	let ae = &mut a.elements;
	let be = &b.elements;

	ae[0][0] += be[0][0];
	ae[0][1] += be[0][1];
	ae[0][2] += be[0][2];

	ae[1][0] += be[1][0];
	ae[1][1] += be[1][1];
	ae[1][2] += be[1][2];

	ae[2][0] += be[2][0];
	ae[2][1] += be[2][1];
	ae[2][2] += be[2][2];
});

impl_op_ex!(-= |a: &mut Matrix3, b: &Matrix3| {
	let ae = &mut a.elements;
	let be = &b.elements;

	ae[0][0] -= be[0][0];
	ae[0][1] -= be[0][1];
	ae[0][2] -= be[0][2];

	ae[1][0] -= be[1][0];
	ae[1][1] -= be[1][1];
	ae[1][2] -= be[1][2];

	ae[2][0] -= be[2][0];
	ae[2][1] -= be[2][1];
	ae[2][2] -= be[2][2];
});

impl_op_ex!(*= |a: &mut Matrix3, b: &Matrix3| {
	let ae = &a.elements;
	let be = &b.elements;

	let c00 = ae[0][0] * be[0][0] + ae[0][1] * be[1][0] + ae[0][2] * be[2][0];
	let c01 = ae[0][0] * be[0][1] + ae[0][1] * be[1][1] + ae[0][2] * be[2][1];
	let c02 = ae[0][0] * be[0][2] + ae[0][1] * be[1][2] + ae[0][2] * be[2][2];

	let c10 = ae[1][0] * be[0][0] + ae[1][1] * be[1][0] + ae[1][2] * be[2][0];
	let c11 = ae[1][0] * be[0][1] + ae[1][1] * be[1][1] + ae[1][2] * be[2][1];
	let c12 = ae[1][0] * be[0][2] + ae[1][1] * be[1][2] + ae[1][2] * be[2][2];

	let c20 = ae[2][0] * be[0][0] + ae[2][1] * be[1][0] + ae[2][2] * be[2][0];
	let c21 = ae[2][0] * be[0][1] + ae[2][1] * be[1][1] + ae[2][2] * be[2][1];
	let c22 = ae[2][0] * be[0][2] + ae[2][1] * be[1][2] + ae[2][2] * be[2][2];

	a.elements = [
		[c00, c01, c02],
		[c10, c11, c12],
		[c20, c21, c22]
	];
});

impl ApproxEq for Matrix3 {
	fn approx_eq(&self, other: &Self, tol: f32) -> bool {
		for i in 0..3 {
			for j in 0..3 {
				if (self.elements[i][j] - other.elements[i][j]).abs() > tol {
					return false;
				}
			}
		}

		true
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::math::assert_approx_eq;

	#[test]
	fn new() {
		assert_eq!(Matrix3::new().elements, IDENTITY);
	}

	#[test]
	fn from() {
		let elements = [
			[0.0, 0.1, 0.2],
			[1.0, 1.1, 1.2],
			[2.0, 2.1, 2.2]
		];

		let m = Matrix3::from(elements);
		assert_eq!(m.elements, elements);
	}

	#[test]
	fn set() {
		let elements = [
			[1.0, 2.0, 3.0],
			[4.0, 5.0, 6.0],
			[7.0, 8.0, 9.0]
		];

		let mut m = Matrix3::new();
		m.set(elements);

		assert_eq!(m.elements, elements);
	}

	#[test]
	fn to_padded_array() {
		let m = Matrix3::from([
			[1.0, 2.0, 3.0],
			[4.0, 5.0, 6.0],
			[7.0, 8.0, 9.0]
		]);

		let expected = [
			[1.0, 2.0, 3.0, 0.0],
			[4.0, 5.0, 6.0, 0.0],
			[7.0, 8.0, 9.0, 0.0]
		];

		assert_eq!(m.to_padded_array(), expected);
	}

	#[test]
	fn compose() {
		let pos = Vector2::from(100.0, 200.0);
		let rot = std::f32::consts::PI;
		let scale = Vector2::from(3.0, 4.0);
		let mut m = Matrix3::new();
		m.compose(&pos, rot, &scale);

		let expected = Matrix3::from([
			[-3.0, 0.0, 100.0],
			[0.0, -4.0, 200.0],
			[0.0, 0.0, 1.0]
		]);

		assert_approx_eq(&m, &expected, 1e-6);
	}

	#[test]
	fn add() {
		let a = Matrix3::from([
			[4.0, 2.0, 8.0],
			[7.0, 1.0, 9.0],
			[0.0, 2.0, 6.0]
		]);

		let b = Matrix3::from([
			[9.0, 0.0, 4.0],
			[7.0, 6.0, 9.0],
			[0.0, 9.0, 1.0]
		]);

		let expected = Matrix3::from([
			[13.0, 2.0, 12.0],
			[14.0, 7.0, 18.0],
			[0.0, 11.0, 7.0]
		]);

		assert_eq!(a + b, expected);
	}

	#[test]
	fn sub() {
		let a = Matrix3::from([
			[4.0, 2.0, 8.0],
			[7.0, 1.0, 9.0],
			[0.0, 2.0, 6.0]
		]);

		let b = Matrix3::from([
			[9.0, 0.0, 4.0],
			[7.0, 6.0, 9.0],
			[0.0, 9.0, 1.0]
		]);

		let expected = Matrix3::from([
			[-5.0, 2.0, 4.0],
			[0.0, -5.0, 0.0],
			[0.0, -7.0, 5.0]
		]);

		assert_eq!(a - b, expected);
	}

	#[test]
	fn mul() {
		let a = Matrix3::from([
			[1.0, 2.0, 3.0],
			[4.0, 5.0, 6.0],
			[7.0, 8.0, 9.0]
		]);

		let b = Matrix3::from([
			[1.0, 2.0, 3.0],
			[4.0, 5.0, 6.0],
			[7.0, 8.0, 9.0]
		]);

		let expected = Matrix3::from([
			[30.0, 36.0, 42.0],
			[66.0, 81.0, 96.0],
			[102.0, 126.0, 150.0]
		]);

		assert_eq!(a * b, expected);
	}

	#[test]
	fn add_assign() {
		let mut a = Matrix3::from([
			[4.0, 2.0, 8.0],
			[7.0, 1.0, 9.0],
			[0.0, 2.0, 6.0]
		]);

		let b = Matrix3::from([
			[9.0, 0.0, 4.0],
			[7.0, 6.0, 9.0],
			[0.0, 9.0, 1.0]
		]);

		let expected = Matrix3::from([
			[13.0, 2.0, 12.0],
			[14.0, 7.0, 18.0],
			[0.0, 11.0, 7.0]
		]);

		a += b;
		assert_eq!(a, expected);
	}

	#[test]
	fn sub_assign() {
		let mut a = Matrix3::from([
			[4.0, 2.0, 8.0],
			[7.0, 1.0, 9.0],
			[0.0, 2.0, 6.0]
		]);

		let b = Matrix3::from([
			[9.0, 0.0, 4.0],
			[7.0, 6.0, 9.0],
			[0.0, 9.0, 1.0]
		]);

		let expected = Matrix3::from([
			[-5.0, 2.0, 4.0],
			[0.0, -5.0, 0.0],
			[0.0, -7.0, 5.0]
		]);

		a -= b;
		assert_eq!(a, expected);
	}

	#[test]
	fn mul_assign() {
		let mut a = Matrix3::from([
			[1.0, 2.0, 3.0],
			[4.0, 5.0, 6.0],
			[7.0, 8.0, 9.0]
		]);

		let b = Matrix3::from([
			[1.0, 2.0, 3.0],
			[4.0, 5.0, 6.0],
			[7.0, 8.0, 9.0]
		]);

		let expected = Matrix3::from([
			[30.0, 36.0, 42.0],
			[66.0, 81.0, 96.0],
			[102.0, 126.0, 150.0]
		]);

		a *= b;
		assert_eq!(a, expected);
	}

	#[test]
	fn approx_eq() {
		let elements = [
			[0.0, 0.1, 0.2],
			[1.0, 1.1, 1.2],
			[2.0, 2.1, 2.2]
		];

		let a = Matrix3::from(elements);
		let b = Matrix3::from(elements);

		assert_approx_eq(&a, &b, 0.0);
	}
}