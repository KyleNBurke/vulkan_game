use std::fmt::Display;
use super::{ApproxEq, Euler, Matrix3, Order, Quaternion, Vector3, Vector4, vector3};
use auto_ops::impl_op_ex;

pub const IDENTITY: Matrix4 = Matrix4 {
	elements: [
		[1.0, 0.0, 0.0, 0.0],
		[0.0, 1.0, 0.0, 0.0],
		[0.0, 0.0, 1.0, 0.0],
		[0.0, 0.0, 0.0, 1.0]
	]
};

#[derive(Default, Copy, Clone, Debug, PartialEq)]
pub struct Matrix4 {
	pub elements: [[f32; 4]; 4]
}

impl Matrix4 {
	pub fn new(elements: [[f32; 4]; 4]) -> Self {
		Self { elements }
	}

	pub fn set(&mut self, elements: [[f32; 4]; 4]) {
		self.elements = elements;
	}

	pub fn identity(&mut self) {
		let se = &mut self.elements;

		se[0][0] = 1.0; se[0][1] = 0.0; se[0][2] = 0.0; se[0][3] = 0.0;
		se[1][0] = 0.0; se[1][1] = 1.0; se[1][2] = 0.0; se[1][3] = 0.0;
		se[2][0] = 0.0; se[2][1] = 0.0; se[2][2] = 1.0; se[2][3] = 0.0;
		se[3][0] = 0.0; se[3][1] = 0.0; se[3][2] = 0.0; se[3][3] = 1.0;
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

	pub fn compose(&mut self, position: &Vector3, orientation: &Quaternion, scale: &Vector3) {
		let (px, py, pz) = (position.x, position.y, position.z);
		let (qx, qy, qz, qw) = (orientation.x, orientation.y, orientation.z, orientation.w);
		let (sx, sy, sz) = (scale.x, scale.y, scale.z);

		let (qx2, qy2, qz2) = (qx * 2.0, qy * 2.0, qz * 2.0);
		let (qxx, qxy, qxz) = (qx * qx2, qx * qy2, qx * qz2);
		let (qyy, qyz, qzz) = (qy * qy2, qy * qz2, qz * qz2);
		let (qwx, qwy, qwz) = (qw * qx2, qw * qy2, qw * qz2);

		let se = &mut self.elements;

		se[0][0] = (1.0 - (qyy + qzz)) * sx;
		se[0][1] = (qxy - qwz) * sy;
		se[0][2] = (qxz + qwy) * sz;
		se[0][3] = px;

		se[1][0] = (qxy + qwz) * sx;
		se[1][1] = (1.0 - (qxx + qzz)) * sy;
		se[1][2] = (qyz - qwx) * sz;
		se[1][3] = py;

		se[2][0] = (qxz - qwy) * sx;
		se[2][1] = (qyz + qwx) * sy;
		se[2][2] = (1.0 - (qxx + qyy)) * sz;
		se[2][3] = pz;

		se[3][0] = 0.0;
		se[3][1] = 0.0;
		se[3][2] = 0.0;
		se[3][3] = 1.0;
	}

	pub fn extract_position(&self) -> Vector3 {
		let se = &self.elements;
		Vector3::new(se[0][3], se[1][3], se[2][3])
	}

	pub fn make_perspective(&mut self, aspect: f32, fov: f32, near: f32, far: f32) {
		let f = (fov / 2.0 * std::f32::consts::PI / 180.0).tan();
		let d = far - near;
		let se = &mut self.elements;

		se[0][0] = -1.0 / (f * aspect);
		se[0][1] = 0.0;
		se[0][2] = 0.0;
		se[0][3] = 0.0;

		se[1][0] = 0.0;
		se[1][1] = -1.0 / f;
		se[1][2] = 0.0;
		se[1][3] = 0.0;

		se[2][0] = 0.0;
		se[2][1] = 0.0;
		se[2][2] = far / d;
		se[2][3] = -(far * near) / d;

		se[3][0] = 0.0;
		se[3][1] = 0.0;
		se[3][2] = 1.0;
		se[3][3] = 0.0;
	}

	pub fn make_orientation_from_quaternion(&mut self, q: &Quaternion) {
		self.compose(&vector3::ZERO, q, &vector3::ONE);
	}

	pub fn make_orientation_from_euler(&mut self, e: &Euler) {
		let (cx, cy, cz) = (e.x.cos(), e.y.cos(), e.z.cos());
		let (sx, sy, sz) = (e.x.sin(), e.y.sin(), e.z.sin());
		let se = &mut self.elements;

		match e.order {
			Order::Xyz => {
				se[0][0] = cy * cz;
				se[0][1] = -cy * sz;
				se[0][2] = sy;

				se[1][0] = cz * sx * sy + cx * sz;
				se[1][1] = cx * cz - sx * sy * sz;
				se[1][2] = -cy * sx;

				se[2][0] = sx * sz - cx * cz * sy;
				se[2][1] = cz * sx + cx * sy * sz;
				se[2][2] = cx * cy;
			},
			Order::Xzy => {
				se[0][0] = cy * cz;
				se[0][1] = -sz;
				se[0][2] = cz * sy;

				se[1][0] = sx * sy + cx * cy * sz;
				se[1][1] = cx * cz;
				se[1][2] = cx * sy * sz - cy * sx;

				se[2][0] = cy * sx * sz - cx * sy;
				se[2][1] = cz * sx;
				se[2][2] = cx * cy + sx * sy * sz;
			},
			Order::Yxz => {
				se[0][0] = cy * cz + sx * sy * sz;
				se[0][1] = cz * sx * sy - cy * sz;
				se[0][2] = cx * sy;

				se[1][0] = cx * sz;
				se[1][1] = cx * cz;
				se[1][2] = -sx;

				se[2][0] = cy * sx * sz - cz * sy;
				se[2][1] = cy * cz * sx + sy * sz;
				se[2][2] = cx * cy;
			},
			Order::Yzx => {
				se[0][0] = cy * cz;
				se[0][1] = sx * sy - cx * cy * sz;
				se[0][2] = cx * sy + cy * sx * sz;

				se[1][0] = sz;
				se[1][1] = cx * cz;
				se[1][2] = -cz * sx;

				se[2][0] = -cz * sy;
				se[2][1] = cy * sx + cx * sy * sz;
				se[2][2] = cx * cy - sx * sy * sz;
			},
			Order::Zxy => {
				se[0][0] = cy * cz - sx * sy * sz;
				se[0][1] = -cx * sz;
				se[0][2] = cz * sy + cy * sx * sz;

				se[1][0] = cz * sx * sz + cy * sz;
				se[1][1] = cx * cz;
				se[1][2] = sy * sz - cy * cz * sx;

				se[2][0] = -cx * sy;
				se[2][1] = sx;
				se[2][2] = cx * cy;
			},
			Order::Zyx => {
				se[0][0] = cy * cz;
				se[0][1] = cz * sx * sy - cx * sz;
				se[0][2] = cx * cz * sy + sx * sz;

				se[1][0] = cy * sz;
				se[1][1] = cx * cz + sx * sy * sz;
				se[1][2] = cx * sy * sz - cz * sx;

				se[2][0] = -sy;
				se[2][1] = cy * sx;
				se[2][2] = cx * cy;
			}
		}

		se[0][3] = 0.0;
		se[1][3] = 0.0;
		se[2][3] = 0.0;

		se[3][0] = 0.0;
		se[3][1] = 0.0;
		se[3][2] = 0.0;
		se[3][3] = 1.0;
	}

	pub fn truncate(&self) -> Matrix3 {
		let e = &self.elements;

		Matrix3 {
			elements: [
				[e[0][0], e[0][1], e[0][2]],
				[e[1][0], e[1][1], e[1][2]],
				[e[2][0], e[2][1], e[2][2]]
			]
		}
	}
}

impl_op_ex!(+ |a: &Matrix4, b: &Matrix4| -> Matrix4 {
	let mut r = *a;
	r += b;
	r
});

impl_op_ex!(- |a: &Matrix4, b: &Matrix4| -> Matrix4 {
	let mut r = *a;
	r -= b;
	r
});

impl_op_ex!(* |a: &Matrix4, b: &Matrix4| -> Matrix4 {
	let mut r = *a;
	r *= b;
	r
});

impl_op_ex!(+= |a: &mut Matrix4, b: &Matrix4| {
	let ae = &mut a.elements;
	let be = &b.elements;

	ae[0][0] += be[0][0];
	ae[0][1] += be[0][1];
	ae[0][2] += be[0][2];
	ae[0][3] += be[0][3];

	ae[1][0] += be[1][0];
	ae[1][1] += be[1][1];
	ae[1][2] += be[1][2];
	ae[1][3] += be[1][3];

	ae[2][0] += be[2][0];
	ae[2][1] += be[2][1];
	ae[2][2] += be[2][2];
	ae[2][3] += be[2][3];

	ae[3][0] += be[3][0];
	ae[3][1] += be[3][1];
	ae[3][2] += be[3][2];
	ae[3][3] += be[3][3];
});

impl_op_ex!(-= |a: &mut Matrix4, b: &Matrix4| {
	let ae = &mut a.elements;
	let be = &b.elements;

	ae[0][0] -= be[0][0];
	ae[0][1] -= be[0][1];
	ae[0][2] -= be[0][2];
	ae[0][3] -= be[0][3];

	ae[1][0] -= be[1][0];
	ae[1][1] -= be[1][1];
	ae[1][2] -= be[1][2];
	ae[1][3] -= be[1][3];

	ae[2][0] -= be[2][0];
	ae[2][1] -= be[2][1];
	ae[2][2] -= be[2][2];
	ae[2][3] -= be[2][3];

	ae[3][0] -= be[3][0];
	ae[3][1] -= be[3][1];
	ae[3][2] -= be[3][2];
	ae[3][3] -= be[3][3];
});

impl_op_ex!(*= |a: &mut Matrix4, b: &Matrix4| {
	#[allow(clippy::suspicious_op_assign_impl)]
	{
		let ae = &mut a.elements;
		let be = &b.elements;

		let (a00, a01, a02, a03) = (ae[0][0], ae[0][1], ae[0][2], ae[0][3]);
		let (a10, a11, a12, a13) = (ae[1][0], ae[1][1], ae[1][2], ae[1][3]);
		let (a20, a21, a22, a23) = (ae[2][0], ae[2][1], ae[2][2], ae[2][3]);
		let (a30, a31, a32, a33) = (ae[3][0], ae[3][1], ae[3][2], ae[3][3]);

		let (b00, b01, b02, b03) = (be[0][0], be[0][1], be[0][2], be[0][3]);
		let (b10, b11, b12, b13) = (be[1][0], be[1][1], be[1][2], be[1][3]);
		let (b20, b21, b22, b23) = (be[2][0], be[2][1], be[2][2], be[2][3]);
		let (b30, b31, b32, b33) = (be[3][0], be[3][1], be[3][2], be[3][3]);

		ae[0][0] = a00 * b00 + a01 * b10 + a02 * b20 + a03 * b30;
		ae[0][1] = a00 * b01 + a01 * b11 + a02 * b21 + a03 * b31;
		ae[0][2] = a00 * b02 + a01 * b12 + a02 * b22 + a03 * b32;
		ae[0][3] = a00 * b03 + a01 * b13 + a02 * b23 + a03 * b33;

		ae[1][0] = a10 * b00 + a11 * b10 + a12 * b20 + a13 * b30;
		ae[1][1] = a10 * b01 + a11 * b11 + a12 * b21 + a13 * b31;
		ae[1][2] = a10 * b02 + a11 * b12 + a12 * b22 + a13 * b32;
		ae[1][3] = a10 * b03 + a11 * b13 + a12 * b23 + a13 * b33;

		ae[2][0] = a20 * b00 + a21 * b10 + a22 * b20 + a23 * b30;
		ae[2][1] = a20 * b01 + a21 * b11 + a22 * b21 + a23 * b31;
		ae[2][2] = a20 * b02 + a21 * b12 + a22 * b22 + a23 * b32;
		ae[2][3] = a20 * b03 + a21 * b13 + a22 * b23 + a23 * b33;

		ae[3][0] = a30 * b00 + a31 * b10 + a32 * b20 + a33 * b30;
		ae[3][1] = a30 * b01 + a31 * b11 + a32 * b21 + a33 * b31;
		ae[3][2] = a30 * b02 + a31 * b12 + a32 * b22 + a33 * b32;
		ae[3][3] = a30 * b03 + a31 * b13 + a32 * b23 + a33 * b33;
	}
});

impl_op_ex!(* |a: &Matrix4, b: &Vector4| -> Vector4 {
	let ae = &a.elements;

	Vector4 {
		x: ae[0][0] * b.x + ae[0][1] * b.y + ae[0][2] * b.z + ae[0][3] * b.w,
		y: ae[1][0] * b.x + ae[1][1] * b.y + ae[1][2] * b.z + ae[1][3] * b.w,
		z: ae[2][0] * b.x + ae[2][1] * b.y + ae[2][2] * b.z + ae[2][3] * b.w,
		w: ae[3][0] * b.x + ae[3][1] * b.y + ae[3][2] * b.z + ae[3][3] * b.w
	}
});

impl ApproxEq for Matrix4 {
	fn approx_eq(&self, other: &Self, tol: f32) -> bool {
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

impl Display for Matrix4 {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{:?}", self.elements)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::math::assert_approx_eq;
	use std::f32::consts::FRAC_PI_2;

	#[test]
	fn new() {
		let elements = [
			[0.0, 0.1, 0.2, 0.3],
			[1.0, 1.1, 1.2, 1.3],
			[2.0, 2.1, 2.2, 2.3],
			[3.0, 3.1, 3.2, 3.3]];

		let m = Matrix4::new(elements);
		assert_eq!(m.elements, elements);
	}

	#[test]
	fn set_array() {
		let elements = [
			[0.0, 0.1, 0.2, 0.3],
			[1.0, 1.1, 1.2, 1.3],
			[2.0, 2.1, 2.2, 2.3],
			[3.0, 3.1, 3.2, 3.3]];

		let mut m = IDENTITY;
		m.set(elements);
		assert_eq!(m.elements, elements);
	}

	#[test]
	fn identity() {
		let mut m = Matrix4::new([
			[0.0, 0.1, 0.2, 0.3],
			[1.0, 1.1, 1.2, 1.3],
			[2.0, 2.1, 2.2, 2.3],
			[3.0, 3.1, 3.2, 3.3]]);
		m.identity();

		assert_eq!(m, IDENTITY);
	}

	#[test]
	fn transpose() {
		let mut m = Matrix4::new([
			[0.0, 0.1, 0.2, 0.3],
			[1.0, 1.1, 1.2, 1.3],
			[2.0, 2.1, 2.2, 2.3],
			[3.0, 3.1, 3.2, 3.3]]);
		m.transpose();

		let expected = Matrix4::new([
			[0.0, 1.0, 2.0, 3.0],
			[0.1, 1.1, 2.1, 3.1],
			[0.2, 1.2, 2.2, 3.2],
			[0.3, 1.3, 2.3, 3.3]]);

		assert_eq!(m, expected);
	}

	#[test]
	fn invert() {
		let mut m = Matrix4::new([
			[1.0, 0.0, 0.0, 1.0],
			[0.0, 2.0, 1.0, 2.0],
			[2.0, 1.0, 0.0, 1.0],
			[2.0, 0.0, 1.0, 4.0]]);
		m.invert();

		let expected = Matrix4::new([
			[-2.0, -0.5, 1.0, 0.5],
			[1.0, 0.5, 0.0, -0.5],
			[-8.0, -1.0, 2.0, 2.0],
			[3.0, 0.5, -1.0, -0.5]]);

		assert_eq!(m, expected);
	}

	#[test]
	fn compose() {
		let pos = Vector3::new(1.0, 2.0, 3.0);
		let rot = Quaternion::new(1.0, 2.0, 3.0, 4.0);
		let scale = Vector3::new(3.0, 4.0, 5.0);
		let mut m = IDENTITY;
		m.compose(&pos, &rot, &scale);

		let expected = Matrix4::new([
			[-75.0, -80.0, 110.0, 1.0],
			[84.0, -76.0, 20.0, 2.0],
			[-30.0, 80.0, -45.0, 3.0],
			[0.0, 0.0, 0.0, 1.0]]);

		assert_eq!(m, expected);
	}

	#[test]
	fn extract_position() {
		let m = Matrix4::new([
			[0.0, 0.1, 0.2, 0.3],
			[1.0, 1.1, 1.2, 1.3],
			[2.0, 2.1, 2.2, 2.3],
			[3.0, 3.1, 3.2, 3.3]]);

		assert_eq!(Vector3::new(0.3, 1.3, 2.3), m.extract_position());
	}

	#[test]
	fn make_perspective() {
		let mut m = IDENTITY;
		m.make_perspective(0.5, 90.0, 1.0, 5.0);

		let expected = Matrix4::new([
			[-2.0, 0.0, 0.0, 0.0],
			[0.0, -1.0, 0.0, 0.0],
			[0.0, 0.0, 1.25, -1.25],
			[0.0, 0.0, 1.0, 0.0]]);

		assert_eq!(m, expected);
	}

	#[test]
	fn make_orientation_from_quaternion() {
		let mut m = IDENTITY;
		m.make_orientation_from_quaternion(&Quaternion::new(0.5, 0.5, 0.5, 0.5));

		let expected = Matrix4::new([
			[0.0, 0.0, 1.0, 0.0],
			[1.0, 0.0, 0.0, 0.0],
			[0.0, 1.0, 0.0, 0.0],
			[0.0, 0.0, 0.0, 1.0]]);

		assert_eq!(m, expected);
	}

	#[test]
	fn make_orientation_from_euler() {
		let mut m = IDENTITY;

		// XYZ
		m.make_orientation_from_euler(&Euler::new(FRAC_PI_2, FRAC_PI_2, FRAC_PI_2, Order::Xyz));
		let expected = Matrix4::new([
			[0.0, 0.0, 1.0, 0.0],
			[0.0, -1.0, 0.0, 0.0],
			[1.0, 0.0, 0.0, 0.0],
			[0.0, 0.0, 0.0, 1.0]]);
		assert_approx_eq(&m, &expected, 1e-6);

		// XZY
		m.make_orientation_from_euler(&Euler::new(FRAC_PI_2, FRAC_PI_2, FRAC_PI_2, Order::Xzy));
		let expected = Matrix4::new([
			[0.0, -1.0, 0.0, 0.0],
			[1.0, 0.0, 0.0, 0.0],
			[0.0, 0.0, 1.0, 0.0],
			[0.0, 0.0, 0.0, 1.0]]);
		assert_approx_eq(&m, &expected, 1e-6);

		// YXZ
		m.make_orientation_from_euler(&Euler::new(FRAC_PI_2, FRAC_PI_2, FRAC_PI_2, Order::Yxz));
		let expected = Matrix4::new([
			[1.0, 0.0, 0.0, 0.0],
			[0.0, 0.0, -1.0, 0.0],
			[0.0, 1.0, 0.0, 0.0],
			[0.0, 0.0, 0.0, 1.0]]);
		assert_approx_eq(&m, &expected, 1e-6);

		// YZX
		m.make_orientation_from_euler(&Euler::new(FRAC_PI_2, FRAC_PI_2, FRAC_PI_2, Order::Yzx));
		let expected = Matrix4::new([
			[0.0, 1.0, 0.0, 0.0],
			[1.0, 0.0, 0.0, 0.0],
			[0.0, 0.0, -1.0, 0.0],
			[0.0, 0.0, 0.0, 1.0]]);
		assert_approx_eq(&m, &expected, 1e-6);

		// ZXY
		m.make_orientation_from_euler(&Euler::new(FRAC_PI_2, FRAC_PI_2, FRAC_PI_2, Order::Zxy));
		let expected = Matrix4::new([
			[-1.0, 0.0, 0.0, 0.0],
			[0.0, 0.0, 1.0, 0.0],
			[0.0, 1.0, 0.0, 0.0],
			[0.0, 0.0, 0.0, 1.0]]);
		assert_approx_eq(&m, &expected, 1e-6);

		// ZYX
		m.make_orientation_from_euler(&Euler::new(FRAC_PI_2, FRAC_PI_2, FRAC_PI_2, Order::Zyx));
		let expected = Matrix4::new([
			[0.0, 0.0, 1.0, 0.0],
			[0.0, 1.0, 0.0, 0.0],
			[-1.0, 0.0, 0.0, 0.0],
			[0.0, 0.0, 0.0, 1.0]]);
		assert_approx_eq(&m, &expected, 1e-6);
	}

	#[test]
	fn truncate() {
		let a = Matrix4::new([
			[4.0, 2.0, 8.0, 5.0],
			[7.0, 1.0, 9.0, 4.0],
			[0.0, 2.0, 6.0, 3.0],
			[7.0, 8.0, 5.0, 3.0]]);
		
		let expected = Matrix3::new([
			[4.0, 2.0, 8.0],
			[7.0, 1.0, 9.0],
			[0.0, 2.0, 6.0]]);
		
		assert_eq!(a.truncate(), expected);
	}

	#[test]
	fn add() {
		let a = Matrix4::new([
			[4.0, 2.0, 8.0, 5.0],
			[7.0, 1.0, 9.0, 4.0],
			[0.0, 2.0, 6.0, 3.0],
			[7.0, 8.0, 5.0, 3.0]]);

		let b = Matrix4::new([
			[9.0, 0.0, 4.0, 5.0],
			[7.0, 6.0, 9.0, 2.0],
			[0.0, 9.0, 1.0, 7.0],
			[3.0, 4.0, 5.0, 2.0]]);

		let expected = Matrix4::new([
			[13.0, 2.0, 12.0, 10.0],
			[14.0, 7.0, 18.0, 6.0],
			[0.0, 11.0, 7.0, 10.0],
			[10.0, 12.0, 10.0, 5.0]]);

		assert_eq!(a + b, expected);
	}

	#[test]
	fn sub() {
		let a = Matrix4::new([
			[4.0, 2.0, 8.0, 5.0],
			[7.0, 1.0, 9.0, 4.0],
			[0.0, 2.0, 6.0, 3.0],
			[7.0, 8.0, 5.0, 3.0]]);

		let b = Matrix4::new([
			[9.0, 0.0, 4.0, 5.0],
			[7.0, 6.0, 9.0, 2.0],
			[0.0, 9.0, 1.0, 7.0],
			[3.0, 4.0, 5.0, 2.0]]);

		let expected = Matrix4::new([
			[-5.0, 2.0, 4.0, 0.0],
			[0.0, -5.0, 0.0, 2.0],
			[0.0, -7.0, 5.0, -4.0],
			[4.0, 4.0, 0.0, 1.0]]);

		assert_eq!(a - b, expected);
	}

	#[test]
	fn mul() {
		let a = Matrix4::new([
			[4.0, 2.0, 8.0, 5.0],
			[7.0, 1.0, 9.0, 4.0],
			[0.0, 2.0, 6.0, 3.0],
			[7.0, 8.0, 5.0, 3.0]]);

		let b = Matrix4::new([
			[9.0, 0.0, 4.0, 5.0],
			[7.0, 6.0, 9.0, 2.0],
			[0.0, 9.0, 1.0, 7.0],
			[3.0, 4.0, 5.0, 2.0]]);

		let expected = Matrix4::new([
			[65.0, 104.0, 67.0, 90.0],
			[82.0, 103.0, 66.0, 108.0],
			[23.0, 78.0, 39.0, 52.0],
			[128.0, 105.0, 120.0, 92.0]]);

		assert_eq!(a * b, expected);
	}

	#[test]
	fn add_assign() {
		let mut a = Matrix4::new([
			[4.0, 2.0, 8.0, 5.0],
			[7.0, 1.0, 9.0, 4.0],
			[0.0, 2.0, 6.0, 3.0],
			[7.0, 8.0, 5.0, 3.0]]);

		let b = Matrix4::new([
			[9.0, 0.0, 4.0, 5.0],
			[7.0, 6.0, 9.0, 2.0],
			[0.0, 9.0, 1.0, 7.0],
			[3.0, 4.0, 5.0, 2.0]]);

		let expected = Matrix4::new([
			[13.0, 2.0, 12.0, 10.0],
			[14.0, 7.0, 18.0, 6.0],
			[0.0, 11.0, 7.0, 10.0],
			[10.0, 12.0, 10.0, 5.0]]);

		a += b;
		assert_eq!(a, expected);
	}

	#[test]
	fn sub_assign() {
		let mut a = Matrix4::new([
			[4.0, 2.0, 8.0, 5.0],
			[7.0, 1.0, 9.0, 4.0],
			[0.0, 2.0, 6.0, 3.0],
			[7.0, 8.0, 5.0, 3.0]]);

		let b = Matrix4::new([
			[9.0, 0.0, 4.0, 5.0],
			[7.0, 6.0, 9.0, 2.0],
			[0.0, 9.0, 1.0, 7.0],
			[3.0, 4.0, 5.0, 2.0]]);

		let expected = Matrix4::new([
			[-5.0, 2.0, 4.0, 0.0],
			[0.0, -5.0, 0.0, 2.0],
			[0.0, -7.0, 5.0, -4.0],
			[4.0, 4.0, 0.0, 1.0]]);

		a -= b;
		assert_eq!(a, expected);
	}

	#[test]
	fn mul_assign() {
		let mut a = Matrix4::new([
			[4.0, 2.0, 8.0, 5.0],
			[7.0, 1.0, 9.0, 4.0],
			[0.0, 2.0, 6.0, 3.0],
			[7.0, 8.0, 5.0, 3.0]]);

		let b = Matrix4::new([
			[9.0, 0.0, 4.0, 5.0],
			[7.0, 6.0, 9.0, 2.0],
			[0.0, 9.0, 1.0, 7.0],
			[3.0, 4.0, 5.0, 2.0]]);

		let expected = Matrix4::new([
			[65.0, 104.0, 67.0, 90.0],
			[82.0, 103.0, 66.0, 108.0],
			[23.0, 78.0, 39.0, 52.0],
			[128.0, 105.0, 120.0, 92.0]]);

		a *= b;
		assert_eq!(a, expected);
	}

	#[test]
	fn mul_vec_4() {
		let a = Matrix4::new([
			[4.0, 2.0, 8.0, 5.0],
			[7.0, 1.0, 9.0, 4.0],
			[0.0, 2.0, 6.0, 3.0],
			[7.0, 8.0, 5.0, 3.0]]);
		
		let b = Vector4::new(3.0, 2.0, 5.0, 4.0);

		let expected = Vector4::new(76.0, 84.0, 46.0, 74.0);
		assert_eq!(a * b, expected);
	}

	#[test]
	fn approx_eq() {
		let a = Matrix4::new([
			[1.0, 2.0, 3.0, 9.0],
			[4.0, 5.0, 6.0, 2.0],
			[7.0, 8.0, 9.0, 0.0],
			[3.0, 5.0, 8.0, 1.0]]);

		let b = Matrix4::new([
			[0.0, 3.0, 2.0, 8.0],
			[5.0, 6.0, 7.0, 1.0],
			[6.0, 9.0, 8.0, 1.0],
			[2.0, 4.0, 7.0, 2.0]]);

		assert_approx_eq(&a, &b, 1.0);
	}
}