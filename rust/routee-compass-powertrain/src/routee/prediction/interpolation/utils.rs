use ndarray::*;

/// Get all possible indeces of an array of length 2 in `n` dimensions. Result will have shape (2<sup>n</sup>, n).
///
/// # Arguments:
/// * `n` - dimensionality
///
/// # Example:
/// ```rust
/// use multilinear::get_binary_indeces;
/// assert_eq!(
///     get_binary_indeces(2),
///     vec![
///         vec![0, 0],
///         vec![0, 1],
///         vec![1, 0],
///         vec![1, 1],
///     ]
/// );
/// assert_eq!(
///     get_binary_indeces(3),
///     vec![
///         vec![0, 0, 0],
///         vec![0, 0, 1],
///         vec![0, 1, 0],
///         vec![0, 1, 1],
///         vec![1, 0, 0],
///         vec![1, 0, 1],
///         vec![1, 1, 0],
///         vec![1, 1, 1],
///     ]
/// );
/// ```
pub fn get_binary_indeces(n: usize) -> Vec<Vec<usize>> {
    let len = 2_usize.pow(n as u32);
    let mut indeces = Vec::with_capacity(len);
    for i in 0..len {
        let mut index = Vec::with_capacity(n);
        for j in (0..n).rev() {
            index.push(((i >> j) & 1));
        }
        indeces.push(index);
    }
    indeces
}

pub struct MultiLinearInterp {
    grid: Vec<Vec<f64>>,
    values: ArrayD<f64>,
}

impl MultiLinearInterp {
    pub fn new(grid: Vec<Vec<f64>>, values: ArrayD<f64>) -> Result<Self, String> {
        // Dimensionality
        let n = values.ndim();

        // Validate inputs
        if grid.len() != n {
            return Err("Supplied `grid` must have same dimensionality as `values`".to_string());
        }
        for i in 0..n {
            if grid[i].len() <= 1 {
                return Err(format!(
                    "Supplied `grid` length must be > 1 for dimension {}",
                    i
                ));
            }
            if grid[i].len() != values.shape()[i] {
                return Err(format!("Supplied `grid` and `values` are not compatible shapes: dimension {}, lengths {} != {}", i, grid[i].len(), values.shape()[i]));
            }
            if !grid[i].windows(2).all(|w| w[0] < w[1]) {
                return Err(format!("Supplied `grid` coordinates must be sorted and non-repeating: dimension {}, {:?}", i, grid[i]));
            }
        }
        Ok(MultiLinearInterp { grid, values })
    }

    pub fn interpolate(&self, point: &[f64]) -> Result<f64, String> {
        let mut n = self.values.ndim();
        if point.len() != n {
            return Err(format!("Length of supplied `point` must be same as `values` dimensionality: {point:?} is not {n}-dimensional"));
        }
        let mut values = self.values.view();

        if values.len() == 1 {
            // Supplied point is coincident with a grid point, so just return the value
            let first_value: f64 = match values.first() {
                Some(v) => *v,
                None => return Err("Could not find first value from array".to_string()),
            };
            return Ok(first_value);
        }

        // Point can share up to N values of a grid point, which reduces the problem dimensionality
        // i.e. the point shares one of three values of a 3-D grid point, then the interpolation becomes 2-D at that slice
        // or   if the point shares two of three values of a 3-D grid point, then the interpolation becomes 1-D
        let mut grid: Vec<&Vec<f64>> = self.grid.iter().collect();
        let mut point: Vec<f64> = point.to_vec();
        for dim in (0..n).rev() {
            // Range is reversed so that removal doesn't affect indexing
            if let Some(pos) = grid[dim]
                .iter()
                .position(|&grid_point| grid_point == point[dim])
            {
                point.remove(dim);
                grid.remove(dim);
                values.index_axis_inplace(Axis(dim), pos);
            }
        }
        // Simplified dimensionality
        n = values.ndim();

        // Extract the lower and upper indices for each dimension,
        // as well as the fraction of how far the supplied point is between the surrounding grid points
        let mut indeces_lower = Vec::with_capacity(n);
        let mut indeces_upper = Vec::with_capacity(n);
        let mut interp_diffs = Vec::with_capacity(n);
        for dim in 0..n {
            let lower_idx = match grid[dim]
                .windows(2)
                .position(|w| w[0] < point[dim] && point[dim] < w[1]) {
                    Some(idx) => idx,
                    None => return Err(format!(
                            "Supplied `point` is outside of `grid` bounds: {point:?} is not between {grid:?}",
                            point = point,
                            grid = grid
                    )),
                };
            let upper_idx = lower_idx + 1;
            let interp_diff =
                (point[dim] - grid[dim][lower_idx]) / (grid[dim][upper_idx] - grid[dim][lower_idx]);
            indeces_lower.push(lower_idx);
            indeces_upper.push(upper_idx);
            interp_diffs.push(interp_diff);
        }
        // `interp_vals` contains all values surrounding the point of interest, starting with shape (2, 2, ...) in N dimensions
        // this gets mutated and reduces in dimension each iteration, filling with the next values to interpolate with
        // this ends up as a 0-dimensional array containing only the final interpolated value
        let mut interp_vals = values
            .slice_each_axis(|ax| Slice::from(indeces_lower[ax.axis.0]..=indeces_upper[ax.axis.0]))
            .to_owned();
        // Binary is handy as there are 2 surrounding values to index in each dimension: lower and upper
        let mut binary_idxs = get_binary_indeces(n);
        // This loop interpolates in each dimension sequentially
        // each outer loop iteration the dimensionality reduces by 1
        // `interp_vals` ends up as a 0-dimensional array containing only the final interpolated value
        for dim in 0..n {
            let diff = interp_diffs[dim];
            let next_dim = n - 1 - dim;
            // Indeces used for saving results of this dimensions interpolation results
            // assigned to `binary_idxs` at end of loop to be used for indexing in next iteration
            let next_idxs = get_binary_indeces(next_dim);
            let mut intermediate_arr = Array::default(vec![2; next_dim]);
            for i in 0..next_idxs.len() {
                // `next_idxs` is always half the length of `binary_idxs`
                let l = binary_idxs[i].as_slice();
                let u = binary_idxs[next_idxs.len() + i].as_slice();
                if dim == 0 && (interp_vals[l].is_nan() || interp_vals[u].is_nan()) {
                    return Err(format!(
                            "Surrounding value(s) cannot be NaN:\npoint = {point:?},\ngrid = {grid:?},\nvalues = {values:?}"
                    ));
                }
                // This calculation happens 2^(n-1) times in the first iteration of the outer loop,
                // 2^(n-2) times in the second iteration, etc.
                intermediate_arr[next_idxs[i].as_slice()] =
                    interp_vals[l] * (1.0 - diff) + interp_vals[u] * diff;
            }
            binary_idxs = next_idxs;
            interp_vals = intermediate_arr;
        }

        // return the only value contained within the 0-dimensional array
        let val: f64 = match interp_vals.first() {
            Some(v) => *v,
            None => return Err("Could not find first value from array".to_string()),
        };
        Ok(val)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multilinear_1d() {
        let grid = vec![vec![0.0, 1.0, 4.0]];
        let values = array![0.0, 2.0, 4.45].into_dyn();

        let interp = MultiLinearInterp::new(grid, values.clone()).unwrap();

        let point_a = [0.82];
        assert_eq!(interp.interpolate(&point_a).unwrap(), 1.64);

        let point_b = [2.98];
        assert_eq!(interp.interpolate(&point_b).unwrap(), 3.617);

        // returns value at x2
        let point_c = [4.0];
        assert_eq!(interp.interpolate(&point_c).unwrap(), values[2]);
    }

    // test targets found using https://www.omnicalculator.com/math/bilinear-interpolation
    #[test]
    fn test_multilinear_2d() {
        let grid = vec![
            vec![0.0, 1.0, 2.0], // x0, x1, x2
            vec![0.0, 1.0, 2.0], // y0, y1, y2
        ];
        let values = array![
            [0.0, 2.0, 1.9], // (x0, y0), (x0, y1), (x0, y2)
            [2.0, 4.0, 3.1], // (x1, y0), (x1, y1), (x1, y2)
            [5.0, 0.0, 1.4], // (x2, y0), (x2, y1), (x2, y2)
        ]
        .into_dyn();

        let interp = MultiLinearInterp::new(grid, values.clone()).unwrap();

        let point_a = [0.5, 0.5];
        assert_eq!(interp.interpolate(&point_a).unwrap(), 2.0);

        let point_b = [1.52, 0.36];
        assert_eq!(interp.interpolate(&point_b).unwrap(), 2.9696);

        // returns value at (x2, y2)
        let point_c = [2.0, 2.0];
        assert_eq!(interp.interpolate(&point_c).unwrap(), values[[2, 2]]);
    }

    #[test]
    fn test_multilinear_3d() {
        let grid = vec![
            vec![0.0, 1.0, 2.0], // x0, x1, x2
            vec![0.0, 1.0, 2.0], // y0, y1, y2
            vec![0.0, 1.0, 2.0], // z0, z1, z2
        ];
        let values = array![
            [
                [0.0, 1.5, 3.0], // (x0, y0, z0), (x0, y0, z1), (x0, y0, z2)
                [2.0, 0.5, 1.4], // (x0, y1, z0), (x0, y1, z1), (x0, y1, z2)
                [1.9, 5.3, 2.2], // (x0, y2, z0), (x0, y0, z1), (x0, y2, z2)
            ],
            [
                [2.0, 5.1, 1.1], // (x1, y0, z0), (x1, y0, z1), (x1, y0, z2)
                [4.0, 1.0, 0.5], // (x1, y1, z0), (x1, y1, z1), (x1, y1, z2)
                [3.1, 0.9, 1.2], // (x1, y2, z0), (x1, y2, z1), (x1, y2, z2)
            ],
            [
                [5.0, 0.2, 5.1], // (x2, y0, z0), (x2, y0, z1), (x2, y0, z2)
                [0.7, 0.1, 3.2], // (x2, y1, z0), (x2, y1, z1), (x2, y1, z2)
                [1.4, 1.1, 0.0], // (x2, y2, z0), (x2, y2, z1), (x2, y2, z2)
            ],
        ]
        .into_dyn();

        let interp = MultiLinearInterp::new(grid, values.clone()).unwrap();

        let point_a = [0.5, 0.5, 0.5];
        assert_eq!(interp.interpolate(&point_a).unwrap(), 2.0125);

        let point_b = [1.52, 0.36, 0.5];
        assert_eq!(interp.interpolate(&point_b).unwrap(), 2.46272);

        // returns value at (x2, y1, z0)
        let point_c = [2.0, 1.0, 0.0];
        assert_eq!(interp.interpolate(&point_c).unwrap(), values[[2, 1, 0]]);
    }

    #[test]
    fn test_multilinear_with_nans() {
        let grid = vec![
            vec![0.0, 1.0, 2.0, 3.0, 4.0], // x0, x1, x2, x3, x4
            vec![0.0, 1.0, 2.0, 3.0],      // y0, y1, y2, y3
        ];
        let values = array![
            [0.000000, 2.000000, 1.900000, 4.200000], // (x0, y0), (x0, y1), (x0, y2), (x0, y3)
            [2.000000, 4.000000, 3.100000, 6.100000], // (x1, y0), (x1, y1), (x1, y2), (x1, y3)
            [f64::NAN, 0.000000, 1.400000, 1.100000], // (x2, y0), (x2, y1), (x2, y2), (x2, y3)
            [f64::NAN, 0.000000, f64::NAN, f64::NAN], // (x3, y0), (x3, y1), (x3, y2), (x3, y3)
            [f64::NAN, f64::NAN, f64::NAN, f64::NAN], // (x4, y0), (x4, y1), (x4, y2), (x4, y3)
        ];

        let interp = MultiLinearInterp::new(grid, values.into_dyn()).unwrap();

        let point_a = [0.51, 0.36];
        assert_eq!(interp.interpolate(&point_a).unwrap(), 1.74);

        let point_b = [1.5, 2.5];
        assert_eq!(interp.interpolate(&point_b).unwrap(), 2.925);

        let point_c = [1.5, 0.5];
        assert!(interp.interpolate(&point_c).is_err());

        let point_d = [3.5, 2.5];
        assert!(interp.interpolate(&point_d).is_err());
    }
}
