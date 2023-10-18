from __future__ import annotations

from enum import Enum
from pathlib import Path

import math
import itertools
import logging
from typing import Union

log = logging.getLogger(__name__)


class TileResolution(Enum):
    ONE_ARC_SECOND = 1
    ONE_THIRD_ARC_SECOND = 13

    @classmethod
    def from_string(self, string: str) -> TileResolution:
        if string.lower() in ["1", "one"]:
            return TileResolution.ONE_ARC_SECOND
        elif string.lower() in ["1/3", "one third"]:
            return TileResolution.ONE_THIRD_ARC_SECOND
        else:
            raise ValueError(
                f"invalid string {string} for tile resolution. Must be one of: 1, one, 1/3, one third"
            )

    @classmethod
    def from_int(self, int: int) -> TileResolution:
        if int == 1:
            return TileResolution.ONE_ARC_SECOND
        elif int == 13:
            return TileResolution.ONE_THIRD_ARC_SECOND
        else:
            raise ValueError(
                f"invalid int {int} for tile resolution. Must be one of: 1, 13"
            )


def _cover_floats_with_integers(float_min: float, float_max: float) -> list[int]:
    if float_max < float_min:
        raise ValueError("float max must be greater than float min")

    start = math.floor(float_min)
    end = math.ceil(float_max)

    integers = list(range(start, end + 1))
    return integers


def _lat_lon_to_tile(coord: tuple[int, int]) -> str:
    lat, lon = coord
    if lat < 0:
        lat_prefix = "s"
    else:
        lat_prefix = "n"
    if lon < 0:
        lon_prefix = "w"
    else:
        lon_prefix = "e"

    return f"{lat_prefix}{abs(lat)}{lon_prefix}{abs(lon)}"


def _build_download_link(tile: str, resolution=TileResolution.ONE_ARC_SECOND) -> str:
    base_link_fragment = "https://prd-tnm.s3.amazonaws.com/StagedProducts/Elevation/"
    resolution_link_fragment = f"{resolution.value}/TIFF/current/{tile}/"
    filename = f"USGS_{resolution.value}_{tile}.tif"
    link = base_link_fragment + resolution_link_fragment + filename

    return link


def _download_tile(
    tile: str,
    output_dir: Path = Path("cache"),
    resolution=TileResolution.ONE_ARC_SECOND,
) -> Path:
    try:
        import requests
    except ImportError:
        raise ImportError(
            "requires requests to be installed. Try 'pip install requests'"
        )
    url = _build_download_link(tile, resolution)
    filename = url.split("/")[-1]
    destination = output_dir / filename
    if destination.is_file():
        log.info(f"{str(destination)} already exists, skipping")
        return destination

    with requests.get(url, stream=True) as r:
        log.info(f"downloading {tile}")
        r.raise_for_status()

        destination.parent.mkdir(exist_ok=True)

        # write to file in chunks
        with destination.open("wb") as f:
            for chunk in r.iter_content(chunk_size=8192):
                f.write(chunk)

    return destination


def add_grade_to_graph(
    g,
    output_dir: Path = Path("cache"),
    resolution_arc_seconds: Union[str, int] = 1,
):
    """
    Adds grade information to the edges of a graph.
    This will download the necessary elevation data from USGS as raster tiles and cache them in the output_dir.
    The resolution of the tiles can be specified with the resolution parameter.
    USGS has elevation data in increasing resolutions of: 1 arc-second and 1/3 arc-second
    Average tile file sizes for each resolution are about:

    * 1 arc-second: 50 MB
    * 1/3 arc-second: 350 MB

    Args:
        g (nx.MultiDiGraph): The networkx graph to add grades to.
        output_dir (Path, optional): The directory to cache the downloaded tiles in. Defaults to Path("cache").
        resolution_arc_seconds (str, optional): The resolution (in arc-seconds) of the tiles to download (either 1 or 1/3). Defaults to 1.

    Returns:
        nx.MultiDiGraph: The graph with grade information added to the edges.

    Example:
        >>> import osmnx as ox
        >>> g = ox.graph_from_place("Denver, Colorado, USA")
        >>> g = add_grade_to_graph(g)
    """
    try:
        import osmnx as ox
    except ImportError:
        raise ImportError("requires osmnx to be installed. Try 'pip install osmnx'")

    node_gdf = ox.graph_to_gdfs(g, nodes=True, edges=False)

    min_lat = node_gdf.y.min()
    max_lat = node_gdf.y.max()
    min_lon = node_gdf.x.min()
    max_lon = node_gdf.x.max()

    lats = _cover_floats_with_integers(min_lat, max_lat)
    lons = _cover_floats_with_integers(min_lon, max_lon)

    tiles = map(_lat_lon_to_tile, itertools.product(lats, lons))

    if isinstance(resolution_arc_seconds, int):
        resolution = TileResolution.from_int(resolution_arc_seconds)
    elif isinstance(resolution_arc_seconds, str):
        resolution = TileResolution.from_string(resolution_arc_seconds)
    else:
        raise ValueError(
            f"invalid type for resolution {resolution_arc_seconds}."
            "Must be one of: int, str"
        )

    files = []
    for tile in tiles:
        downloaded_file = _download_tile(
            tile, output_dir=output_dir, resolution=resolution
        )
        files.append(str(downloaded_file))

    g = ox.add_node_elevations_raster(g, files)
    g = ox.add_edge_grades(g)

    return g
