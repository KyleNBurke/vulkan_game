use crate::math::Vector3;
use std::ops::{Mul};

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

	pub fn from(x: f32, y: f32, z: f32, w: f32) -> Self {
		Self { x, y, z, w }
	}

	pub fn conjigate(&mut self) {
		self.x = -self.x;
		self.y = -self.y;
		self.z = -self.z;
	}

	pub fn set_from_axis_angle(&mut self, axis: &Vector3, angle: f32) {
		let half_angle = angle / 2.0;
		let s = half_angle.sin();

		self.x = axis.x * s;
		self.y = axis.y * s;
		self.z = axis.z * s;
		self.w = half_angle.cos();
	}
}

impl PartialEq for Quaternion {
	fn eq(&self, other: &Self) -> bool {
		self.x == other.x && self.y == other.y && self.z == other.z && self.w == other.w
	}
}

impl Mul for Quaternion {
	type Output = Self;

	fn mul(self, rhs: Self) -> Self {
		let a = self;
		let b = rhs;

		Self {
			x:  a.x * b.w + a.y * b.z - a.z * b.y + a.w * b.x,
			y: -a.x * b.z + a.y * b.w + a.z * b.x + a.w * b.y,
			z:  a.x * b.y - a.y * b.x + a.z * b.w + a.w * b.z,
			w: -a.x * b.x - a.y * b.y - a.z * b.z + a.w * b.w
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn new() {
		let q = Quaternion::new();
		assert_eq!(q, Quaternion::from(0.0, 0.0, 0.0, 1.0));
	}

	#[test]
	fn from() {
		let q = Quaternion::from(1.0, 2.0, 3.0, 4.0);
		assert_eq!(q, Quaternion{ x: 1.0, y: 2.0, z: 3.0, w: 4.0 });
	}

	#[test]
	fn conjigate() {
		let mut q = Quaternion::from(1.0, 2.0, 3.0, 4.0);
		q.conjigate();
		assert_eq!(q, Quaternion::from(-1.0, -2.0, -3.0, 4.0));
	}

	#[test]
	fn set_from_axis_angle() {
		todo!();
	}

	#[test]
	fn eq() {
		let a = Quaternion::from(1.0, 2.0, 3.0, 4.0);
		assert_eq!(a, a);
	}

	#[test]
	fn ne() {
		let a = Quaternion::from(1.0, 2.0, 3.0, 4.0);
		let b = Quaternion::from(4.0, 3.0, 2.0, 1.0);
		assert_ne!(a, b);
	}

	#[test]
	fn mul() {
		let a = Quaternion::from(3.0, 1.0, 2.0, 4.0);
		let b = Quaternion::from(2.0, 5.0, 3.0, 1.0);
		let expected = Quaternion::from(4.0, 16.0, 27.0, -13.0);
		assert_eq!(a * b, expected);
	}
}