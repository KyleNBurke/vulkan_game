macro_rules! assert_approx_eq {
	($left: expr, $right: expr, $tol: expr) => {
		let left_minus_right = $left - $right;
		let right_minus_left = $right - $left;
		if (left_minus_right > $tol || right_minus_left > $tol) {
			let pos_diff = if left_minus_right >= 0.0 { left_minus_right } else { right_minus_left };
			panic!(
				"assertion failed: `(left ≈ right)`\n  left: `{:?}`\n right: `{:?}`\n  diff: `{:?}`\n   tol: `{:?}`", $left, $right, pos_diff, $tol);
		}
	};
	($left: expr, $right: expr) => (assert_approx_eq!(($left), ($right), 1e-8))
}

macro_rules! assert_approx_neq {
	($left: expr, $right: expr, $tol: expr) => {
		let left_minus_right = $left - $right;
		let right_minus_left = $right - $left;
		if (left_minus_right <= $tol && right_minus_left <= $tol) {
			let pos_diff = if left_minus_right >= 0.0 { left_minus_right } else { right_minus_left };
			panic!(
				"assertion failed: `(left !≈ right)`\n  left: `{:?}`\n right: `{:?}`\n  diff: `{:?}`\n   tol: `{:?}`", $left, $right, pos_diff, $tol);
		}
	};
	($left: expr, $right: expr) => (assert_approx_neq!(($left), ($right), 1e-8))
}

#[cfg(test)]
mod tests {
	#[test]
	fn equal() {
		assert_approx_eq!(1.0, 1.0, 0.0);
		assert_approx_eq!(1.0, -1.0, 2.0);
		assert_approx_eq!(0.0001, 0.0002, 0.0001);
		assert_approx_eq!(0.00000001, 0.000000002);
	}

	#[test]
	fn not_equal() {
		assert_approx_neq!(1.0, 2.0, 0.0);
		assert_approx_neq!(1.0, -1.0, 0.5);
		assert_approx_neq!(0.0001, 0.0002, 0.00001);
		assert_approx_neq!(0.1, 0.2);
	}
}