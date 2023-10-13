from typing import Any, Callable, Union
from nrel.routee.compass.plot.plot_utils import ColormapCircularIterator, rgba_to_hex


def plot_route_folium(
    result_dict, route_name=None, route_color="blue", folium_map=None
):
    """
    Plots a single route from a compass query on a folium map.

    Args:
        result_dict (Dict[str, Any]): A result dictionary from a CompassApp query
        route_name (str, optional): The name of the route. Defaults to None.
        route_color (str, optional): The color of the route. Defaults to "blue".
        folium_map (folium.Map, optional): A existing folium map to plot the route on. Defaults to None.

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

    geom = result_dict.get("geometry")
    if geom is None:
        raise KeyError(
            "Could not find geometry in result. "
            "Make sure the geometry output plugin is activated"
        )

    if isinstance(geom, shapely.geometry.LineString):
        linestring = geom
    if isinstance(geom, str):
        linestring = shapely.from_wkt(geom)
    elif isinstance(geom, bytes):
        linestring = shapely.from_wkb(geom)
    else:
        raise ValueError("Could not parse geometry")

    coords = [(lat, lon) for lon, lat in linestring.coords]

    if folium_map is None:
        mid = coords[int(len(coords) / 2)]

        folium_map = folium.Map(location=mid, zoom_start=12)

    folium.PolyLine(
        locations=coords,
        weight=10,
        opacity=0.8,
        color=route_color,
        tooltip=route_name,
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


def plot_results_folium(
    results: Union[dict, list[dict]],
    value_fn: Callable[[dict], Any] = lambda r: r["request"].get("name"),
    color_map: str = "viridis",
):
    """
    Plot all results from a CompassApp query on a folium map

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
        import matplotlib.colors as colors
    except ImportError:
        raise ImportError(
            "You need to install the matplotlib package to use this function"
        )

    if isinstance(results, dict):
        results = [results]

    values = [value_fn(result) for result in results]

    cmap = plt.get_cmap(color_map)
    if all(isinstance(v, float) or isinstance(v, int) for v in values):
        norm = colors.Normalize(vmin=min(values), vmax=max(values))
        colors = [rgba_to_hex(cmap(norm(v))) for v in values]
    else:
        cmap_iter = ColormapCircularIterator(cmap, len(values))
        colors = [next(cmap_iter) for _ in values]

    folium_map = None
    for result, value, route_color in zip(results, values, colors):
        folium_map = plot_route_folium(
            result, value, route_color, folium_map=folium_map
        )
    return folium_map
