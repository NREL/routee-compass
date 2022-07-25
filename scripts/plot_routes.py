import argparse
from pathlib import Path
from typing import List, Tuple
import webbrowser
from compass.rotuee_model_collection import RouteeModelCollection

import folium

from compass.compass_map import CompassMap
from polestar.constructs.geometry import Coordinate
from polestar.graph.graph_interface import Link

parser = argparse.ArgumentParser(description="Plot routee-compass routes")
parser.add_argument("road_network_file", help="Road network file to use")
parser.add_argument("--origin-lat", help="Origin latitude", type=float)
parser.add_argument("--origin-lon", help="Origin longitude", type=float)
parser.add_argument("--dest-lat", help="Destination latitude", type=float)
parser.add_argument("--dest-lon", help="Destination longitude", type=float)
parser.add_argument("--output", help="Output file", default="routes.html")

if __name__ == "__main__":
    args = parser.parse_args()
    road_network_file = Path(args.road_network_file)
    road_map: CompassMap = CompassMap.from_file(road_network_file)

    origin = Coordinate.from_lat_lon(lat=args.origin_lat, lon=args.origin_lon)
    destination = Coordinate.from_lat_lon(lat=args.dest_lat, lon=args.dest_lon)

    # use the default collection which has two models:
    # "Gasoline" and "Electric"
    routee_models = RouteeModelCollection()

    # compute the energy on the road map for each model
    road_map.compute_energy(routee_models)

    dist_path = road_map.route(origin, destination, weight="kilometers")
    time_path = road_map.route(origin, destination, weight="minutes")
    enrg_path = road_map.route(origin, destination, weight="Gasoline")

    path_mid_point = dist_path[int(len(dist_path) / 2)]

    def coords_from_path(path: List[Link]) -> List[Tuple[float, float]]:
        coords = []
        for link in path:
            line = [(lat, lon) for lon, lat in link.geom.coords]
            coords.extend(line)
        return coords

    m = folium.Map(
        location=[path_mid_point.geom.coords[0][1], path_mid_point.geom.coords[0][0]],
        zoom_start=13,
    )
    folium.PolyLine(
        coords_from_path(dist_path),
        color="red",
        tooltip="Shortest Distance",
    ).add_to(m)
    folium.PolyLine(
        coords_from_path(time_path),
        color="blue",
        tooltip="Shortest Time",
    ).add_to(m)
    folium.PolyLine(
        coords_from_path(enrg_path),
        color="green",
        tooltip="Least Energy",
    ).add_to(m)

    outfile = Path(args.output)
    m.save(str(outfile))

    webbrowser.open(outfile.absolute().as_uri())
