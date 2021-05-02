use super::{Vector3, Euler, Order, ApproxEq};
use std::ops::{Mul, MulAssign};
use std::fmt::Display;

pub const ZERO: Quaternion = Quaternion { x: 0.0, y: 0.0, z: 0.0, w: 1.0 };

#[derive(Default, Copy, Clone, Debug, PartialEq)]
pub struct Quaternion {
	pub x: f32,
	pub y: f32,
	pub z: f32,
	pub w: f32
}

impl Quaternion {
	pub fn new() -> Self {
		ZERO
	}

	pub fn from(x: f32, y: f32, z: f32, w: f32) -> Self {
		Self { x, y, z, w }
	}

	pub fn set(&mut self, x: f32, y: f32, z: f32, w: f32) {
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

	pub fn set_from_euler(&mut self, e: &Euler) {
		let (cx, cy, cz) = ((e.x / 2.0).cos(), (e.y / 2.0).cos(), (e.z / 2.0).cos());
		let (sx, sy, sz) = ((e.x / 2.0).sin(), (e.y / 2.0).sin(), (e.z / 2.0).sin());

		match e.order {
			Order::Xyz => {
				self.x = sx * cy * cz + cx * sy * sz;
				self.y = cx * sy * cz - sx * cy * sz;
				self.z = cx * cy * sz + sx * sy * cz;
				self.w = cx * cy * cz - sx * sy * sz;
			},
			Order::Xzy => {
				self.x = sx * cy * cz - cx * sy * sz;
				self.y = cx * sy * cz - sx * cy * sz;
				self.z = cx * cy * sz + sx * sy * cz;
				self.w = cx * cy * cz + sx * sy * sz;
			},
			Order::Yxz => {
				self.x = sx * cy * cz + cx * sy * sz;
				self.y = cx * sy * cz - sx * cy * sz;
				self.z = cx * cy * sz - sx * sy * cz;
				self.w = cx * cy * cz + sx * sy * sz;
			},
			Order::Yzx => {
				self.x = sx * cy * cz + cx * sy * sz;
				self.y = cx * sy * cz + sx * cy * sz;
				self.z = cx * cy * sz - sx * sy * cz;
				self.w = cx * cy * cz - sx * sy * sz;
			},
			Order::Zxy => {
				self.x = sx * cy * cz - cx * sy * sz;
				self.y = cx * sy * cz + sx * cy * sz;
				self.z = cx * cy * sz + sx * sy * cz;
				self.w = cx * cy * cz - sx * sy * sz;
			},
			Order::Zyx => {
				self.x = sx * cy * cz - cx * sy * sz;
				self.y = cx * sy * cz + sx * cy * sz;
				self.z = cx * cy * sz - sx * sy * cz;
				self.w = cx * cy * cz + sx * sy * sz;
			}
		}
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

impl ApproxEq for Quaternion {
	fn approx_eq(&self, other: &Self, tol: f32) -> bool {
		let x_diff = (self.x - other.x).abs();
		let y_diff = (self.y - other.y).abs();
		let z_diff = (self.z - other.z).abs();
		let w_diff = (self.w - other.w).abs();

		x_diff <= tol && y_diff <= tol && z_diff <= tol && w_diff <= tol
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
	use crate::math::assert_approx_eq;
	use std::f32::consts::{PI, FRAC_PI_2, FRAC_1_SQRT_2};

	#[test]
	fn new() {
		assert_eq!(Quaternion::new(), Quaternion{ x: 0.0, y: 0.0, z: 0.0, w: 1.0 });
	}

	#[test]
	fn from() {
		assert_eq!(Quaternion::from(1.0, 2.0, 3.0, 4.0), Quaternion{ x: 1.0, y: 2.0, z: 3.0, w: 4.0 });
	}

	#[test]
	fn set() {
		let mut q = Quaternion::new();
		q.set(1.0, 2.0, 3.0, 4.0);
		assert_eq!(q, Quaternion{ x: 1.0, y: 2.0, z: 3.0, w: 4.0 });
	}

	#[test]
	fn set_from_axis_angle() {
		let mut q = Quaternion::new();
		q.set_from_axis_angle(&Vector3::from(1.0, 2.0, 3.0), PI);
		assert_approx_eq(&q, &Quaternion{ x: 1.0, y: 2.0, z: 3.0, w: 0.0 }, 1e-6)
	}

	#[test]
	fn set_from_euler() {
		let mut q = Quaternion::new();

		// XYZ
		q.set_from_euler(&Euler::from(FRAC_PI_2, FRAC_PI_2, FRAC_PI_2, Order::Xyz));
		assert_approx_eq(&q, &Quaternion{ x: FRAC_1_SQRT_2, y: 0.0, z: FRAC_1_SQRT_2, w: 0.0 }, 1e-6);

		// XZY
		q.set_from_euler(&Euler::from(FRAC_PI_2, FRAC_PI_2, FRAC_PI_2, Order::Xzy));
		assert_approx_eq(&q, &Quaternion{ x: 0.0, y: 0.0, z: FRAC_1_SQRT_2, w: FRAC_1_SQRT_2 }, 1e-6);

		// YZX
		q.set_from_euler(&Euler::from(FRAC_PI_2, FRAC_PI_2, FRAC_PI_2, Order::Yzx));
		assert_approx_eq(&q, &Quaternion{ x: FRAC_1_SQRT_2, y: FRAC_1_SQRT_2, z: 0.0, w: 0.0 }, 1e-6);

		// YXZ
		q.set_from_euler(&Euler::from(FRAC_PI_2, FRAC_PI_2, FRAC_PI_2, Order::Yxz));
		assert_approx_eq(&q, &Quaternion{ x: FRAC_1_SQRT_2, y: 0.0, z: 0.0, w: FRAC_1_SQRT_2 }, 1e-6);

		// ZXY
		q.set_from_euler(&Euler::from(FRAC_PI_2, FRAC_PI_2, FRAC_PI_2, Order::Zxy));
		assert_approx_eq(&q, &Quaternion{ x: 0.0, y: FRAC_1_SQRT_2, z: FRAC_1_SQRT_2, w: 0.0 }, 1e-6);

		// ZYX
		q.set_from_euler(&Euler::from(FRAC_PI_2, FRAC_PI_2, FRAC_PI_2, Order::Zyx));
		assert_approx_eq(&q, &Quaternion{ x: 0.0, y: FRAC_1_SQRT_2, z: 0.0, w: FRAC_1_SQRT_2 }, 1e-6);
	}

	#[test]
	fn conjigate() {
		let mut q = Quaternion::from(1.0, 2.0, 3.0, 4.0);
		q.conjigate();
		assert_eq!(q, Quaternion{ x: -1.0, y: -2.0, z: -3.0, w: 4.0 });
	}

	#[test]
	fn dot() {
		let a = Quaternion::from(1.0, 2.0, 3.0, 4.0);
		let b = Quaternion::from(2.0, -3.0, 1.0, 0.0);
		assert_eq!(a.dot(&b), -1.0);
	}

	#[test]
	fn length() {
		assert_eq!(Quaternion::from(5.0, 3.0, 1.0, -1.0).length(), 6.0);
	}

	#[test]
	fn length_sq() {
		assert_eq!(Quaternion::from(5.0, 3.0, 1.0, -1.0).length_sq(), 36.0);
	}

	#[test]
	fn normalize() {
		let mut q = Quaternion::from(0.0, 0.0, 0.0, 0.0);
		q.normalize();
		assert_eq!(q, Quaternion{ x: 0.0, y: 0.0, z: 0.0, w: 1.0 });

		q.set(5.0, 3.0, 1.0, -1.0);
		q.normalize();
		assert_approx_eq(&q, &Quaternion{ x: 0.833, y: 0.5, z: 0.166, w: -0.166 }, 0.001);
	}

	#[test]
	fn mul() {
		let a = Quaternion::from(3.0, 1.0, 2.0, 4.0);
		let b = Quaternion::from(2.0, 5.0, 3.0, 1.0);
		assert_eq!(a * b, Quaternion{ x: 4.0, y: 16.0, z: 27.0, w: -13.0 });
	}

	#[test]
	fn mul_assign() {
		let mut a = Quaternion::from(3.0, 1.0, 2.0, 4.0);
		a *= Quaternion::from(2.0, 5.0, 3.0, 1.0);
		assert_eq!(a, Quaternion{ x: 4.0, y: 16.0, z: 27.0, w: -13.0 });
	}

	#[test]
	fn approx_eq() {
		let a = Quaternion::from(1.0, 2.0, 3.0, 4.0);
		let b = Quaternion::from(1.0, 2.0, 3.0, 4.0);
		assert_approx_eq(&a, &b, 0.0);
	}
}