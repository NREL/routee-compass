use ordered_float::OrderedFloat;

// based on https://stackoverflow.com/a/70840233
pub fn linspace(x0: f64, xend: f64, n: usize) -> Vec<f64> {
    let dx = (xend - x0) / ((n - 1) as f64);
    let mut x = vec![x0; n];
    for i in 1..n {
        x[i] = x[i - 1] + dx;
    }
    x
}

pub fn find_nearest_index(arr: &[OrderedFloat<f64>], target: OrderedFloat<f64>) -> usize {
    let mut low = 0;
    let mut high = arr.len() - 1;

    while low < high {
        let mid = low + (high - low) / 2;

        if arr[mid] >= target {
            high = mid;
        } else {
            low = mid + 1;
        }
    }

    if low > 0 && arr[low] >= target {
        low - 1
    } else {
        low
    }
}
