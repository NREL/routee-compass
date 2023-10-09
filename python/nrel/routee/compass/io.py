from typing import Callable, Dict, Optional
import numpy as np
import osmnx as ox
from networkx import MultiDiGraph
from pathlib import Path
from pkg_resources import resource_filename
import logging
import csv

try:
    import toml
except Exception:
    try: 
        import tomllib as toml
    except Exception:
        raise ImportError("requires Python 3.11 tomllib or pip install toml for earier Python versions")

log = logging.getLogger("nrel.routee.compass.io")

def generate_compass_dataset(
        g: MultiDiGraph,
        output_directory: Path,
        hwy_speeds: Optional[Dict] = None,
        fallback: Optional[float] = None,
        agg: Callable = np.mean
        ):
    """processes a graph downloaded via OSMNx, generating the set of input
    files required for running RouteE Compass.

    the input graph is assumed to be the direct output of a osmnx download.

    .. code-block:: python

        import osmnx as ox
        g = ox.graph_from_place("Denver, Colorado, USA")
        generate_compass_dataset(g, Path("denver_co"))

    :param g: a network graph
    :type g: MultiDiGraph
    :param output_directory: directory path to use for writing new Compass files
    :type output_directory: Path
    :param hwy_speeds OSM highway types and values = typical speeds (km per
                      hour) to assign to edges of that highway type for any edges missing
                      speed data. Any edges with highway type not in `hwy_speeds` will be
                      assigned the mean preexisting speed value of all edges of that highway
                      type.
    :type hwy_speeds: Optional[Dict]
    :param fallback default speed value (km per hour) to assign to edges whose highway
                    type did not appear in `hwy_speeds` and had no preexisting speed
                    values on any edge
    :type fallback: Optional[float]
    :param agg: aggregation function to impute missing values from observed values.
                the default is numpy.mean, but you might also consider for example
                numpy.median, numpy.nanmedian, or your own custom function
    :type agg: Callable
    """
    
    # pre-process the graph
    g1 = g.copy()
    g1 = ox.utils_graph.get_largest_component(g1)
    g1 = ox.add_edge_speeds(g1, hwy_speeds=hwy_speeds, fallback=fallback, agg=agg)
    v, e = ox.graph_to_gdfs(g1)

    # process vertices
    v = v.reset_index(drop=False).rename(columns={"osmid": "vertex_uuid"})
    v['vertex_id'] = range(len(v))

    # process edges
    lookup = v.set_index("vertex_uuid")
    def replace_id(vertex_uuid):
        return lookup.loc[vertex_uuid].vertex_id
    e = e.reset_index(drop=False).rename(columns={
        "u": "src_vertex_uuid",
        "v": "dst_vertex_uuid",
        "osmid": "edge_uuid",
        "length": "distance"
    })
    e = e[e['key']==0]  # take the first entry regardless of what it is (is this ok?)
    e['edge_id'] = range(len(e))
    e['src_vertex_id'] = e.src_vertex_uuid.apply(replace_id)
    e['dst_vertex_id'] = e.dst_vertex_uuid.apply(replace_id)
    
    # NETWORK FILES
    output_directory.mkdir(parents=True, exist_ok=True)
    #   vertex tables
    v.to_csv(output_directory / "vertices-complete.csv.gz", index=False)
    v[['vertex_id', 'vertex_uuid']].to_csv(output_directory / "vertices-mapping.csv.gz", index=False)
    v[['vertex_uuid']].to_csv(output_directory / "vertices-uuid-enumerated.txt.gz", index=False, header=False)
    v[['vertex_id', 'x', 'y']].to_csv(output_directory / "vertices-compass.csv.gz", index=False)

    #   edge tables (CSV)
    compass_cols = ['edge_id', 'src_vertex_id', 'dst_vertex_id', 'distance']
    e.to_csv(output_directory / "edges-complete.csv.gz", index=False)
    e[compass_cols].to_csv(output_directory / "edges-compass.csv.gz", index=False)
    e[['edge_id', 'edge_uuid']].to_csv(output_directory / "edges-mapping.csv.gz", index=False)

    #   edge tables (TXT)
    e.edge_uuid.to_csv(output_directory / "edges-uuid-enumerated.txt.gz", index=False, header=False)
    np.savetxt(output_directory / "edges-geometries-enumerated.txt.gz", e.geometry, fmt = "%s") # doesn't quote LINESTRINGS
    e.speed_kph.to_csv(output_directory / "edges-posted-speed-enumerated.txt.gz", index=False, header=False)
    e.highway.to_csv(output_directory / "edges-road-class-enumerated.txt.gz", index=False, header=False)

    # DEFAULT CONFIGURATION FILES
    for filename in ['osm_default_speed.toml', 'osm_default_energy.toml']:
        init_toml_file = resource_filename("nrel.routee.compass.io.resources", filename)
        with open(init_toml_file, 'r') as f:
            init_toml = toml.loads(f.read())
        with open(output_directory / filename, 'w') as f:
            f.write(toml.dumps(init_toml))