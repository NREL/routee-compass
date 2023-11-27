The RouteE-Compass energy-aware routing engine.

### Crates

This documentation is built around use of the `routee_compass` crate.
This repo is setup as a [workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html) and CompassApp is defined with two upstream dependencies, [routee-compass-core] and [routee-compass-powertrain]:

* [routee-compass-core] - core data structures and algorithms used by Compass
* [routee-compass-powertrain] - traversal model supporting energy-optimal route planning via [RouteE Powertrain](https://github.com/nrel/routee-powertrain)
* [routee-compass] - application built around the core model intended for command-line execution or longer-running applications such as the python sdk (this README)

### Building CompassApp instances

A RouteE Compass app exists as a value of type [CompassApp] on a given system.
An instance can be built using one of two `try_from` methods:
  1. from a path, which assumes the default [CompassAppBuilder]
  2. from an instance of [Config](https://docs.rs/config/latest/config/) along with a (possibly customized) [CompassAppBuilder]

Customizing a [CompassAppBuilder] is the extension point for adding 3rd party extensions to [CompassApp].
If this is not needed, then sticking to the default is sufficient, via the `CompassApp::try_from(path)` builder method.

### Running queries on CompassApp

With a running instance of [CompassApp], one can repeatedly issue queries via the `run` method:

```ignore
let path: PathBuf = todo!();
let app = CompassApp::try_from(path)?;
// use vec![query] to run a single query
let queries: Vec<serde_json::Value> = vec![];

let result = app.run(queries);
```

Based on the parallelism argument to [CompassApp], the batch of queries will be split into chunks in a SIMD parallelization scheme across the available system threads. 
Keep in mind that each chunk needs enough RAM to conduct a search over your road network.
For example, if a road network has 1 million links, and parallelism is 8, then _(in the worst case)_ there should be sufficient RAM to store 8 million rows of search data.

### Customizing RouteE Compass

If you wish to add your own features to a [CompassApp] instance, then see the following links for info on:
  - a custom [TraversalModelBuilder]
  - a custom [FrontierModelBuilder]
  - a custom [InputPluginBuilder]
  - a custom [OutputPluginBuilder]

Any custom builders will need to be added to a [CompassAppBuilder] instance that should be used to create a [CompassApp].

[CompassApp]: crate::app::compass::routee_compass::CompassApp
[CompassAppBuilder]: crate::app::compass::config::routee_compass_builder::CompassAppBuilder
[TraversalModelBuilder]: routee_compass_core::app::compass::config::builders::TraversalModelBuilder
[FrontierModelBuilder]: crate::app::compass::config::builders::FrontierModelBuilder
[InputPluginBuilder]: crate::app::compass::config::builders::InputPluginBuilder
[OutputPluginBuilder]: crate::app::compass::config::builders::OutputPluginBuilder

[routee-compass-core]: routee_compass_core
[routee-compass-powertrain]: routee_compass_powertrain
[routee-compass]: self
