pub mod vector2;
pub use vector2::Vector2;

pub mod vector3;
pub use vector3::Vector3;

pub mod quaternion;
pub use quaternion::Quaternion;

pub mod matrix3;
pub use matrix3::Matrix3;

pub mod matrix4;
pub use matrix4::Matrix4;

pub mod euler;
pub use euler::Order;
pub use euler::Euler;

pub mod box3;
pub use box3::Box3;

use std::fmt::Debug;

pub trait ApproxEq {
	fn approx_eq(&self, other: &Self, tol: f32) -> bool;
}

pub fn assert_approx_eq<T: ApproxEq + Debug>(left: &T, right: &T, tol: f32) {
	if !left.approx_eq(right, tol) {
		panic!("assertion failed: `(left ≈ right)`\n  left: `{:?}`\n right: `{:?}`\n   tol: `{}`", left, right, tol);
	}
}