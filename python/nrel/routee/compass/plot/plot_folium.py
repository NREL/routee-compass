from typing import Any, Callable, Optional, Union
from nrel.routee.compass.plot.plot_utils import ColormapCircularIterator, rgba_to_hex
import json

DEFAULT_LINE_KWARGS = {
    "color": "blue",
    "weight": 10,
    "opacity": 0.8,
}

# routes should exist at a "route" key
ROUTE_KEY = "route"


def plot_route_folium(
    result_dict: dict,
    line_kwargs: Optional[dict] = None,
    folium_map=None,
):
    """
    Plots a single route from a compass query on a folium map.

    Args:
        result_dict (Dict[str, Any]): A result dictionary from a CompassApp query
        line_kwargs (Optional[Dict[str, Any]], optional): A dictionary of keyword
            arguments to pass to the folium Polyline
        folium_map (folium.Map, optional): A existing folium map to plot the route on.
            Defaults to None.

    Returns:
        folium.Map: A folium map with the route plotted on it

    Example:
        >>> from nrel.routee.compass import CompassApp
        >>> from nrel.routee.compass.plot import plot_route_folium
        >>> app = CompassApp.from_config_file("config.toml")
        >>> query = {origin_x: -105.1710052, origin_y: 39.7402804, destination_x: -104.9009913, destination_y: 39.6757025}
        >>> result = app.run(query)
        >>> m = plot_route_folium(result)

    """
    try:
        import folium
        import shapely
    except ImportError:
        raise ImportError(
            "You need to install the folium and shapely packages to use this function"
        )

    if not isinstance(result_dict, dict):
        raise ValueError(f"Expected to get a dictionary but got a {type(result_dict)}")

    geom = result_dict.get(ROUTE_KEY)
    if geom is None:
        raise KeyError(
            f"Could not find '{ROUTE_KEY}' in result. "
            "Make sure the geometry output plugin is activated"
        )

    if isinstance(geom, shapely.geometry.LineString):
        linestring = geom
    if isinstance(geom, str):
        linestring = shapely.from_wkt(geom)
    elif isinstance(geom, bytes):
        linestring = shapely.from_wkb(geom)
    elif isinstance(geom, dict) and geom.get("features") is not None:
        # RouteE Compass can output GeoJson as a GeometryCollection
        # and we expect we can concatenate the result as a single linestring
        feature_collection = shapely.from_geojson(json.dumps(geom))
        multilinestring = shapely.MultiLineString(feature_collection.geoms)
        linestring = shapely.line_merge(multilinestring)
    else:
        raise ValueError("Could not parse route geometry")

    coords = [(lat, lon) for lon, lat in linestring.coords]

    if folium_map is None:
        mid = coords[int(len(coords) / 2)]

        folium_map = folium.Map(location=mid, zoom_start=12)

    if line_kwargs is None:
        kwargs = DEFAULT_LINE_KWARGS
    else:
        kwargs = DEFAULT_LINE_KWARGS
        kwargs.update(line_kwargs)

    folium.PolyLine(
        locations=coords,
        **kwargs,
    ).add_to(folium_map)

    start_icon = folium.Icon(color="green", icon="circle", prefix="fa")
    folium.Marker(
        location=coords[0],
        icon=start_icon,
        tooltip="Origin",
    ).add_to(folium_map)

    end_icon = folium.Icon(color="red", icon="circle", prefix="fa")
    folium.Marker(
        location=coords[-1],
        icon=end_icon,
        tooltip="Destination",
    ).add_to(folium_map)

    return folium_map


def plot_routes_folium(
    results: Union[dict, list[dict]],
    value_fn: Callable[[dict], Any] = lambda r: r["request"].get("name"),
    color_map: str = "viridis",
):
    """
    Plot multiple routes from a CompassApp query on a folium map

    Args:
        results (Union[dict, list[dict]]): A result dictionary or list of result
            dictionaries from a CompassApp query
        value_fn (Callable[[Dict[str, Any]], Any], optional): A function that takes a
            result dictionary and returns a value to use for coloring the routes.
            Defaults to lambda r: r["request"].get("name").
        color_map (str, optional): The name of the matplotlib colormap to use
            for coloring the routes. Defaults to "viridis".

    Returns:
        folium.Map: A folium map with the routes plotted on it

    Example:
        >>> from nrel.routee.compass import CompassApp
        >>> from nrel.routee.compass.plot import plot_results_folium
        >>> app = CompassApp.from_config_file("config.toml")
        >>> query = {origin_x: -105.1710052, origin_y: 39.7402804, destination_x: -104.9009913, destination_y: 39.6757025}
        >>> result = app.run(query)
        >>> m = plot_results_folium(result)


    """
    try:
        import matplotlib.pyplot as plt
        import matplotlib.colors as mcolors
    except ImportError:
        raise ImportError(
            "You need to install the matplotlib package to use this function"
        )

    if isinstance(results, dict):
        results = [results]

    values = [value_fn(result) for result in results]

    cmap = plt.get_cmap(color_map)
    if all(isinstance(v, float) or isinstance(v, int) for v in values):
        norm = mcolors.Normalize(vmin=min(values), vmax=max(values))
        colors = [rgba_to_hex(cmap(norm(v))) for v in values]
    else:
        cmap_iter = ColormapCircularIterator(cmap, len(values))
        colors = [next(cmap_iter) for _ in values]

    folium_map = None
    for result, value, route_color in zip(results, values, colors):
        line_kwargs = {"color": route_color, "tooltip": f"{value}"}
        folium_map = plot_route_folium(result, line_kwargs, folium_map=folium_map)
    return folium_map
