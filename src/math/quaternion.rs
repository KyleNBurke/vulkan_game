use crate::math::Vector3;
use std::ops::{Mul, MulAssign};
use std::fmt::Display;

#[derive(Copy, Clone, Debug)]
pub struct Quaternion {
	pub x: f32,
	pub y: f32,
	pub z: f32,
	pub w: f32
}

impl Quaternion {
	pub fn new() -> Self {
		Self { x: 0.0, y: 0.0, z: 0.0, w: 1.0 }
	}

	pub fn from_xyzw(x: f32, y: f32, z: f32, w: f32) -> Self {
		Self { x, y, z, w }
	}

	pub fn set_from_xyzw(&mut self, x: f32, y: f32, z: f32, w: f32) {
		self.x = x;
		self.y = y;
		self.z = z;
		self.w = w;
	}

	pub fn set_from_axis_angle(&mut self, axis: &Vector3, angle: f32) {
		let half_angle = angle / 2.0;
		let s = half_angle.sin();

		self.x = axis.x * s;
		self.y = axis.y * s;
		self.z = axis.z * s;
		self.w = half_angle.cos();
	}

	pub fn conjigate(&mut self) {
		self.x = -self.x;
		self.y = -self.y;
		self.z = -self.z;
	}

	pub fn dot(&self, other: &Self) -> f32 {
		self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
	}

	pub fn length(&self) -> f32 {
		(self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w).sqrt()
	}

	pub fn length_sq(&self) -> f32 {
		self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w
	}

	pub fn normalize(&mut self) {
		let l = self.length();

		if l == 0.0 {
			self.w = 1.0;
		}
		else {
			self.x /= l;
			self.y /= l;
			self.z /= l;
			self.w /= l;
		}
	}

	pub fn approx_eq(&self, other: &Self, tol: f32) -> bool {
		let x_diff = (self.x - other.x).abs();
		let y_diff = (self.y - other.y).abs();
		let z_diff = (self.z - other.z).abs();
		let w_diff = (self.w - other.w).abs();

		x_diff <= tol && y_diff <= tol && z_diff <= tol && w_diff <= tol
	}
}

impl Mul for Quaternion {
	type Output = Self;

	fn mul(self, other: Self) -> Self {
		let a = self;
		let b = other;

		Self {
			x:  a.x * b.w + a.y * b.z - a.z * b.y + a.w * b.x,
			y: -a.x * b.z + a.y * b.w + a.z * b.x + a.w * b.y,
			z:  a.x * b.y - a.y * b.x + a.z * b.w + a.w * b.z,
			w: -a.x * b.x - a.y * b.y - a.z * b.z + a.w * b.w
		}
	}
}

impl MulAssign for Quaternion {
	fn mul_assign(&mut self, other: Self) {
		let (ax, ay, az, aw) = (self.x, self.y, self.z, self.w);
		let (bx, by, bz, bw) = (other.x, other.y, other.z, other.w);

		self.x =  ax * bw + ay * bz - az * by + aw * bx;
		self.y = -ax * bz + ay * bw + az * bx + aw * by;
		self.z =  ax * by - ay * bx + az * bw + aw * bz;
		self.w = -ax * bx - ay * by - az * bz + aw * bw;
	}
}

impl PartialEq for Quaternion {
	fn eq(&self, other: &Self) -> bool {
		self.x == other.x && self.y == other.y && self.z == other.z && self.w == other.w
	}
}

impl Display for Quaternion {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "({} {} {} {})", self.x, self.y, self.z, self.w)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn new() {
		assert_eq!(Quaternion::new(), Quaternion{ x: 0.0, y: 0.0, z: 0.0, w: 1.0 });
	}

	#[test]
	fn from_xyzw() {
		assert_eq!(Quaternion::from_xyzw(1.0, 2.0, 3.0, 4.0), Quaternion{ x: 1.0, y: 2.0, z: 3.0, w: 4.0 });
	}

	#[test]
	fn set_from_xyzw() {
		let mut q = Quaternion::new();
		q.set_from_xyzw(1.0, 2.0, 3.0, 4.0);
		assert_eq!(q, Quaternion{ x: 1.0, y: 2.0, z: 3.0, w: 4.0 });
	}

	#[test]
	fn set_from_axis_angle() {
		let mut q = Quaternion::new();
		q.set_from_axis_angle(&Vector3::from_xyz(1.0, 2.0, 3.0), std::f32::consts::PI);
		assert!(q.approx_eq(&Quaternion{ x: 1.0, y: 2.0, z: 3.0, w: 0.0 }, 0.001));
	}

	#[test]
	fn conjigate() {
		let mut q = Quaternion::from_xyzw(1.0, 2.0, 3.0, 4.0);
		q.conjigate();
		assert_eq!(q, Quaternion{ x: -1.0, y: -2.0, z: -3.0, w: 4.0 });
	}

	#[test]
	fn dot() {
		let a = Quaternion::from_xyzw(1.0, 2.0, 3.0, 4.0);
		let b = Quaternion::from_xyzw(2.0, -3.0, 1.0, 0.0);
		assert_eq!(a.dot(&b), -1.0);
	}

	#[test]
	fn length() {
		assert_eq!(Quaternion::from_xyzw(5.0, 3.0, 1.0, -1.0).length(), 6.0);
	}

	#[test]
	fn length_sq() {
		assert_eq!(Quaternion::from_xyzw(5.0, 3.0, 1.0, -1.0).length_sq(), 36.0);
	}

	#[test]
	fn normalize() {
		let mut q = Quaternion::from_xyzw(0.0, 0.0, 0.0, 0.0);
		q.normalize();
		assert_eq!(q, Quaternion{ x: 0.0, y: 0.0, z: 0.0, w: 1.0 });

		q.set_from_xyzw(5.0, 3.0, 1.0, -1.0);
		q.normalize();
		assert!(q.approx_eq(&Quaternion{ x: 0.833, y: 0.5, z: 0.166, w: -0.166 }, 0.001));
	}

	#[test]
	fn approx_eq() {
		let a = Quaternion::from_xyzw(1.0, 2.0, 3.0, 4.0);
		let b = Quaternion::from_xyzw(1.0, 2.0, 3.0, 4.0);
		assert!(a.approx_eq(&b, 0.0));

		let a = Quaternion::from_xyzw(1.0, 2.0, 3.0, 4.0);
		let b = Quaternion::from_xyzw(2.0, 3.0, 4.0, 5.0);
		assert!(a.approx_eq(&b, 1.0));

		let a = Quaternion::from_xyzw(0.003, -0.0051, 5.0008, 2.0);
		let b = Quaternion::from_xyzw(0.002, -0.006, 5.00001, 2.0);
		assert!(a.approx_eq(&b, 0.001));

		let a = Quaternion::from_xyzw(1.0, 2.0, 3.0, 4.0);
		let b = Quaternion::from_xyzw(1.0, -2.0, 3.0, 4.0);
		assert!(!a.approx_eq(&b, 0.0));

		let a = Quaternion::from_xyzw(0.003, -0.0051, 5.0008, 3.0);
		let b = Quaternion::from_xyzw(0.002, -0.006, 5.01, 6.0);
		assert!(!a.approx_eq(&b, 0.001));
	}

	#[test]
	fn mul() {
		let a = Quaternion::from_xyzw(3.0, 1.0, 2.0, 4.0);
		let b = Quaternion::from_xyzw(2.0, 5.0, 3.0, 1.0);
		assert_eq!(a * b, Quaternion{ x: 4.0, y: 16.0, z: 27.0, w: -13.0 });
	}

	#[test]
	fn mul_assign() {
		let mut a = Quaternion::from_xyzw(3.0, 1.0, 2.0, 4.0);
		a *= Quaternion::from_xyzw(2.0, 5.0, 3.0, 1.0);
		assert_eq!(a, Quaternion{ x: 4.0, y: 16.0, z: 27.0, w: -13.0 });
	}

	#[test]
	fn eq() {
		let a = Quaternion::from_xyzw(1.0, 2.0, 3.0, 4.0);
		let b = Quaternion::from_xyzw(1.0, 2.0, 3.0, 4.0);
		assert_eq!(a, b);
	}
}