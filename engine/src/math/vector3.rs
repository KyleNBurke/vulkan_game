use super::{Vector4, Quaternion, ApproxEq};
use std::fmt::Display;
use auto_ops::impl_op_ex;

pub const ZERO: Vector3 = Vector3 { x: 0.0, y: 0.0, z: 0.0 };
pub const ONE: Vector3 = Vector3 { x: 1.0, y: 1.0, z: 1.0 };
pub const UNIT_X: Vector3 = Vector3 { x: 1.0, y: 0.0, z: 0.0 };
pub const UNIT_Y: Vector3 = Vector3 { x: 0.0, y: 1.0, z: 0.0 };
pub const UNIT_Z: Vector3 = Vector3 { x: 0.0, y: 0.0, z: 1.0 };

#[derive(Default, Copy, Clone, Debug, PartialEq)]
#[repr(C)]
pub struct Vector3 {
	pub x: f32,
	pub y: f32,
	pub z: f32
}

impl Vector3 {
	pub fn new(x: f32, y: f32, z: f32) -> Self {
		Self { x, y, z }
	}

	pub fn from_scalar(scalar: f32) -> Self {
		Self { x: scalar, y: scalar, z: scalar }
	}

	pub fn set(&mut self, x: f32, y: f32, z: f32) {
		self.x = x;
		self.y = y;
		self.z = z;
	}

	pub fn set_from_scalar(&mut self, scalar: f32) {
		self.x = scalar;
		self.y = scalar;
		self.z = scalar;
	}

	pub fn set_from_index(&mut self, index: u32, value: f32) {
		match index {
			0 => self.x = value,
			1 => self.y = value,
			2 => self.z = value,
			_ => panic!("invalid index {}", index)
		}
	}

	pub fn get_from_index(&self, index: u32) -> f32 {
		match index {
			0 => self.x,
			1 => self.y,
			2 => self.z,
			_ => panic!("invalid index {}", index)
		}
	}

	pub fn expand(&self, w: f32) -> Vector4 {
		Vector4::new(self.x, self.y, self.z, w)
	}

	pub fn length(&self) -> f32 {
		(self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
	}

	pub fn length_sq(&self) -> f32 {
		self.x * self.x + self.y * self.y + self.z * self.z
	}

	pub fn normalize(&mut self) {
		let length = self.length();

		if length != 0.0 {
			*self /= length;
		}
	}

	pub fn dot(&self, other: &Self) -> f32 {
		self.x * other.x + self.y * other.y + self.z * other.z
	}

	pub fn cross(&mut self, other: &Self) {
		let self_x = self.x;
		let self_y = self.y;
		let self_z = self.z;

		self.x = self_y * other.z - self_z * other.y;
		self.y = self_z * other.x - self_x * other.z;
		self.z = self_x * other.y - self_y * other.x;
	}

	pub fn apply_quaternion(&mut self, q: &Quaternion) {
		let ix = q.w * self.x + q.y * self.z - q.z * self.y;
		let iy = q.w * self.y + q.z * self.x - q.x * self.z;
		let iz = q.w * self.z + q.x * self.y - q.y * self.x;
		let iw = -q.x * self.x - q.y * self.y - q.z * self.z;

		self.x = ix * q.w + iw * -q.x + iy * -q.z - iz * -q.y;
		self.y = iy * q.w + iw * -q.y + iz * -q.x - ix * -q.z;
		self.z = iz * q.w + iw * -q.z + ix * -q.y - iy * -q.x;
	}
	
	pub fn min(&mut self, other: &Self) {
		self.x = self.x.min(other.x);
		self.y = self.y.min(other.y);
		self.z = self.z.min(other.z);
	}

	pub fn max(&mut self, other: &Self) {
		self.x = self.x.max(other.x);
		self.y = self.y.max(other.y);
		self.z = self.z.max(other.z);
	}
}

impl_op_ex!(+ |a: &Vector3, b: &Vector3| -> Vector3 {
	let mut r = *a;
	r += b;
	r
});

impl_op_ex!(- |a: &Vector3, b: &Vector3| -> Vector3 {
	let mut r = *a;
	r -= b;
	r
});

impl_op_ex!(* |a: &Vector3, b: &Vector3| -> Vector3 {
	let mut r = *a;
	r *= b;
	r
});

impl_op_ex!(/ |a: &Vector3, b: &Vector3| -> Vector3 {
	let mut r = *a;
	r /= b;
	r
});

impl_op_ex!(+= |a: &mut Vector3, b: &Vector3| {
	a.x += b.x;
	a.y += b.y;
	a.z += b.z;
});

impl_op_ex!(-= |a: &mut Vector3, b: &Vector3| {
	a.x -= b.x;
	a.y -= b.y;
	a.z -= b.z;
});

impl_op_ex!(*= |a: &mut Vector3, b: &Vector3| {
	a.x *= b.x;
	a.y *= b.y;
	a.z *= b.z;
});

impl_op_ex!(/= |a: &mut Vector3, b: &Vector3| {
	a.x /= b.x;
	a.y /= b.y;
	a.z /= b.z;
});

impl_op_ex!(+ |a: &Vector3, b: f32| -> Vector3 {
	let mut r = *a;
	r += b;
	r
});

impl_op_ex!(- |a: &Vector3, b: f32| -> Vector3 {
	let mut r = *a;
	r -= b;
	r
});

impl_op_ex!(* |a: &Vector3, b: f32| -> Vector3 {
	let mut r = *a;
	r *= b;
	r
});

impl_op_ex!(/ |a: &Vector3, b: f32| -> Vector3 {
	let mut r = *a;
	r /= b;
	r
});

impl_op_ex!(+= |a: &mut Vector3, b: f32| {
	a.x += b;
	a.y += b;
	a.z += b;
});

impl_op_ex!(-= |a: &mut Vector3, b: f32| {
	a.x -= b;
	a.y -= b;
	a.z -= b;
});

impl_op_ex!(*= |a: &mut Vector3, b: f32| {
	a.x *= b;
	a.y *= b;
	a.z *= b;
});

impl_op_ex!(/= |a: &mut Vector3, b: f32| {
	a.x /= b;
	a.y /= b;
	a.z /= b;
});

impl_op_ex!(- |a: &Vector3| -> Vector3 {
	Vector3 {
		x: -a.x,
		y: -a.y,
		z: -a.z
	}
});

impl ApproxEq for Vector3 {
	fn approx_eq(&self, other: &Self, tol: f32) -> bool {
		let x_diff = (self.x - other.x).abs();
		let y_diff = (self.y - other.y).abs();
		let z_diff = (self.z - other.z).abs();

		x_diff <= tol && y_diff <= tol && z_diff <= tol
	}
}

impl Display for Vector3 {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "({} {} {})", self.x, self.y, self.z)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::math::{quaternion, assert_approx_eq};
	use std::f32::consts::FRAC_PI_2;

	#[test]
	fn new() {
		assert_eq!(Vector3::new(1.0, 2.0, 3.0), Vector3 { x: 1.0, y: 2.0, z: 3.0 });
	}

	#[test]
	fn from_scalar() {
		assert_eq!(Vector3::from_scalar(1.0), Vector3 { x: 1.0, y: 1.0, z: 1.0 });
	}

	#[test]
	fn set() {
		let mut v = ZERO;
		v.set(1.0, 2.0, 3.0);
		assert_eq!(v, Vector3 { x: 1.0, y: 2.0, z: 3.0 });
	}

	#[test]
	fn set_from_scalar() {
		let mut v = ZERO;
		v.set_from_scalar(1.0);
		assert_eq!(v, Vector3 { x: 1.0, y: 1.0, z: 1.0 });
	}

	#[test]
	fn set_from_index() {
		let mut v = ZERO;
		v.set_from_index(0, 1.0);
		v.set_from_index(1, 2.0);
		v.set_from_index(2, 3.0);
		assert_eq!(v, Vector3 { x: 1.0, y: 2.0, z: 3.0 });
	}

	#[test]
	#[should_panic]
	fn set_from_index_panics() {
		let mut v = ZERO;
		v.set_from_index(3, 1.0);
	}

	#[test]
	fn get_from_index() {
		let v = Vector3::new(1.0, 2.0, 3.0);
		assert_eq!(v.get_from_index(0), 1.0);
		assert_eq!(v.get_from_index(1), 2.0);
		assert_eq!(v.get_from_index(2), 3.0);
	}

	#[test]
	fn expand() {
		let v = Vector3::new(1.0, 2.0, 3.0);
		assert_eq!(v.expand(4.0), Vector4 { x: 1.0, y: 2.0, z: 3.0, w: 4.0 });
	}

	#[test]
	#[should_panic]
	fn get_from_index_panics() {
		ZERO.get_from_index(3);
	}

	#[test]
	fn length() {
		assert_eq!(Vector3::new(3.0, 0.0, 4.0).length(), 5.0);
	}

	#[test]
	fn length_sq() {
		assert_eq!(Vector3::new(1.0, 2.0, 3.0).length_sq(), 14.0);
	}

	#[test]
	fn normalize() {
		let mut v = Vector3::new(3.0, 0.0, 4.0);
		v.normalize();
		assert_eq!(v, Vector3 { x: 0.6, y: 0.0, z: 0.8 });

		v = ZERO;
		v.normalize();
		assert_eq!(v, Vector3 { x: 0.0, y: 0.0, z: 0.0 });
	}

	#[test]
	fn dot() {
		let a = Vector3::new(1.0, -2.0, 3.0);
		let b = Vector3::new(4.0, -3.0, 1.0);
		assert_eq!(a.dot(&b), 13.0);
	}

	#[test]
	fn cross() {
		let mut v = Vector3::new(1.0, -2.0, 3.0);
		v.cross(&Vector3::new(4.0, -3.0, 1.0));
		assert_eq!(v, Vector3 { x: 7.0, y: 11.0, z: 5.0 });
	}

	#[test]
	fn apply_quaternion() {
		let mut v = Vector3::new(0.0, 0.0, 1.0);
		let mut q = quaternion::ZERO;
		q.set_from_axis_angle(&Vector3::new(0.0, 1.0, 0.0), FRAC_PI_2);
		v.apply_quaternion(&q);
		assert_approx_eq(&v, &Vector3 { x: 1.0, y: 0.0, z: 0.0 }, 1e-6);
	}

	#[test]
	fn min() {
		let mut a = Vector3::new(1.0, -4.2, 0.0);
		a.min(&Vector3::new(-1.0, 2.0, 0.0));
		assert_eq!(a, Vector3 { x: -1.0, y: -4.2, z: 0.0 });
	}

	#[test]
	fn max() {
		let mut a = Vector3::new(1.0, -4.2, 0.0);
		a.max(&Vector3::new(-1.0, 2.0, 0.0));
		assert_eq!(a, Vector3 { x: 1.0, y: 2.0, z: 0.0 });
	}

	#[test]
	fn add_vector() {
		let a = Vector3::new(1.0, -2.0, 3.0);
		let b = Vector3::new(-3.0, 1.0, 2.0);
		assert_eq!(a + b, Vector3 { x: -2.0, y: -1.0, z: 5.0 });
	}

	#[test]
	fn sub_vector() {
		let a = Vector3::new(1.0, -2.0, 3.0);
		let b = Vector3::new(-3.0, 1.0, 2.0);
		assert_eq!(a - b, Vector3 { x: 4.0, y: -3.0, z: 1.0 });
	}

	#[test]
	fn mul_vector() {
		let a = Vector3::new(1.0, -2.0, 3.0);
		let b = Vector3::new(-3.0, 1.0, 2.0);
		assert_eq!(a * b, Vector3 { x: -3.0, y: -2.0, z: 6.0 });
	}

	#[test]
	fn div_vector() {
		let a = Vector3::new(-3.0, 4.0, 9.0);
		let b = Vector3::new(1.0, -2.0, 3.0);
		assert_eq!(a / b, Vector3 { x: -3.0, y: -2.0, z: 3.0 });
	}

	#[test]
	fn add_assign_vector() {
		let mut v = Vector3::new(1.0, -2.0, 3.0);
		v += Vector3::new(-3.0, 1.0, 2.0);
		assert_eq!(v, Vector3 { x: -2.0, y: -1.0, z: 5.0 });
	}

	#[test]
	fn sub_assign_vector() {
		let mut v = Vector3::new(1.0, -2.0, 3.0);
		v -= Vector3::new(-3.0, 1.0, 2.0);
		assert_eq!(v, Vector3 { x: 4.0, y: -3.0, z: 1.0 });
	}

	#[test]
	fn mul_assign_vector() {
		let mut v = Vector3::new(1.0, -2.0, 3.0);
		v *= Vector3::new(-3.0, 1.0, 2.0);
		assert_eq!(v, Vector3 { x: -3.0, y: -2.0, z: 6.0 });
	}

	#[test]
	fn div_assign_vector() {
		let mut v = Vector3::new(-3.0, 4.0, 9.0);
		v /= Vector3::new(1.0, -2.0, 3.0);
		assert_eq!(v, Vector3 { x: -3.0, y: -2.0, z: 3.0 });
	}

	#[test]
	fn add_scalar() {
		let v = Vector3::new(1.0, -2.0, 3.0);
		assert_eq!(v + 3.0, Vector3 { x: 4.0, y: 1.0, z: 6.0 });
	}

	#[test]
	fn sub_scalar() {
		let v = Vector3::new(1.0, -2.0, 3.0);
		assert_eq!(v - 3.0, Vector3 { x: -2.0, y: -5.0, z: 0.0 });
	}

	#[test]
	fn mul_scalar() {
		let v = Vector3::new(1.0, -2.0, 3.0);
		assert_eq!(v * 3.0, Vector3 { x: 3.0, y: -6.0, z: 9.0 });
	}

	#[test]
	fn div_scalar() {
		let v = Vector3::new(-2.0, 4.0, 6.0);
		assert_eq!(v / 2.0, Vector3 { x: -1.0, y: 2.0, z: 3.0 });
	}

	#[test]
	fn add_assign_scalar() {
		let mut v = Vector3::new(1.0, -2.0, 3.0);
		v += 3.0;
		assert_eq!(v, Vector3 { x: 4.0, y: 1.0, z: 6.0 });
	}

	#[test]
	fn sub_assign_scalar() {
		let mut v = Vector3::new(1.0, -2.0, 3.0);
		v -= 3.0;
		assert_eq!(v, Vector3 { x: -2.0, y: -5.0, z: 0.0 });
	}

	#[test]
	fn mul_assign_scalar() {
		let mut v = Vector3::new(1.0, -2.0, 3.0);
		v *= 3.0;
		assert_eq!(v, Vector3 { x: 3.0, y: -6.0, z: 9.0 });
	}

	#[test]
	fn div_assign_scalar() {
		let mut v = Vector3::new(-2.0, 4.0, 6.0);
		v /= 2.0;
		assert_eq!(v, Vector3 { x: -1.0, y: 2.0, z: 3.0 });
	}

	#[test]
	fn neg() {
		let v = Vector3::new(1.0, 2.0, 3.0);
		assert_eq!(-v, Vector3 { x: -1.0, y: -2.0, z: -3.0 });
	}

	#[test]
	fn approx_eq() {
		let a = Vector3::new(1.0, 2.0, 3.0);
		let b = Vector3::new(1.0, 2.0, 3.0);
		assert_approx_eq(&a, &b, 0.0);
	}
}