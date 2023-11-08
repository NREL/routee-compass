## Rust Code Style

### Prefer `Box<[T]>` (boxed slices) over `Vec<T>` for large collections

while a `Vec<T>` is more common, it lacks the ability to match capacity exactly with length unless it is set beforehand. in many cases this is not possible. when dealing with large persistent datasets such as `Graph` and `TraversalModel` data, we want to store that data in a `Box<[T]>`. when constructing the dataset, the `Vec` can be used, and then converted to a `Box<[T]>` afterward via `into_boxed_slice`:

```ignore
let path: Path = todo!();
let data: Vec<T> = read_utils::from_csv(&file_path, True, None)?;
let output: Box<[T]> = data.into_boxed_slice();
```

`Vec` is preferred for smaller or non-persistent datasets, or when the additional features of `Vec` are used, such as iteration or dynamic allocation.