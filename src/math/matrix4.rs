use crate::math::{Vector3, Quaternion};
use std::ops::Mul;

const IDENTITY: [[f32; 4]; 4] = [
	[1.0, 0.0, 0.0, 0.0],
	[0.0, 1.0, 0.0, 0.0],
	[0.0, 0.0, 1.0, 0.0],
	[0.0, 0.0, 0.0, 1.0]
];

#[derive(Copy, Clone, Debug)]
pub struct Matrix4 {
	pub elements: [[f32; 4]; 4]
}

impl Matrix4 {
	pub fn new() -> Self {
		Matrix4 { elements: IDENTITY }
	}

	pub fn from_elements(elements: [[f32; 4]; 4]) -> Self {
		Matrix4 { elements }
	}

	pub fn set_from_elements(&mut self, elements: [[f32; 4]; 4]) {
		self.elements = elements;
	}

	pub fn identity(&mut self) {
		self.elements = IDENTITY;
	}

	pub fn transpose(&mut self) {
		let e = &mut self.elements;
		let mut temp;
		
		temp = e[1][0]; e[1][0] = e[0][1]; e[0][1] = temp;
		temp = e[2][0]; e[2][0] = e[0][2]; e[0][2] = temp;
		temp = e[3][0]; e[3][0] = e[0][3]; e[0][3] = temp;
		
		temp = e[2][1]; e[2][1] = e[1][2]; e[1][2] = temp;
		temp = e[3][1]; e[3][1] = e[1][3]; e[1][3] = temp;

		temp = e[3][2]; e[3][2] = e[2][3]; e[2][3] = temp;
	}

	pub fn invert(&mut self) {
		let m = &mut self.elements;

		let (m00, m01, m02, m03) = (m[0][0], m[0][1], m[0][2], m[0][3]);
		let (m10, m11, m12, m13) = (m[1][0], m[1][1], m[1][2], m[1][3]);
		let (m20, m21, m22, m23) = (m[2][0], m[2][1], m[2][2], m[2][3]);
		let (m30, m31, m32, m33) = (m[3][0], m[3][1], m[3][2], m[3][3]);

		let t11 = m21 * m32 * m13 - m31 * m22 * m13 + m31 * m12 * m23 - m11 * m32 * m23 - m21 * m12 * m33 + m11 * m22 * m33;
		let t12 = m30 * m22 * m13 - m20 * m32 * m13 - m30 * m12 * m23 + m10 * m32 * m23 + m20 * m12 * m33 - m10 * m22 * m33;
		let t13 = m20 * m31 * m13 - m30 * m21 * m13 + m30 * m11 * m23 - m10 * m31 * m23 - m20 * m11 * m33 + m10 * m21 * m33;
		let t14 = m30 * m21 * m12 - m20 * m31 * m12 - m30 * m11 * m22 + m10 * m31 * m22 + m20 * m11 * m32 - m10 * m21 * m32;

		let det = m00 * t11 + m01 * t12 + m02 * t13 + m03 * t14;

		if det == 0.0 {
			self.identity();
			return;
		}

		let det_rec = 1.0 / det;

		m[0][0] = t11 * det_rec;
		m[0][1] = (m31 * m22 * m03 - m21 * m32 * m03 - m31 * m02 * m23 + m01 * m32 * m23 + m21 * m02 * m33 - m01 * m22 * m33) * det_rec;
		m[0][2] = (m11 * m32 * m03 - m31 * m12 * m03 + m31 * m02 * m13 - m01 * m32 * m13 - m11 * m02 * m33 + m01 * m12 * m33) * det_rec;
		m[0][3] = (m21 * m12 * m03 - m11 * m22 * m03 - m21 * m02 * m13 + m01 * m22 * m13 + m11 * m02 * m23 - m01 * m12 * m23) * det_rec;

		m[1][0] = t12 * det_rec;
		m[1][1] = (m20 * m32 * m03 - m30 * m22 * m03 + m30 * m02 * m23 - m00 * m32 * m23 - m20 * m02 * m33 + m00 * m22 * m33) * det_rec;
		m[1][2] = (m30 * m12 * m03 - m10 * m32 * m03 - m30 * m02 * m13 + m00 * m32 * m13 + m10 * m02 * m33 - m00 * m12 * m33) * det_rec;
		m[1][3] = (m10 * m22 * m03 - m20 * m12 * m03 + m20 * m02 * m13 - m00 * m22 * m13 - m10 * m02 * m23 + m00 * m12 * m23) * det_rec;

		m[2][0] = t13 * det_rec;
		m[2][1] = (m30 * m21 * m03 - m20 * m31 * m03 - m30 * m01 * m23 + m00 * m31 * m23 + m20 * m01 * m33 - m00 * m21 * m33) * det_rec;
		m[2][2] = (m10 * m31 * m03 - m30 * m11 * m03 + m30 * m01 * m13 - m00 * m31 * m13 - m10 * m01 * m33 + m00 * m11 * m33) * det_rec;
		m[2][3] = (m20 * m11 * m03 - m10 * m21 * m03 - m20 * m01 * m13 + m00 * m21 * m13 + m10 * m01 * m23 - m00 * m11 * m23) * det_rec;

		m[3][0] = t14 * det_rec;
		m[3][1] = (m20 * m31 * m02 - m30 * m21 * m02 + m30 * m01 * m22 - m00 * m31 * m22 - m20 * m01 * m32 + m00 * m21 * m32) * det_rec;
		m[3][2] = (m30 * m11 * m02 - m10 * m31 * m02 - m30 * m01 * m12 + m00 * m31 * m12 + m10 * m01 * m32 - m00 * m11 * m32) * det_rec;
		m[3][3] = (m10 * m21 * m02 - m20 * m11 * m02 + m20 * m01 * m12 - m00 * m21 * m12 - m10 * m01 * m22 + m00 * m11 * m22) * det_rec;
	}

	pub fn compose(&mut self, position: &Vector3, rotation: &Quaternion, scale: &Vector3) {
		let (px, py, pz) = (position.x, position.y, position.z);
		let (qx, qy, qz, qw) = (rotation.x, rotation.y, rotation.z, rotation.w);
		let (sx, sy, sz) = (scale.x, scale.y, scale.z);

		let (qx2, qy2, qz2) = (qx * 2.0, qy * 2.0, qz * 2.0);
		let (qxx, qxy, qxz) = (qx * qx2, qx * qy2, qx * qz2);
		let (qyy, qyz, qzz) = (qy * qy2, qy * qz2, qz * qz2);
		let (qwx, qwy, qwz) = (qw * qx2, qw * qy2, qw * qz2);

		self.elements = [
			[(1.0 - (qyy + qzz)) * sx, (qxy - qwz) * sy, (qxz + qwy) * sz, px],
			[(qxy + qwz) * sx, (1.0 - (qxx + qzz)) * sy, (qyz - qwx) * sz, py],
			[(qxz - qwy) * sx, (qyz + qwx) * sy, (1.0 - (qxx + qyy)) * sz, pz],
			[0.0, 0.0, 0.0, 1.0]
		];
	}

	pub fn make_perspective(&mut self, aspect: f32, fov: f32, near: f32, far: f32) {
		let s = (fov / 2.0 * std::f32::consts::PI / 180.0).tan();
		let d = far - near;

		self.elements = [
			[1.0 / (s * aspect), 0.0, 0.0, 0.0],
			[0.0, 1.0 / s, 0.0, 0.0],
			[0.0, 0.0, far / d, -(far * near) / d],
			[0.0, 0.0, 1.0, 0.0]
		];
	}

	pub fn approx_eq(&self, other: &Self, tol: f32) -> bool {
		for i in 0..4 {
			for j in 0..4 {
				if (self.elements[i][j] - other.elements[i][j]).abs() > tol {
					return false;
				}
			}
		}

		true
	}
}

impl Mul for Matrix4 {
	type Output = Matrix4;

	fn mul(self, rhs: Self) -> Self {
		let a = &self.elements;
		let b = &rhs.elements;
		
		let c00 = a[0][0] * b[0][0] + a[0][1] * b[1][0] + a[0][2] * b[2][0] + a[0][3] * b[3][0];
		let c01 = a[0][0] * b[0][1] + a[0][1] * b[1][1] + a[0][2] * b[2][1] + a[0][3] * b[3][1];
		let c02 = a[0][0] * b[0][2] + a[0][1] * b[1][2] + a[0][2] * b[2][2] + a[0][3] * b[3][2];
		let c03 = a[0][0] * b[0][3] + a[0][1] * b[1][3] + a[0][2] * b[2][3] + a[0][3] * b[3][3];

		let c10 = a[1][0] * b[0][0] + a[1][1] * b[1][0] + a[1][2] * b[2][0] + a[1][3] * b[3][0];
		let c11 = a[1][0] * b[0][1] + a[1][1] * b[1][1] + a[1][2] * b[2][1] + a[1][3] * b[3][1];
		let c12 = a[1][0] * b[0][2] + a[1][1] * b[1][2] + a[1][2] * b[2][2] + a[1][3] * b[3][2];
		let c13 = a[1][0] * b[0][3] + a[1][1] * b[1][3] + a[1][2] * b[2][3] + a[1][3] * b[3][3];

		let c20 = a[2][0] * b[0][0] + a[2][1] * b[1][0] + a[2][2] * b[2][0] + a[2][3] * b[3][0];
		let c21 = a[2][0] * b[0][1] + a[2][1] * b[1][1] + a[2][2] * b[2][1] + a[2][3] * b[3][1];
		let c22 = a[2][0] * b[0][2] + a[2][1] * b[1][2] + a[2][2] * b[2][2] + a[2][3] * b[3][2];
		let c23 = a[2][0] * b[0][3] + a[2][1] * b[1][3] + a[2][2] * b[2][3] + a[2][3] * b[3][3];

		let c30 = a[3][0] * b[0][0] + a[3][1] * b[1][0] + a[3][2] * b[2][0] + a[3][3] * b[3][0];
		let c31 = a[3][0] * b[0][1] + a[3][1] * b[1][1] + a[3][2] * b[2][1] + a[3][3] * b[3][1];
		let c32 = a[3][0] * b[0][2] + a[3][1] * b[1][2] + a[3][2] * b[2][2] + a[3][3] * b[3][2];
		let c33 = a[3][0] * b[0][3] + a[3][1] * b[1][3] + a[3][2] * b[2][3] + a[3][3] * b[3][3];

		Self {
			elements: [
				[c00, c01, c02, c03],
				[c10, c11, c12, c13],
				[c20, c21, c22, c23],
				[c30, c31, c32, c33]
			]
		}
	}
}

impl PartialEq for Matrix4 {
	fn eq(&self, other: &Self) -> bool {
		self.elements == other.elements
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn new() {
		assert_eq!(Matrix4::new().elements, IDENTITY);
	}

	#[test]
	fn from_elements() {
		let elements = [
			[0.0, 0.1, 0.2, 0.3],
			[1.0, 1.1, 1.2, 1.3],
			[2.0, 2.1, 2.2, 2.3],
			[3.0, 3.1, 3.2, 3.3]
		];

		let m = Matrix4::from_elements(elements);

		assert_eq!(m.elements, elements);
	}

	#[test]
	fn set_from_elements() {
		let elements = [
			[0.0, 0.1, 0.2, 0.3],
			[1.0, 1.1, 1.2, 1.3],
			[2.0, 2.1, 2.2, 2.3],
			[3.0, 3.1, 3.2, 3.3]
		];

		let mut m = Matrix4::new();
		m.set_from_elements(elements);

		assert_eq!(m.elements, elements);
	}

	#[test]
	fn identity() {
		let mut m = Matrix4::from_elements([
			[0.0, 0.1, 0.2, 0.3],
			[1.0, 1.1, 1.2, 1.3],
			[2.0, 2.1, 2.2, 2.3],
			[3.0, 3.1, 3.2, 3.3]
		]);
		m.identity();

		assert_eq!(m.elements, IDENTITY);
	}

	#[test]
	fn transpose() {
		let mut m = Matrix4::from_elements([
			[0.0, 0.1, 0.2, 0.3],
			[1.0, 1.1, 1.2, 1.3],
			[2.0, 2.1, 2.2, 2.3],
			[3.0, 3.1, 3.2, 3.3]
		]);
		m.transpose();

		let expected = [
			[0.0, 1.0, 2.0, 3.0],
			[0.1, 1.1, 2.1, 3.1],
			[0.2, 1.2, 2.2, 3.2],
			[0.3, 1.3, 2.3, 3.3]
		];

		assert_eq!(m.elements, expected);
	}

	#[test]
	fn invert() {
		let mut m = Matrix4::from_elements([
			[2.0, 4.0, 3.0, 7.0],
			[5.0, 2.0, 8.0, 3.0],
			[7.0, 6.0, 1.0, 0.0],
			[4.0, 9.0, 5.0, 7.0]
		]);
		m.invert();

		let expected = Matrix4::from_elements([
			[0.205, 0.038, 0.183, -0.222],
			[-0.209, -0.066, -0.028, 0.238],
			[-0.181, 0.127, -0.111, 0.126],
			[0.281, -0.027, 0.011, -0.126]
		]);

		assert!(m.approx_eq(&expected, 0.001));
	}

	#[test]
	fn compose() {
		let pos = Vector3::from_xyz(1.0, 2.0, 3.0);
		let rot = Quaternion::from_xyzw(1.0, 2.0, 3.0, 4.0);
		let scale = Vector3::from_xyz(3.0, 4.0, 5.0);
		let mut m = Matrix4::new();
		m.compose(&pos, &rot, &scale);

		let expected = [
			[-75.0, -80.0, 110.0, 1.0],
			[84.0, -76.0, 20.0, 2.0],
			[-30.0, 80.0, -45.0, 3.0],
			[0.0, 0.0, 0.0, 1.0]
		];

		assert_eq!(m.elements, expected);
	}

	#[test]
	fn make_perspective() {
		let mut m = Matrix4::new();
		m.make_perspective(0.5, 90.0, 1.0, 5.0);

		let expected = [
			[2.0, 0.0, 0.0, 0.0],
			[0.0, 1.0, 0.0, 0.0],
			[0.0, 0.0, 1.25, -1.25],
			[0.0, 0.0, 1.0, 0.0]
		];

		assert_eq!(m.elements, expected);
	}

	#[test]
	pub fn approx_eq() {
		let elements = [
			[0.0, 0.1, 0.2, 0.3],
			[1.0, 1.1, 1.2, 1.3],
			[2.0, 2.1, 2.2, 2.3],
			[3.0, 3.1, 3.2, 3.3]
		];

		let a = Matrix4::from_elements(elements);
		let b = Matrix4::from_elements(elements);
		assert!(a.approx_eq(&b, 0.0));

		let a = Matrix4::from_elements([
			[4.0, 2.0, 8.0, 5.0],
			[7.0, 1.0, 9.0, 4.0],
			[0.0, 2.0, 6.0, 3.0],
			[7.0, 8.0, 5.0, 3.0]
		]);

		let b = Matrix4::from_elements([
			[9.1, 0.0, 4.0, 5.0],
			[7.0, 6.0, 9.0, 2.0],
			[0.0, 9.0, 1.0, 7.0],
			[3.0, 4.0, 5.0, 2.0]
		]);

		assert!(!a.approx_eq(&b, 0.001));
	}

	#[test]
	fn mul() {
		let a = Matrix4::from_elements([
			[4.0, 2.0, 8.0, 5.0],
			[7.0, 1.0, 9.0, 4.0],
			[0.0, 2.0, 6.0, 3.0],
			[7.0, 8.0, 5.0, 3.0]
		]);

		let b = Matrix4::from_elements([
			[9.0, 0.0, 4.0, 5.0],
			[7.0, 6.0, 9.0, 2.0],
			[0.0, 9.0, 1.0, 7.0],
			[3.0, 4.0, 5.0, 2.0]
		]);

		let expected = [
			[65.0, 104.0, 67.0, 90.0],
			[82.0, 103.0, 66.0, 108.0],
			[23.0, 78.0, 39.0, 52.0],
			[128.0, 105.0, 120.0, 92.0]
		];

		assert_eq!((a * b).elements, expected);
	}

	#[test]
	pub fn eq() {
		let elements = [
			[0.0, 0.1, 0.2, 0.3],
			[1.0, 1.1, 1.2, 1.3],
			[2.0, 2.1, 2.2, 2.3],
			[3.0, 3.1, 3.2, 3.3]
		];

		let a = Matrix4::from_elements(elements);
		let b = Matrix4::from_elements(elements);

		assert_eq!(a, b);
	}
}