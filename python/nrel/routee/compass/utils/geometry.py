from nrel.routee.compass.compass_app import Route
import shapely
import json

# routes should exist at a "route.path" key
ROUTE_KEY = "route"
PATH_KEY = "path"


def geometry_from_route(route: Route) -> shapely.geometry.LineString:
    """
    Parse a route dictionary and return a shapely LineString object

    Args:
        route (Route): A route dictionary from the results

    Returns:
        shapely.geometry.LineString: A LineString object representing the route geometry

    Raises:
        KeyError: If the route dictionary does not have a 'route.path' key
        NotImplementedError: If the route dictionary has a multi-geometry
        ValueError: If the route dictionary has an unparseable geometry
    """
    geom = route.get(PATH_KEY)
    if geom is None:
        raise KeyError(
            f"Could not find '{ROUTE_KEY}.{PATH_KEY}' in result. "
            "Make sure the geometry output plugin is activated"
        )
    elif isinstance(geom, list):
        raise NotImplementedError(
            "Multi-geometries are yet not supported. "
            "Please ensure the path only has one geometry"
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

    return linestring
