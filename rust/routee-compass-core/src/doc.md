# Compass Core Crate

Crate containing core routing modules of RouteE Compass used by all downstream crates.
This can be broken up into the following modules:
- [`crate::model`] - [data-model](#data-model) of the search algorithm
  - [`crate::model::graph`] - road network topology collection type
  - [`crate::model::property`] - road network record types
  - [`crate::model::traversal`] - search cost model
  - [`crate::model::frontier`] - search frontier validation predicates
  - [`crate::model::termination::termination_model`] - system-level rules on timeouts + limits for search
- [`crate::algorithm`] - [algorithm](#algorithm) implementations
  - [`crate::algorithm::search`] - search algorithm module
    - [`crate::algorithm::search::search_algorithm_type::SearchAlgorithmType`] - enumeration listing search algorithm types, so far only traditional a star supported
    - [`crate::algorithm::search::a_star::a_star`] - a star search implementation
  - [`crate::algorithm::component:scc`] - strongly-connected components algorithm
- [`crate::util`] - utility modules

### Data Model

RouteE Compass takes a layered approach to modeling the road network.
At the core is the [Graph] model. 
The edges and vertices are stored in `vec`s and carry the minimal data required for most search applications:
- [`crate::model::property::edge::Edge`] records store distance in meters
- [`crate::model::property::vertex::Vertex`] records store coordinate locations, assumed in WGS84 projection

A forward and reverse adjacency list describes connectivity in the graph using the indices of the edges and vertices in their respective `vec`s.

From this alone we can implement a distance-minimizing search.
This is done via a [TraversalModel], which provides an API for computing the costs of traversal based on the search state, graph, and any exogenous datasets.
The convention in RouteE Compass is to load additional `vec`s in the [TraversalModel] which can serve as lookup functions by `EdgeId` for traversal costs and `(EdgeId, EdgeId)` for access costs.
This is also where the compass-powertrain crate loads an energy estimator.
See [TraversalModel] for more details.

### Algorithm

RouteE Compass is set up to provide a suite of search algorithms which may be useful for different search problems.
In all cases, these are assumed to be deterministic searches.
At present, only an a star algorithm is implemented.

Other graph algorithms may be added here in future, such as the connected components module.

[Graph]: crate::model::graph::Graph
[TraversalModel]: crate::model::traversal::traversal_model::TraversalModel