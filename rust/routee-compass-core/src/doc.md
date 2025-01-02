# RouteE Compass Core

The core routing algorithms and abstractions used by [RouteE Compass](https://docs.rs/routee-compass/).

This crate has the following three top-level modules: 
 1. the traits for [data models](#data-model) ([`crate::model`])
 2. implementations for the [search algorithms](#algorithm) which use those models ([`crate::algorithm`])
 3. utility modules ([`crate::util`])

## Data Model

### Top-Level Data Models

A search algorithm for a search query uses a unique [SearchInstance] built of the following types to execute a search. These are grouped into three **categories** for readability: the _topology_ of the network, the _physics_ of how the search updates, and the _metrics_ that are captured by the search. These are either **implemented** with some _static_ type closed for extension (e.g. a Rust `struct`) or, a _dynamic_ type (a Rust `trait object`) which can be implemented in downstream crates. Each of these models is configured in the Compass configuration file with the given configuration **key**.

model | category | implementation | key | description
--- | --- | --- | --- | ---
[Graph] | topology | static | `[graph]` | the road network topology as a vectorized adjacency list 
[MapModel] | topology | static | `[mapping]` | geospatial map matching and LineString reconstruction of routing results
[TraversalModel] | physics | dynamic | `[traversal]` | applies link traversal updates to search state (e.g., link travel time)
[AccessModel] | physics | dynamic | `[access]` | applies updates between link pairs to search state (e.g., turn delays)
[FrontierModel] | physics | dynamic | `[frontier]` | predicates for determining if a given link is traversable
[TerminationModel] | physics | static | `[termination]` | applies rules on compute resource utilization for each search instance
[StateModel] | metrics | static | `[state]` | mapping between domain-level state representation and the vectorized search state
[CostModel] | metrics | static | `[cost]` | maps search state to a cost scalar that is minimized by the search algorithm

### Builder, Service, Model

In Compass, the requirement for phased initialization of (thread-) shared assets is to represent them as new values in the system. 
At initialization, empty **Builder** objects that implement a `build` method can construct a **Service**. 
A **Service** has a lifecycle of the entire program but is not read directly by a search algorithm as it has not yet had the chance to be customized for a given search query.
For that, the **Service** must build a **Model** from the query arguments.
A **Model** is instantiated in the thread when running the search and is destroyed when the search is completed.

For details on the builder, service, and model traits for each dynamic model type, see:
  - [`crate::model::traversal`]
  - [`crate::model::access`]
  - [`crate::model::frontier`]

## Algorithm

RouteE Compass provides implementations for the following search algorithms in the core module. These _algorithms_ are selectable via the [SearchAlgorithm] enum which is configured using the `[algorithm]` configuration key. The algorithm _type_ is either single-sourced shortest path (SSSP) or k-shortest path (KSP).

algorithm | implementation | type | link | description
--- | --- | --- | --- | ---
Dijkstra's Algorithm | [a_star] | SSSP | [wikipedia](https://en.wikipedia.org/wiki/Dijkstra%27s_algorithm) | implemented as A<sup>*</sup> with h(n) = 0 for all vertices
A<sup>*</sup> | [a_star] | SSSP | [wikipedia](https://en.wikipedia.org/wiki/A*_search_algorithm) | graph search with goal-oriented heuristic tuned by the [StateModel], [TraversalModel], [AccessModel] and [CostModel] parameters
Yen's Algorithm | [yens] | KSP | [wikipedia](https://en.wikipedia.org/wiki/Yen%27s_algorithm) | metaheuristic to approximate the k least-cost paths subject to an optional similarity constraint, does not scale well to national routing
Single-Via Paths | [svp] | KSP | [dl.acm.org](https://dl.acm.org/doi/pdf/10.1145/2444016.2444019) | metaheuristic to approximate the k least-cost paths to an optional similarity constraint, suitable for national routing

[Graph]: crate::model::network::Graph
[MapModel]: crate::model::map::MapModel
[TraversalModel]: crate::model::traversal::TraversalModel
[AccessModel]: crate::model::access::AccessModel
[FrontierModel]: crate::model::frontier::FrontierModel
[TerminationModel]: crate::model::termination::TerminationModel
[StateModel]: crate::model::state::StateModel
[CostModel]: crate::model::cost::CostModel
[SearchInstance]: crate::algorithm::search::SearchInstance
[SearchAlgorithm]: crate::algorithm::search::SearchAlgorithm
[a_star]: crate::algorithm::search::a_star
[yens]: crate::algorithm::search::ksp::yens
[svp]: crate::algorithm::search::ksp::svp