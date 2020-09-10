# energy-optimal-routing
A routing engine that considers energy weights on edges of a graph for particular vehicle types - built for integration with RouteE.

# setup 

for prototyping, this uses some local wheels that need to be installed manually:

```bash
conda create -n eor python=3.6
conda activate eor

pip install -e .

pip install lib/routee=0.3.1/routee-0.3.1-py3-none-any.whl
```
