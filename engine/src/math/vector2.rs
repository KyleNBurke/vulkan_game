use auto_ops::impl_op_ex;

pub const ZERO: Vector2 = Vector2 { x: 0.0, y: 0.0 };

#[derive(Default, Copy, Clone, Debug, PartialEq)]
pub struct Vector2 {
	pub x: f32,
	pub y: f32
}

impl Vector2 {
	pub fn new() -> Self {
		ZERO
	}

	pub fn from(x: f32, y: f32) -> Self {
		Self { x, y }
	}

	pub fn from_scalar(scalar: f32) -> Self {
		Self { x: scalar, y: scalar }
	}

	pub fn set(&mut self, x: f32, y: f32) {
		self.x = x;
		self.y = y;
	}
}

impl_op_ex!(+ |a: &Vector2, b: &Vector2| -> Vector2 {
	let mut r = *a;
	r += b;
	r
});

impl_op_ex!(- |a: &Vector2, b: &Vector2| -> Vector2 {
	let mut r = *a;
	r -= b;
	r
});

impl_op_ex!(* |a: &Vector2, b: &Vector2| -> Vector2 {
	let mut r = *a;
	r *= b;
	r
});

impl_op_ex!(/ |a: &Vector2, b: &Vector2| -> Vector2 {
	let mut r = *a;
	r /= b;
	r
});

impl_op_ex!(+= |a: &mut Vector2, b: &Vector2| {
	a.x += b.x;
	a.y += b.y;
});

impl_op_ex!(-= |a: &mut Vector2, b: &Vector2| {
	a.x -= b.x;
	a.y -= b.y;
});

impl_op_ex!(*= |a: &mut Vector2, b: &Vector2| {
	a.x *= b.x;
	a.y *= b.y;
});

impl_op_ex!(/= |a: &mut Vector2, b: &Vector2| {
	a.x /= b.x;
	a.y /= b.y;
});

impl_op_ex!(+ |a: &Vector2, b: f32| -> Vector2 {
	let mut r = *a;
	r += b;
	r
});

impl_op_ex!(- |a: &Vector2, b: f32| -> Vector2 {
	let mut r = *a;
	r -= b;
	r
});

impl_op_ex!(* |a: &Vector2, b: f32| -> Vector2 {
	let mut r = *a;
	r *= b;
	r
});

impl_op_ex!(/ |a: &Vector2, b: f32| -> Vector2 {
	let mut r = *a;
	r /= b;
	r
});

impl_op_ex!(+= |a: &mut Vector2, b: f32| {
	a.x += b;
	a.y += b;
});

impl_op_ex!(-= |a: &mut Vector2, b: f32| {
	a.x -= b;
	a.y -= b;
});

impl_op_ex!(*= |a: &mut Vector2, b: f32| {
	a.x *= b;
	a.y *= b;
});

impl_op_ex!(/= |a: &mut Vector2, b: f32| {
	a.x /= b;
	a.y /= b;
});

impl_op_ex!(- |a: &Vector2| -> Vector2 {
	Vector2 {
		x: -a.x,
		y: -a.y
	}
});

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn new() {
		assert_eq!(Vector2::new(), Vector2 { x: 0.0, y: 0.0 });
	}

	#[test]
	fn from() {
		assert_eq!(Vector2::from(1.0, 2.0), Vector2 { x: 1.0, y: 2.0 });
	}

	#[test]
	fn from_scalar() {
		assert_eq!(Vector2::from_scalar(1.0), Vector2 { x: 1.0, y: 1.0 });
	}

	#[test]
	fn set() {
		let mut v = Vector2::new();
		v.set(1.0, 2.0);
		assert_eq!(v, Vector2 { x: 1.0, y: 2.0 });
	}

	#[test]
	fn add_vector() {
		let a = Vector2::from(1.0, -2.0);
		let b = Vector2::from(-3.0, 1.0);
		assert_eq!(a + b, Vector2 { x: -2.0, y: -1.0 });
	}

	#[test]
	fn sub_vector() {
		let a = Vector2::from(1.0, -2.0);
		let b = Vector2::from(-3.0, 1.0);
		assert_eq!(a - b, Vector2 { x: 4.0, y: -3.0 });
	}

	#[test]
	fn mul_vector() {
		let a = Vector2::from(1.0, -2.0);
		let b = Vector2::from(-3.0, 1.0);
		assert_eq!(a * b, Vector2 { x: -3.0, y: -2.0 });
	}

	#[test]
	fn div_vector() {
		let a = Vector2::from(-3.0, 4.0);
		let b = Vector2::from(1.0, -2.0);
		assert_eq!(a / b, Vector2 { x: -3.0, y: -2.0 });
	}

	#[test]
	fn add_assign_vector() {
		let mut v = Vector2::from(1.0, -2.0);
		v += Vector2::from(-3.0, 1.0);
		assert_eq!(v, Vector2 { x: -2.0, y: -1.0 });
	}

	#[test]
	fn sub_assign_vector() {
		let mut v = Vector2::from(1.0, -2.0);
		v -= Vector2::from(-3.0, 1.0);
		assert_eq!(v, Vector2 { x: 4.0, y: -3.0 });
	}

	#[test]
	fn mul_assign_vector() {
		let mut v = Vector2::from(1.0, -2.0);
		v *= Vector2::from(-3.0, 1.0);
		assert_eq!(v, Vector2 { x: -3.0, y: -2.0 });
	}

	#[test]
	fn div_assign_vector() {
		let mut v = Vector2::from(-3.0, 4.0);
		v /= Vector2::from(1.0, -2.0);
		assert_eq!(v, Vector2 { x: -3.0, y: -2.0 });
	}

	#[test]
	fn add_scalar() {
		let v = Vector2::from(1.0, -2.0);
		assert_eq!(v + 3.0, Vector2 { x: 4.0, y: 1.0 });
	}

	#[test]
	fn sub_scalar() {
		let v = Vector2::from(1.0, -2.0);
		assert_eq!(v - 3.0, Vector2 { x: -2.0, y: -5.0 });
	}

	#[test]
	fn mul_scalar() {
		let v = Vector2::from(1.0, -2.0);
		assert_eq!(v * 3.0, Vector2 { x: 3.0, y: -6.0 });
	}

	#[test]
	fn div_scalar() {
		let v = Vector2::from(-2.0, 4.0);
		assert_eq!(v / 2.0, Vector2 { x: -1.0, y: 2.0 });
	}

	#[test]
	fn add_assign_scalar() {
		let mut v = Vector2::from(1.0, -2.0);
		v += 3.0;
		assert_eq!(v, Vector2 { x: 4.0, y: 1.0 });
	}

	#[test]
	fn sub_assign_scalar() {
		let mut v = Vector2::from(1.0, -2.0);
		v -= 3.0;
		assert_eq!(v, Vector2 { x: -2.0, y: -5.0 });
	}

	#[test]
	fn mul_assign_scalar() {
		let mut v = Vector2::from(1.0, -2.0);
		v *= 3.0;
		assert_eq!(v, Vector2 { x: 3.0, y: -6.0 });
	}

	#[test]
	fn div_assign_scalar() {
		let mut v = Vector2::from(-2.0, 4.0);
		v /= 2.0;
		assert_eq!(v, Vector2 { x: -1.0, y: 2.0 });
	}

	#[test]
	fn neg() {
		let v = Vector2::from(1.0, 2.0);
		assert_eq!(-v, Vector2 { x: -1.0, y: -2.0 });
	}
}