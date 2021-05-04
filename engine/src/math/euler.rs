use super::{Quaternion, Matrix4, ApproxEq};
use std::fmt::Display;

const SINGULARITY_THRESHOLD: f32 = 0.999999;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Order {
	Xyz,
	Xzy,
	Yxz,
	Yzx,
	Zxy,
	Zyx
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
		Self { x: 0.0, y: 0.0, z: 0.0, order: Order::Xyz }
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
			Order::Xyz => {
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
			Order::Xzy => {
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
			Order::Yxz => {
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
			Order::Yzx => {
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
			Order::Zxy => {
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
			Order::Zyx => {
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
		assert_eq!(Euler::new(), Euler { x: 0.0, y: 0.0, z: 0.0, order: Order::Xyz });
	}

	#[test]
	fn default() {
		assert_eq!(Euler::new(), Euler { x: 0.0, y: 0.0, z: 0.0, order: Order::Xyz });
	}

	#[test]
	fn from() {
		let e = Euler::from(1.0, 2.0, 3.0, Order::Xyz);
		assert_eq!(e, Euler { x: 1.0, y: 2.0, z: 3.0, order: Order::Xyz });
	}

	#[test]
	fn set() {
		let mut e = Euler::new();
		e.set(1.0, 2.0, 3.0, Order::Xyz);
		assert_eq!(e, Euler { x: 1.0, y: 2.0, z: 3.0, order: Order::Xyz });
	}

	#[test]
	fn set_from_rotation_matrix() {
		let mut m = Matrix4::new();

		{ // XYZ
			let mut e = Euler::from(0.0, 0.0, 0.0, Order::Xyz);

			// No gimbal lock
			m.set([
				[0.0, -1.0, 0.0, 0.0],
				[0.0, 0.0, -1.0, 0.0],
				[1.0, 0.0, 0.0, 0.0],
				[0.0, 0.0, 0.0, 1.0]]);
			e.set_from_rotation_matrix(&m);
			assert_eq!(e, Euler { x: FRAC_PI_2, y: 0.0, z: FRAC_PI_2, order: Order::Xyz });

			// PI / 2
			m.set([
				[0.0, 0.0, 1.0, 0.0],
				[0.0, -1.0, 0.0, 0.0],
				[1.0, 0.0, 0.0, 0.0],
				[0.0, 0.0, 0.0, 1.0]]);
			e.set_from_rotation_matrix(&m);
			assert_eq!(e, Euler { x: PI, y: FRAC_PI_2, z: 0.0, order: Order::Xyz });

			// -PI / 2
			m.set([
				[0.0, 0.0, -1.0, 0.0],
				[0.0, 1.0, 0.0, 0.0],
				[1.0, 0.0, 0.0, 0.0],
				[0.0, 0.0, 0.0, 1.0]]);
			e.set_from_rotation_matrix(&m);
			assert_eq!(e, Euler { x: 0.0, y: -FRAC_PI_2, z: 0.0, order: Order::Xyz });
		}

		{ // XZY
			let mut e = Euler::from(0.0, 0.0, 0.0, Order::Xzy);

			// No gimbal lock
			m.set([
				[0.0, 0.0, 1.0, 0.0],
				[1.0, 0.0, 0.0, 0.0],
				[0.0, 1.0, 0.0, 0.0],
				[0.0, 0.0, 0.0, 1.0]]);
			e.set_from_rotation_matrix(&m);
			assert_eq!(e, Euler { x: FRAC_PI_2, y: FRAC_PI_2, z: 0.0, order: Order::Xzy });

			// PI / 2
			m.set([
				[0.0, -1.0, 0.0, 0.0],
				[1.0, 0.0, 0.0, 0.0],
				[0.0, 0.0, 1.0, 0.0],
				[0.0, 0.0, 0.0, 1.0]]);
			e.set_from_rotation_matrix(&m);
			assert_eq!(e, Euler { x: 0.0, y: 0.0, z: FRAC_PI_2, order: Order::Xzy });

			// -PI / 2
			m.set([
				[0.0, 1.0, 0.0, 0.0],
				[1.0, 0.0, 0.0, 0.0],
				[0.0, 0.0, -1.0, 0.0],
				[0.0, 0.0, 0.0, 1.0]]);
			e.set_from_rotation_matrix(&m);
			assert_eq!(e, Euler { x: -PI, y: 0.0, z: -FRAC_PI_2, order: Order::Xzy });
		}

		{ // YXZ
			let mut e = Euler::from(0.0, 0.0, 0.0, Order::Yxz);

			// No gimbal lock
			m.set([
				[0.0, 0.0, 1.0, 0.0],
				[1.0, 0.0, 0.0, 0.0],
				[0.0, 1.0, 0.0, 0.0],
				[0.0, 0.0, 0.0, 1.0]]);
			e.set_from_rotation_matrix(&m);
			assert_eq!(e, Euler { x: 0.0, y: FRAC_PI_2, z: FRAC_PI_2, order: Order::Yxz });

			// PI / 2
			m.set([
				[1.0, 0.0, 0.0, 0.0],
				[0.0, 0.0, -1.0, 0.0],
				[0.0, 1.0, 0.0, 0.0],
				[0.0, 0.0, 0.0, 1.0]]);
			e.set_from_rotation_matrix(&m);
			assert_eq!(e, Euler { x: FRAC_PI_2, y: 0.0, z: 0.0, order: Order::Yxz });

			// -PI / 2
			m.set([
				[-1.0, 0.0, 0.0, 0.0],
				[0.0, 0.0, 1.0, 0.0],
				[0.0, 1.0, 0.0, 0.0],
				[0.0, 0.0, 0.0, 1.0]]);
			e.set_from_rotation_matrix(&m);
			assert_eq!(e, Euler { x: -FRAC_PI_2, y: -PI, z: 0.0, order: Order::Yxz });
		}

		{ // YZX
			let mut e = Euler::from(0.0, 0.0, 0.0, Order::Yzx);

			// No gimbal lock
			m.set([
				[0.0, 1.0, 0.0, 0.0],
				[0.0, 0.0, -1.0, 0.0],
				[-1.0, 0.0, 0.0, 0.0],
				[0.0, 0.0, 0.0, 1.0]]);
			e.set_from_rotation_matrix(&m);
			assert_eq!(e, Euler { x: FRAC_PI_2, y: FRAC_PI_2, z: 0.0, order: Order::Yzx });

			// PI / 2
			m.set([
				[0.0, 1.0, 0.0, 0.0],
				[1.0, 0.0, 0.0, 0.0],
				[0.0, 0.0, -1.0, 0.0],
				[0.0, 0.0, 0.0, 1.0]]);
			e.set_from_rotation_matrix(&m);
			assert_eq!(e, Euler { x: 0.0, y: PI, z: FRAC_PI_2, order: Order::Yzx });

			// -PI / 2
			m.set([
				[0.0, 1.0, 0.0, 0.0],
				[-1.0, 0.0, 1.0, 0.0],
				[0.0, 0.0, 1.0, 0.0],
				[0.0, 0.0, 0.0, 1.0]]);
			e.set_from_rotation_matrix(&m);
			assert_eq!(e, Euler { x: 0.0, y: 0.0, z: -FRAC_PI_2, order: Order::Yzx });
		}

		{ // ZXY
			let mut e = Euler::from(0.0, 0.0, 0.0, Order::Zxy);

			// No gimbal lock
			m.set([
				[0.0, -1.0, 0.0, 0.0],
				[0.0, 0.0, 1.0, 0.0],
				[-1.0, 0.0, 0.0, 0.0],
				[0.0, 0.0, 0.0, 1.0]]);
			e.set_from_rotation_matrix(&m);
			assert_eq!(e, Euler { x: 0.0, y: FRAC_PI_2, z: FRAC_PI_2, order: Order::Zxy });

			// PI / 2
			m.set([
				[-1.0, 0.0, 0.0, 0.0],
				[0.0, 0.0, 1.0, 0.0],
				[0.0, 1.0, 0.0, 0.0],
				[0.0, 0.0, 0.0, 1.0]]);
			e.set_from_rotation_matrix(&m);
			assert_eq!(e, Euler { x: FRAC_PI_2, y: 0.0, z: PI, order: Order::Zxy });

			// -PI / 2
			m.set([
				[1.0, 0.0, 0.0, 0.0],
				[0.0, 0.0, 1.0, 0.0],
				[0.0, -1.0, 0.0, 0.0],
				[0.0, 0.0, 0.0, 1.0]]);
			e.set_from_rotation_matrix(&m);
			assert_eq!(e, Euler { x: -FRAC_PI_2, y: 0.0, z: 0.0, order: Order::Zxy });
		}

		{ // ZYX
			let mut e = Euler::from(0.0, 0.0, 0.0, Order::Zyx);

			// No gimbal lock
			m.set([
				[0.0, 0.0, 1.0, 0.0],
				[1.0, 0.0, 0.0, 0.0],
				[0.0, 1.0, 0.0, 0.0],
				[0.0, 0.0, 0.0, 1.0]]);
			e.set_from_rotation_matrix(&m);
			assert_eq!(e, Euler { x: FRAC_PI_2, y: 0.0, z: FRAC_PI_2, order: Order::Zyx });

			// PI / 2
			m.set([
				[0.0, 0.0, 1.0, 0.0],
				[0.0, 1.0, 0.0, 0.0],
				[-1.0, 0.0, 0.0, 0.0],
				[0.0, 0.0, 0.0, 1.0]]);
			e.set_from_rotation_matrix(&m);
			assert_eq!(e, Euler { x: 0.0, y: FRAC_PI_2, z: 0.0, order: Order::Zyx });

			// -PI / 2
			m.set([
				[0.0, 0.0, 1.0, 0.0],
				[0.0, -1.0, 0.0, 0.0],
				[1.0, 0.0, 0.0, 0.0],
				[0.0, 0.0, 0.0, 1.0]]);
			e.set_from_rotation_matrix(&m);
			assert_eq!(e, Euler { x: 0.0, y: -FRAC_PI_2, z: -PI, order: Order::Zyx });
		}
	}

	#[test]
	fn set_from_quaternion() {
		let mut e = Euler::new();
		e.set_from_quaternion(&Quaternion::from(0.5, 0.5, 0.5, 0.5));
		assert_eq!(e, Euler { x: FRAC_PI_2, y: FRAC_PI_2, z: 0.0, order: Order::Xyz });
	}

	#[test]
	fn approx_eq() {
		let a = Euler::from(1.0, 2.0, 3.0, Order::Xyz);
		let b = Euler::from(1.0, 2.0, 3.0, Order::Xyz);
		assert_approx_eq(&a, &b, 0.0);
	}
}