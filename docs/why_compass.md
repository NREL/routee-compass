# Yet Another Route Planner: RouteE Compass

RouteE Compass was developed at NREL to support energy prediction in mesoscopic route planning, a cornerstone of many areas of mobility research science.
Many such projects attempting to explore routing hit scaling roadblocks that cannot be solved when attempting to use open source libraries in Python or R or route planning APIs.
Compass attempts to overcome these roadblocks by bringing scalable, energy-aware route planning to a data science notebook or to an HPC environment.

![image comparing trip results by algorithm type](./images/example_ksp_comparison.png)

> _Left: An exploration of energy consumption with respect to routing algorithm choice. Routes ranging from blue to pink use a k-shortest path (KSP) algorithm with either a time or distance objective. Routes from yellow to green use endogenous energy estimation and a generalized cost model sweeping energy cost weight from 98% to 100%._
>
> _Right: The KSP routes are shown to have random optimality with respect to time and energy, while the routes using a generalized cost model find the pareto-optimal set of paths between time and energy-optimal, with the cost-optimal path found along the pareto frontier._

## Compass Features

RouteE Compass has a core set of features that set it apart from existing solutions. 
It is designed for batch parallel execution, provides a dynamic cost function API, computes energy endogenously, and is extensible via trait objects.
Each feature is described below in detail.

### Batch Execution

### Dynamic Cost Function

### Endogenous Energy Estimation

### Extendability
