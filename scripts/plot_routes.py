import webbrowser

import folium

from compass.road_network.base import PathWeight
from compass.road_network.road_network import TomTomNetworkX
from compass.utils.fs import root_dir
from compass.utils.geo_utils import Coordinate

ROAD_NETWORK_FILE = root_dir() / "tests" / "test_assets" / "denver_downtown_tomtom_network.pickle"
OUTFILE = root_dir() / "scripts" / "denver_downtown_routes.html"

if __name__ == "__main__":
    network = TomTomNetworkX(ROAD_NETWORK_FILE)

    home_plate = Coordinate(lat=39.754372, lon=-104.994300)
    bk_lounge = Coordinate(lat=39.779098, lon=-104.951241)

    dist_path = network.shortest_path(home_plate, bk_lounge, weight=PathWeight.DISTANCE)
    time_path = network.shortest_path(home_plate, bk_lounge, weight=PathWeight.TIME)
    enrg_path = network.shortest_path(home_plate, bk_lounge, weight=PathWeight.ENERGY)

    path_mid_point = dist_path[int(len(dist_path) / 2)]

    m = folium.Map(
        location=[path_mid_point.lat, path_mid_point.lon],
        zoom_start=13,
    )
    folium.PolyLine(
        [(c.lat, c.lon) for c in dist_path],
        color="red",
        tooltip="Shortest Distance",
    ).add_to(m)
    folium.PolyLine(
        [(c.lat, c.lon) for c in time_path],
        color="blue",
        tooltip="Shortest Time",
    ).add_to(m)
    folium.PolyLine(
        [(c.lat, c.lon) for c in enrg_path],
        color="green",
        tooltip="Least Energy",
    ).add_to(m)

    m.save(str(OUTFILE))

    webbrowser.open(OUTFILE.as_uri())

