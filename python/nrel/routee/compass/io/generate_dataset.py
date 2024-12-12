from __future__ import annotations
from typing import Any, Callable, Optional, Union, TYPE_CHECKING
from pathlib import Path

import importlib.resources
import json
import logging
import shutil


from nrel.routee.compass.io import utils
from nrel.routee.compass.io.utils import CACHE_DIR, add_grade_to_graph

if TYPE_CHECKING:
    import networkx
    import pandas as pd

log = logging.getLogger(__name__)


HIGHWAY_TYPE = str
KM_PER_HR = float
HIGHWAY_SPEED_MAP = dict[HIGHWAY_TYPE, KM_PER_HR]

# Parameters annotated with this pass through OSMnx, then GeoPandas, then to Pandas,
# this is a best-effort annotation since the upstream doesn't really have one
AggFunc = Callable[[Any], Any]


def generate_compass_dataset(
    g: networkx.MultiDiGraph,
    output_directory: Union[str, Path],
    hwy_speeds: Optional[HIGHWAY_SPEED_MAP] = None,
    fallback: Optional[float] = None,
    agg: Optional[AggFunc] = None,
    add_grade: bool = False,
    raster_resolution_arc_seconds: Union[str, int] = 1,
    default_config: bool = True,
) -> None:
    """
    Processes a graph downloaded via OSMNx, generating the set of input
    files required for running RouteE Compass.

    The input graph is assumed to be the direct output of an osmnx download.

    Args:
        g: OSMNx graph used to generate input files
        output_directory: Directory path to use for writing new Compass files.
        hwy_speeds: OSM highway types and values = typical speeds (km per
            hour) to assign to edges of that highway type for any edges missing
            speed data. Any edges with highway type not in `hwy_speeds` will be
            assigned the mean preexisting speed value of all edges of that highway
            type.
        fallback: Default speed value (km per hour) to assign to edges whose highway
            type did not appear in `hwy_speeds` and had no preexisting speed
            values on any edge.
        agg: Aggregation function to impute missing values from observed values.
            The default is numpy.mean, but you might also consider for example
            numpy.median, numpy.nanmedian, or your own custom function. Defaults to numpy.mean.
        add_grade (bool, optional): If true, add grade information. Defaults to False. See add_grade_to_graph() for more info.
        raster_resolution_arc_seconds (str, optional): If grade is added, the resolution (in arc-seconds) of the tiles to download (either 1 or 1/3). Defaults to 1.
        default_config (bool, optional): If true, copy default configuration files into the output directory. Defaults to True.

    Example:
        >>> import osmnx as ox
        >>> g = ox.graph_from_place("Denver, Colorado, USA")
        >>> generate_compass_dataset(g, Path("denver_co"))
    """
    try:
        import osmnx as ox
        import numpy as np
        import pandas as pd
        import requests
    except ImportError:
        raise ImportError("requires osmnx to be installed. " "Try 'pip install osmnx'")

    try:
        import toml
    except ImportError:
        try:
            import tomllib as toml  # type: ignore
        except ImportError:
            raise ImportError(
                "requires Python 3.11 tomllib or pip install toml for earier Python versions"
            )

    output_directory = Path(output_directory)

    # default aggregation is via numpy mean operation
    agg = agg if agg is not None else np.mean

    # pre-process the graph
    print("processing graph topology and speeds")
    g1 = ox.truncate.largest_component(g)
    g1 = ox.add_edge_speeds(g1, hwy_speeds=hwy_speeds, fallback=fallback, agg=agg)
    g1 = ox.add_edge_bearings(g1)

    if add_grade:
        print("adding grade information")
        g1 = add_grade_to_graph(
            g1, resolution_arc_seconds=raster_resolution_arc_seconds
        )

    v, e = ox.graph_to_gdfs(g1)

    # process vertices
    print("processing vertices")
    v = v.reset_index(drop=False).rename(columns={"osmid": "vertex_uuid"})
    v["vertex_id"] = range(len(v))

    # process edges
    print("processing edges")
    lookup = v.set_index("vertex_uuid")

    def replace_id(vertex_uuid: pd.Index) -> pd.Series[int]:
        return lookup.loc[vertex_uuid].vertex_id

    e = e.reset_index(drop=False).rename(
        columns={
            "u": "src_vertex_uuid",
            "v": "dst_vertex_uuid",
            "osmid": "edge_uuid",
            "length": "distance",
        }
    )
    e = e[e["key"] == 0]  # take the first entry regardless of what it is (is this ok?)
    e["edge_id"] = range(len(e))
    e["src_vertex_id"] = e.src_vertex_uuid.apply(replace_id)
    e["dst_vertex_id"] = e.dst_vertex_uuid.apply(replace_id)

    # WRITE NETWORK FILES
    output_directory.mkdir(parents=True, exist_ok=True)
    #   vertex tables
    print("writing vertex files")
    v.to_csv(output_directory / "vertices-complete.csv.gz", index=False)
    v[["vertex_id", "vertex_uuid"]].to_csv(
        output_directory / "vertices-mapping.csv.gz", index=False
    )
    v[["vertex_uuid"]].to_csv(
        output_directory / "vertices-uuid-enumerated.txt.gz", index=False, header=False
    )
    v[["vertex_id", "x", "y"]].to_csv(
        output_directory / "vertices-compass.csv.gz", index=False
    )

    #   edge tables (CSV)
    print("writing edge files")
    compass_cols = ["edge_id", "src_vertex_id", "dst_vertex_id", "distance"]
    e.to_csv(output_directory / "edges-complete.csv.gz", index=False)
    e[compass_cols].to_csv(output_directory / "edges-compass.csv.gz", index=False)
    e[["edge_id", "edge_uuid"]].to_csv(
        output_directory / "edges-mapping.csv.gz", index=False
    )

    #   edge tables (TXT)
    print("writing edge attribute files")
    e.edge_uuid.to_csv(
        output_directory / "edges-uuid-enumerated.txt.gz", index=False, header=False
    )
    np.savetxt(
        output_directory / "edges-geometries-enumerated.txt.gz", e.geometry, fmt="%s"
    )  # doesn't quote LINESTRINGS
    e.speed_kph.to_csv(
        output_directory / "edges-posted-speed-enumerated.txt.gz",
        index=False,
        header=False,
    )
    e.highway.to_csv(
        output_directory / "edges-road-class-enumerated.txt.gz",
        index=False,
        header=False,
    )

    headings = [utils.calculate_bearings(i) for i in e.geometry.values]
    headings_df = pd.DataFrame(
        headings, columns=["arrival_heading", "departure_heading"]
    )
    headings_df.to_csv(
        output_directory / "edges-headings-enumerated.csv.gz",
        index=False,
        compression="gzip",
    )

    if add_grade:
        e.grade.to_csv(
            output_directory / "edges-grade-enumerated.txt.gz",
            index=False,
            header=False,
        )

    # COPY DEFAULT CONFIGURATION FILES
    if default_config:
        print("copying default configuration TOML files")
        for filename in [
            "osm_default_distance.toml",
            "osm_default_speed.toml",
            "osm_default_energy.toml",
            "osm_default_energy_all_vehicles.toml",
        ]:
            with importlib.resources.path(
                "nrel.routee.compass.resources", filename
            ) as init_toml_path:
                with init_toml_path.open() as f:
                    init_toml = toml.loads(f.read())
                if filename == "osm_default_energy.toml":
                    if add_grade:
                        init_toml["traversal"]["grade_table_input_file"] = (
                            "edges-grade-enumerated.txt.gz"
                        )
                        init_toml["traversal"]["grade_table_grade_unit"] = "decimal"
            with open(output_directory / filename, "w") as f:
                f.write(toml.dumps(init_toml))

    # DOWLOAD ROUTEE ENERGY MODEL CATALOG
    print("downloading the default RouteE Powertrain models")
    model_output_directory = output_directory / "models"
    if not model_output_directory.exists():
        model_output_directory.mkdir(exist_ok=True)

    with importlib.resources.path(
        "nrel.routee.compass.resources.models", "download_links.json"
    ) as model_link_path:
        with model_link_path.open() as f:
            model_links = json.load(f)

        for model_name, model_link in model_links.items():
            model_destination = model_output_directory / f"{model_name}.bin"

            cached_model_destination = CACHE_DIR / f"{model_name}.bin"
            if not cached_model_destination.exists():
                download_response = requests.get(model_link)
                download_response.raise_for_status()
                with cached_model_destination.open("wb") as f:  # type: ignore
                    f.write(download_response.content)  # type: ignore

            shutil.copy(cached_model_destination, model_destination)
