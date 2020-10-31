use super::{Quaternion, Matrix4, ApproxEq};
use std::fmt::Display;

const SINGULARITY_THRESHOLD: f32 = 0.999999;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Order {
	XYZ,
	XZY,
	YXZ,
	YZX,
	ZXY,
	ZYX
}

impl Display for Order {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{:?}", self)
	}
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Euler {
	pub x: f32,
	pub y: f32,
	pub z: f32,
	pub order: Order
}

impl Euler {
	pub fn new() -> Self {
		Self { x: 0.0, y: 0.0, z: 0.0, order: Order::XYZ }
	}

	pub fn from(x: f32, y: f32, z: f32, order: Order) -> Self {
		Self { x, y, z, order }
	}

	pub fn set(&mut self, x: f32, y: f32, z: f32, order: Order) {
		self.x = x;
		self.y = y;
		self.z = z;
		self.order = order;
	}

	pub fn set_from_rotation_matrix(&mut self, m: &Matrix4) {
		let e = &m.elements;

		match self.order {
			Order::XYZ => {
				self.y = e[0][2].asin();

				if e[0][2] > SINGULARITY_THRESHOLD {
					self.x = e[1][0].atan2(e[1][1]);
					self.z = 0.0;
				}
				else if e[0][2] < -SINGULARITY_THRESHOLD {
					self.x = -e[1][0].atan2(e[1][1]);
					self.z = 0.0;
				}
				else {
					self.x = (-e[1][2]).atan2(e[2][2]);
					self.z = (-e[0][1]).atan2(e[0][0]);
				}
			},
			Order::XZY => {
				self.z = (-e[0][1]).asin();

				if e[0][1] > SINGULARITY_THRESHOLD {
					self.x = (-e[2][0]).atan2(e[2][2]);
					self.y = 0.0;
				}
				else if e[0][1] < -SINGULARITY_THRESHOLD {
					self.x = -(-e[2][0]).atan2(e[2][2]);
					self.y = 0.0;
				}
				else {
					self.x = e[2][1].atan2(e[1][1]);
					self.y = e[0][2].atan2(e[0][0]);
				}
			},
			Order::YXZ => {
				self.x = -e[1][2].asin();

				if e[1][2] > SINGULARITY_THRESHOLD {
					self.y = (-e[0][1]).atan2(e[0][0]);
					self.z = 0.0;
				}
				else if e[1][2] < -SINGULARITY_THRESHOLD {
					self.y = -(-e[0][1]).atan2(e[0][0]);
					self.z = 0.0;
				}
				else {
					self.y = e[0][2].atan2(e[2][2]);
					self.z = e[1][0].atan2(e[1][1]);
				}
			},
			Order::YZX => {
				self.z = e[1][0].asin();

				if e[1][0] > SINGULARITY_THRESHOLD {
					self.x = 0.0;
					self.y = e[2][1].atan2(e[2][2]);
				}
				else if e[1][0] < -SINGULARITY_THRESHOLD {
					self.x = 0.0;
					self.y = -e[2][1].atan2(e[2][2]);
				}
				else {
					self.x = (-e[1][2]).atan2(e[1][1]);
					self.y = (-e[2][0]).atan2(e[0][0]);
				}
			},
			Order::ZXY => {
				self.x = e[2][1].asin();

				if e[2][1] > SINGULARITY_THRESHOLD {
					self.y = 0.0;
					self.z = e[0][2].atan2(e[0][0]);
				}
				else if e[2][1] < -SINGULARITY_THRESHOLD {
					self.y = 0.0;
					self.z = -e[0][2].atan2(e[0][0]);
				}
				else {
					self.y = (-e[2][0]).atan2(e[2][2]);
					self.z = (-e[0][1]).atan2(e[1][1]);
				}
			},
			Order::ZYX => {
				self.y = -e[2][0].asin();

				if e[2][0] > SINGULARITY_THRESHOLD {
					self.x = 0.0;
					self.z = (-e[1][2]).atan2(e[1][1]);
				}
				else if e[2][0] < -SINGULARITY_THRESHOLD {
					self.x = 0.0;
					self.z = -(-e[1][2]).atan2(e[1][1]);
				}
				else {
					self.x = e[2][1].atan2(e[2][2]);
					self.z = e[1][0].atan2(e[0][0]);
				}
			}
		}
	}

	pub fn set_from_quaternion(&mut self, q: &Quaternion) {
		let mut m = Matrix4::new();
		m.make_rotation_from_quaternion(q);
		self.set_from_rotation_matrix(&m);
	}
}

impl Default for Euler {
	fn default() -> Self {
		Self::new()
	}
}

impl ApproxEq for Euler {
	fn approx_eq(&self, other: &Self, tol: f32) -> bool {
		let x_diff = (self.x - other.x).abs();
		let y_diff = (self.y - other.y).abs();
		let z_diff = (self.z - other.z).abs();

		x_diff <= tol && y_diff <= tol && z_diff <= tol && self.order == other.order
	}
}

impl Display for Euler {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "({} {} {} {})", self.x, self.y, self.z, self.order)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::math::assert_approx_eq;
	use std::f32::consts::{PI, FRAC_PI_2};

	#[test]
	fn new() {
		assert_eq!(Euler::new(), Euler { x: 0.0, y: 0.0, z: 0.0, order: Order::XYZ });
	}

	#[test]
	fn default() {
		assert_eq!(Euler::new(), Euler { x: 0.0, y: 0.0, z: 0.0, order: Order::XYZ });
	}

	#[test]
	fn from() {
		let e = Euler::from(1.0, 2.0, 3.0, Order::XYZ);
		assert_eq!(e, Euler { x: 1.0, y: 2.0, z: 3.0, order: Order::XYZ });
	}

	#[test]
	fn set() {
		let mut e = Euler::new();
		e.set(1.0, 2.0, 3.0, Order::XYZ);
		assert_eq!(e, Euler { x: 1.0, y: 2.0, z: 3.0, order: Order::XYZ });
	}

	#[test]
	fn set_from_rotation_matrix() {
		let mut m = Matrix4::new();

		{ // XYZ
			let mut e = Euler::from(0.0, 0.0, 0.0, Order::XYZ);

			// No gimbal lock
			m.set([
				[0.0, -1.0, 0.0, 0.0],
				[0.0, 0.0, -1.0, 0.0],
				[1.0, 0.0, 0.0, 0.0],
				[0.0, 0.0, 0.0, 1.0]
			]);
			e.set_from_rotation_matrix(&m);
			assert_eq!(e, Euler { x: FRAC_PI_2, y: 0.0, z: FRAC_PI_2, order: Order::XYZ });

			// PI / 2
			m.set([
				[0.0, 0.0, 1.0, 0.0],
				[0.0, -1.0, 0.0, 0.0],
				[1.0, 0.0, 0.0, 0.0],
				[0.0, 0.0, 0.0, 1.0]
			]);
			e.set_from_rotation_matrix(&m);
			assert_eq!(e, Euler { x: PI, y: FRAC_PI_2, z: 0.0, order: Order::XYZ });

			// -PI / 2
			m.set([
				[0.0, 0.0, -1.0, 0.0],
				[0.0, 1.0, 0.0, 0.0],
				[1.0, 0.0, 0.0, 0.0],
				[0.0, 0.0, 0.0, 1.0]
			]);
			e.set_from_rotation_matrix(&m);
			assert_eq!(e, Euler { x: 0.0, y: -FRAC_PI_2, z: 0.0, order: Order::XYZ });
		}

		{ // XZY
			let mut e = Euler::from(0.0, 0.0, 0.0, Order::XZY);

			// No gimbal lock
			m.set([
				[0.0, 0.0, 1.0, 0.0],
				[1.0, 0.0, 0.0, 0.0],
				[0.0, 1.0, 0.0, 0.0],
				[0.0, 0.0, 0.0, 1.0]
			]);
			e.set_from_rotation_matrix(&m);
			assert_eq!(e, Euler { x: FRAC_PI_2, y: FRAC_PI_2, z: 0.0, order: Order::XZY });

			// PI / 2
			m.set([
				[0.0, -1.0, 0.0, 0.0],
				[1.0, 0.0, 0.0, 0.0],
				[0.0, 0.0, 1.0, 0.0],
				[0.0, 0.0, 0.0, 1.0]
			]);
			e.set_from_rotation_matrix(&m);
			assert_eq!(e, Euler { x: 0.0, y: 0.0, z: FRAC_PI_2, order: Order::XZY });

			// -PI / 2
			m.set([
				[0.0, 1.0, 0.0, 0.0],
				[1.0, 0.0, 0.0, 0.0],
				[0.0, 0.0, -1.0, 0.0],
				[0.0, 0.0, 0.0, 1.0]
			]);
			e.set_from_rotation_matrix(&m);
			assert_eq!(e, Euler { x: -PI, y: 0.0, z: -FRAC_PI_2, order: Order::XZY });
		}

		{ // YXZ
			let mut e = Euler::from(0.0, 0.0, 0.0, Order::YXZ);

			// No gimbal lock
			m.set([
				[0.0, 0.0, 1.0, 0.0],
				[1.0, 0.0, 0.0, 0.0],
				[0.0, 1.0, 0.0, 0.0],
				[0.0, 0.0, 0.0, 1.0]
			]);
			e.set_from_rotation_matrix(&m);
			assert_eq!(e, Euler { x: 0.0, y: FRAC_PI_2, z: FRAC_PI_2, order: Order::YXZ });

			// PI / 2
			m.set([
				[1.0, 0.0, 0.0, 0.0],
				[0.0, 0.0, -1.0, 0.0],
				[0.0, 1.0, 0.0, 0.0],
				[0.0, 0.0, 0.0, 1.0]
			]);
			e.set_from_rotation_matrix(&m);
			assert_eq!(e, Euler { x: FRAC_PI_2, y: 0.0, z: 0.0, order: Order::YXZ });

			// -PI / 2
			m.set([
				[-1.0, 0.0, 0.0, 0.0],
				[0.0, 0.0, 1.0, 0.0],
				[0.0, 1.0, 0.0, 0.0],
				[0.0, 0.0, 0.0, 1.0]
			]);
			e.set_from_rotation_matrix(&m);
			assert_eq!(e, Euler { x: -FRAC_PI_2, y: -PI, z: 0.0, order: Order::YXZ });
		}

		{ // YZX
			let mut e = Euler::from(0.0, 0.0, 0.0, Order::YZX);

			// No gimbal lock
			m.set([
				[0.0, 1.0, 0.0, 0.0],
				[0.0, 0.0, -1.0, 0.0],
				[-1.0, 0.0, 0.0, 0.0],
				[0.0, 0.0, 0.0, 1.0]
			]);
			e.set_from_rotation_matrix(&m);
			assert_eq!(e, Euler { x: FRAC_PI_2, y: FRAC_PI_2, z: 0.0, order: Order::YZX });

			// PI / 2
			m.set([
				[0.0, 1.0, 0.0, 0.0],
				[1.0, 0.0, 0.0, 0.0],
				[0.0, 0.0, -1.0, 0.0],
				[0.0, 0.0, 0.0, 1.0]
			]);
			e.set_from_rotation_matrix(&m);
			assert_eq!(e, Euler { x: 0.0, y: PI, z: FRAC_PI_2, order: Order::YZX });

			// -PI / 2
			m.set([
				[0.0, 1.0, 0.0, 0.0],
				[-1.0, 0.0, 1.0, 0.0],
				[0.0, 0.0, 1.0, 0.0],
				[0.0, 0.0, 0.0, 1.0]
			]);
			e.set_from_rotation_matrix(&m);
			assert_eq!(e, Euler { x: 0.0, y: 0.0, z: -FRAC_PI_2, order: Order::YZX });
		}

		{ // ZXY
			let mut e = Euler::from(0.0, 0.0, 0.0, Order::ZXY);

			// No gimbal lock
			m.set([
				[0.0, -1.0, 0.0, 0.0],
				[0.0, 0.0, 1.0, 0.0],
				[-1.0, 0.0, 0.0, 0.0],
				[0.0, 0.0, 0.0, 1.0]
			]);
			e.set_from_rotation_matrix(&m);
			assert_eq!(e, Euler { x: 0.0, y: FRAC_PI_2, z: FRAC_PI_2, order: Order::ZXY });

			// PI / 2
			m.set([
				[-1.0, 0.0, 0.0, 0.0],
				[0.0, 0.0, 1.0, 0.0],
				[0.0, 1.0, 0.0, 0.0],
				[0.0, 0.0, 0.0, 1.0]
			]);
			e.set_from_rotation_matrix(&m);
			assert_eq!(e, Euler { x: FRAC_PI_2, y: 0.0, z: PI, order: Order::ZXY });

			// -PI / 2
			m.set([
				[1.0, 0.0, 0.0, 0.0],
				[0.0, 0.0, 1.0, 0.0],
				[0.0, -1.0, 0.0, 0.0],
				[0.0, 0.0, 0.0, 1.0]
			]);
			e.set_from_rotation_matrix(&m);
			assert_eq!(e, Euler { x: -FRAC_PI_2, y: 0.0, z: 0.0, order: Order::ZXY });
		}

		{ // ZYX
			let mut e = Euler::from(0.0, 0.0, 0.0, Order::ZYX);

			// No gimbal lock
			m.set([
				[0.0, 0.0, 1.0, 0.0],
				[1.0, 0.0, 0.0, 0.0],
				[0.0, 1.0, 0.0, 0.0],
				[0.0, 0.0, 0.0, 1.0]
			]);
			e.set_from_rotation_matrix(&m);
			assert_eq!(e, Euler { x: FRAC_PI_2, y: 0.0, z: FRAC_PI_2, order: Order::ZYX });

			// PI / 2
			m.set([
				[0.0, 0.0, 1.0, 0.0],
				[0.0, 1.0, 0.0, 0.0],
				[-1.0, 0.0, 0.0, 0.0],
				[0.0, 0.0, 0.0, 1.0]
			]);
			e.set_from_rotation_matrix(&m);
			assert_eq!(e, Euler { x: 0.0, y: FRAC_PI_2, z: 0.0, order: Order::ZYX });

			// -PI / 2
			m.set([
				[0.0, 0.0, 1.0, 0.0],
				[0.0, -1.0, 0.0, 0.0],
				[1.0, 0.0, 0.0, 0.0],
				[0.0, 0.0, 0.0, 1.0]
			]);
			e.set_from_rotation_matrix(&m);
			assert_eq!(e, Euler { x: 0.0, y: -FRAC_PI_2, z: -PI, order: Order::ZYX });
		}
	}

	#[test]
	fn set_from_quaternion() {
		let mut e = Euler::new();
		e.set_from_quaternion(&Quaternion::from(0.5, 0.5, 0.5, 0.5));

		assert_eq!(e, Euler { x: FRAC_PI_2, y: FRAC_PI_2, z: 0.0, order: Order::XYZ });
	}

	#[test]
	fn approx_eq() {
		let a = Euler::from(1.0, 2.0, 3.0, Order::XYZ);
		let b = Euler::from(1.0, 2.0, 3.0, Order::XYZ);
		assert_approx_eq(&a, &b, 0.0);
	}
}