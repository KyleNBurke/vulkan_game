use crate::math::Quaternion;
use std::ops::{Add, AddAssign, Mul};
use std::fmt::Display;

#[derive(Copy, Clone)]
pub struct Vector3 {
	pub x: f32,
	pub y: f32,
	pub z: f32
}

impl Vector3 {
	pub fn new() -> Self {
		Self { x: 0.0, y: 0.0, z: 0.0 }
	}

	pub fn from(x: f32, y: f32, z: f32) -> Self {
		Self { x, y, z }
	}

	pub fn dot(&self, other: &Self) -> f32 {
		self.x * other.x + self.y * other.y + self.z * other.z
	}

	pub fn cross_vectors(a: &Self, b: &Self) -> Self {
		let mut r = a.clone();
		r.cross(b);
		r
	}

	pub fn cross(&mut self, other: &Self) {
		let x = self.x;
		let y = self.y;
		let z = self.z;

		self.x = y * other.z - z * other.y;
		self.y = z * other.x - x * other.z;
		self.z = x * other.y - y * other.x;
	}

	pub fn apply_quaternion(&mut self, q: &Quaternion) {
		let u = Vector3::from(q.x, q.y, q.z);
		let s = q.w;

		*self = u * u.dot(self) * 2.0 + (*self * (s * s - u.dot(&u))) + Vector3::cross_vectors(&u, self) * s * 2.0;
	}
}

impl Add for Vector3 {
	type Output = Self;

	fn add(self, other: Self) -> Self {
		Self {
			x: self.x + other.x,
			y: self.y + other.y,
			z: self.z + other.z
		}
	}
}

impl AddAssign for Vector3 {
	fn add_assign(&mut self, other: Self) {
		self.x += other.x;
		self.y += other.y;
		self.z += other.z;
	}
}

impl Mul<Vector3> for Vector3 {
	type Output = Self;

	fn mul(self, other: Self) -> Self {
		Self {
			x: self.x * other.x,
			y: self.y * other.y,
			z: self.z * other.z
		}
	}
}

impl Mul<f32> for Vector3 {
	type Output = Self;

	fn mul(self, scalar: f32) -> Self {
		Self {
			x: self.x * scalar,
			y: self.y * scalar,
			z: self.z * scalar
		}
	}
}

impl Display for Vector3 {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "<{} {} {}>", self.x, self.y, self.z)
	}
}