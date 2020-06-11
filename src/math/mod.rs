pub mod vector3;
pub use vector3::Vector3;

pub mod quaternion;
pub use quaternion::Quaternion;

pub mod matrix4;
pub use matrix4::Matrix4;

use std::fmt::Debug;

pub trait ApproxEq {
	fn approx_eq(&self, other: &Self, tol: f32) -> bool;
}

pub fn assert_approx_eq<T: ApproxEq + Debug>(left: &T, right: &T, tol: f32) {
	if !left.approx_eq(right, tol) {
		panic!("assertion failed: `(left ≈ right)`\n  left: `{:?}`\n right: `{:?}`\n   tol: `{}`", left, right, tol);
	}
}