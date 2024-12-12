# Rust Code Style

## Prefer `Box<[T]>` (boxed slices) over `Vec<T>` for large collections

while a `Vec<T>` is more common, it lacks the ability to match capacity exactly with length unless it is set beforehand. in many cases this is not possible. when dealing with large persistent datasets such as `Graph` and `TraversalModel` data, we want to store that data in a `Box<[T]>`. when constructing the dataset, the `Vec` can be used, and then converted to a `Box<[T]>` afterward via `into_boxed_slice`:

```rust
let path: Path = todo!();
let data: Vec<T> = read_utils::from_csv(&file_path, True, None)?;
let output: Box<[T]> = data.into_boxed_slice();
```

`Vec` is preferred for smaller or non-persistent datasets, or when the additional features of `Vec` are used, such as iteration or dynamic allocation.

## The Builder, Service, Model Convention in RouteE Compass

in RouteE Compass, large Traversal and Frontier objects are loaded once and shared across threads.
they are built in two phases: once at application initialization, and once again for each query, so that query-specific parameters can be applied.
these phases are represented by the following types:

| phase       | description                                                      | lifetime                                              | Frontier               | Traversal               |
| ----------- | ---------------------------------------------------------------- | ----------------------------------------------------- | ---------------------- | ----------------------- |
| **builder** | an empty struct with a `build` method that creates a **service** | app initialization only                               | `FrontierModelBuilder` | `TraversalModelBuilder` |
| **service** | struct with a `build` method that creates a **model**            | entire program lifetime (same as CompassApp instance) | `FrontierModelService` | `TraversalModelService` |
| **model**   | object used by the search algorithm                              | duration of a single query                            | `FrontierModel`        | `TraversalModel`        |

when we apply the `build` methods, we get these results (using the travel time `TraversalModel` as an example):

```rust
let config: serde_json::Value = json!(); // application configuration
let query: serde_json::Value = json!();  // a single user search query
let builder: Box<dyn TraversalModelBuilder> = Box::new(SpeedLookupBuilder {});
let service: Arc<dyn TraversalModelService> = builder.build(&config)?;
let model: Arc<dyn TraversalModel> = service.build(&query);
```

the **builder** object instances are wrapped in a `Box` referenced by the `CompassAppBuilder` and used when creating `CompassApp` instances. once we build a **service** from the **builder**, the app requires that they are wrapped in an `Arc`, which is a thread-safe pointer. this way, the **service** can be shared across threads so we can build a **model** for a specific user query from within a query thread.
