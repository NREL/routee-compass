// based on https://stackoverflow.com/a/70840233
pub fn linspace(x0: f64, xend: f64, n: usize) -> Vec<f64> {
    let dx = (xend - x0) / ((n - 1) as f64);
    let mut x = vec![x0; n];
    for i in 1..n {
        x[i] = x[i - 1] + dx;
    }
    x
}

pub fn find_nearest_index(arr: &[f64], target: f64) -> Result<usize, String> {
    if &target
        == arr
            .last()
            .ok_or("Could not get last grid value of arr, is arr empty?")?
    {
        return Ok(arr.len() - 2);
    }

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
        Ok(low - 1)
    } else {
        Ok(low)
    }
}
