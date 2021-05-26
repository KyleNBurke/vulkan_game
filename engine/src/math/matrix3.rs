use super::{Vector2, ApproxEq};
use auto_ops::impl_op_ex;

pub const IDENTITY: Matrix3 = Matrix3 {
	elements: [
		[1.0, 0.0, 0.0],
		[0.0, 1.0, 0.0],
		[0.0, 0.0, 1.0]
	]
};

#[derive(Default, Copy, Clone, Debug, PartialEq)]
pub struct Matrix3 {
	pub elements: [[f32; 3]; 3]
}

impl Matrix3 {
	pub fn new(elements: [[f32; 3]; 3]) -> Self {
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
			[e[2][0], e[2][1], e[2][2], 0.0]
		]
	}

	pub fn compose(&mut self, position: &Vector2, orientation: f32, scale: &Vector2) {
		let se = &mut self.elements;

		se[0][0] = orientation.cos() * scale.x;
		se[0][1] = -orientation.sin();
		se[0][2] = position.x;

		se[1][0] = orientation.sin();
		se[1][1] = orientation.cos() * scale.y;
		se[1][2] = position.y;

		se[2][0] = 0.0;
		se[2][1] = 0.0;
		se[2][2] = 1.0;
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
	#[allow(clippy::clippy::suspicious_op_assign_impl)]
	{
		let ae = &mut a.elements;
		let be = &b.elements;

		let (a00, a01, a02) = (ae[0][0], ae[0][1], ae[0][2]);
		let (a10, a11, a12) = (ae[1][0], ae[1][1], ae[1][2]);
		let (a20, a21, a22) = (ae[2][0], ae[2][1], ae[2][2]);

		let (b00, b01, b02) = (be[0][0], be[0][1], be[0][2]);
		let (b10, b11, b12) = (be[1][0], be[1][1], be[1][2]);
		let (b20, b21, b22) = (be[2][0], be[2][1], be[2][2]);

		ae[0][0] = a00 * b00 + a01 * b10 + a02 * b20;
		ae[0][1] = a00 * b01 + a01 * b11 + a02 * b21;
		ae[0][2] = a00 * b02 + a01 * b12 + a02 * b22;

		ae[1][0] = a10 * b00 + a11 * b10 + a12 * b20;
		ae[1][1] = a10 * b01 + a11 * b11 + a12 * b21;
		ae[1][2] = a10 * b02 + a11 * b12 + a12 * b22;

		ae[2][0] = a20 * b00 + a21 * b10 + a22 * b20;
		ae[2][1] = a20 * b01 + a21 * b11 + a22 * b21;
		ae[2][2] = a20 * b02 + a21 * b12 + a22 * b22;
	}
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
		let elements = [
			[1.0, 2.0, 3.0],
			[4.0, 5.0, 6.0],
			[7.0, 8.0, 9.0]];

		let m = Matrix3::new(elements);
		assert_eq!(m.elements, elements);
	}

	#[test]
	fn set() {
		let elements = [
			[1.0, 2.0, 3.0],
			[4.0, 5.0, 6.0],
			[7.0, 8.0, 9.0]];

		let mut m = IDENTITY;
		m.set(elements);
		assert_eq!(m.elements, elements);
	}

	#[test]
	fn to_padded_array() {
		let m = Matrix3::new([
			[1.0, 2.0, 3.0],
			[4.0, 5.0, 6.0],
			[7.0, 8.0, 9.0]]);

		let expected = [
			[1.0, 2.0, 3.0, 0.0],
			[4.0, 5.0, 6.0, 0.0],
			[7.0, 8.0, 9.0, 0.0]
		];

		assert_eq!(m.to_padded_array(), expected);
	}

	#[test]
	fn compose() {
		let pos = Vector2::new(100.0, 200.0);
		let rot = std::f32::consts::PI;
		let scale = Vector2::new(3.0, 4.0);
		let mut m = IDENTITY;
		m.compose(&pos, rot, &scale);

		let expected = Matrix3::new([
			[-3.0, 0.0, 100.0],
			[0.0, -4.0, 200.0],
			[0.0, 0.0, 1.0]]);

		assert_approx_eq(&m, &expected, 1e-6);
	}

	#[test]
	fn add() {
		let a = Matrix3::new([
			[4.0, 2.0, 8.0],
			[7.0, 1.0, 9.0],
			[0.0, 2.0, 6.0]]);

		let b = Matrix3::new([
			[9.0, 0.0, 4.0],
			[7.0, 6.0, 9.0],
			[0.0, 9.0, 1.0]]);

		let expected = Matrix3::new([
			[13.0, 2.0, 12.0],
			[14.0, 7.0, 18.0],
			[0.0, 11.0, 7.0]]);

		assert_eq!(a + b, expected);
	}

	#[test]
	fn sub() {
		let a = Matrix3::new([
			[4.0, 2.0, 8.0],
			[7.0, 1.0, 9.0],
			[0.0, 2.0, 6.0]]);

		let b = Matrix3::new([
			[9.0, 0.0, 4.0],
			[7.0, 6.0, 9.0],
			[0.0, 9.0, 1.0]]);

		let expected = Matrix3::new([
			[-5.0, 2.0, 4.0],
			[0.0, -5.0, 0.0],
			[0.0, -7.0, 5.0]]);

		assert_eq!(a - b, expected);
	}

	#[test]
	fn mul() {
		let a = Matrix3::new([
			[1.0, 2.0, 3.0],
			[4.0, 5.0, 6.0],
			[7.0, 8.0, 9.0]]);

		let b = Matrix3::new([
			[1.0, 2.0, 3.0],
			[4.0, 5.0, 6.0],
			[7.0, 8.0, 9.0]]);

		let expected = Matrix3::new([
			[30.0, 36.0, 42.0],
			[66.0, 81.0, 96.0],
			[102.0, 126.0, 150.0]]);

		assert_eq!(a * b, expected);
	}

	#[test]
	fn add_assign() {
		let mut a = Matrix3::new([
			[4.0, 2.0, 8.0],
			[7.0, 1.0, 9.0],
			[0.0, 2.0, 6.0]]);

		let b = Matrix3::new([
			[9.0, 0.0, 4.0],
			[7.0, 6.0, 9.0],
			[0.0, 9.0, 1.0]]);

		let expected = Matrix3::new([
			[13.0, 2.0, 12.0],
			[14.0, 7.0, 18.0],
			[0.0, 11.0, 7.0]]);

		a += b;
		assert_eq!(a, expected);
	}

	#[test]
	fn sub_assign() {
		let mut a = Matrix3::new([
			[4.0, 2.0, 8.0],
			[7.0, 1.0, 9.0],
			[0.0, 2.0, 6.0]]);

		let b = Matrix3::new([
			[9.0, 0.0, 4.0],
			[7.0, 6.0, 9.0],
			[0.0, 9.0, 1.0]]);

		let expected = Matrix3::new([
			[-5.0, 2.0, 4.0],
			[0.0, -5.0, 0.0],
			[0.0, -7.0, 5.0]]);

		a -= b;
		assert_eq!(a, expected);
	}

	#[test]
	fn mul_assign() {
		let mut a = Matrix3::new([
			[1.0, 2.0, 3.0],
			[4.0, 5.0, 6.0],
			[7.0, 8.0, 9.0]]);

		let b = Matrix3::new([
			[1.0, 2.0, 3.0],
			[4.0, 5.0, 6.0],
			[7.0, 8.0, 9.0]]);

		let expected = Matrix3::new([
			[30.0, 36.0, 42.0],
			[66.0, 81.0, 96.0],
			[102.0, 126.0, 150.0]]);

		a *= b;
		assert_eq!(a, expected);
	}

	#[test]
	fn approx_eq() {
		let a = Matrix3::new([
			[1.0, 2.0, 3.0],
			[4.0, 5.0, 6.0],
			[7.0, 8.0, 9.0]]);

		let b = Matrix3::new([
			[0.0, 3.0, 4.0],
			[5.0, 4.0, 7.0],
			[6.0, 7.0, 10.0]]);

		assert_approx_eq(&a, &b, 1.0);
	}
}