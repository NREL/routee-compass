import argparse
from enum import Enum
import getpass
import os
from pathlib import Path

from sqlalchemy import create_engine

from nrel.mappymatch.readers.tomtom import read_tomtom_nxmap_from_sql, read_tomtom_igraph_from_sql

from mappymatch.constructs.geofence import Geofence
from mappymatch.utils.crs import LATLON_CRS

parser = argparse.ArgumentParser(description="Download a road map from Trolley")

parser.add_argument("geofence_file", help="Geofence file to use")
parser.add_argument("output_file", help="Output file to write to")
parser.add_argument("--map_type", help="Which type of map to use [igraph, nxmap]", default="igraph")


class MapType(Enum):
    igraph = "igraph"
    nxmap = "nxmap"

    @classmethod
    def from_string(cls, s):
        for m in MapType:
            if m.value == s.lower():
                return m
        raise ValueError(f"Invalid map type {s}")


if __name__ == "__main__":
    args = parser.parse_args()
    user = os.environ.get("TROLLEY_USERNAME")
    if not user:
        user = input("Enter your trolley username: ")

    password = os.environ.get("TROLLEY_PASSWORD")
    if not password:
        password = getpass.getpass("Enter your trolley password: ")

    print("connecting to trolley..")
    engine = create_engine(f"postgresql://{user}:{password}@trolley.nrel.gov:5432/master")

    print("loading geofence file..")
    geofence_path = Path(args.geofence_file)
    geofence = Geofence.from_geojson(geofence_path)

    map_type = MapType.from_string(args.map_type)

    if map_type == MapType.igraph:
        print("building road map from sql..")
        cmap = read_tomtom_igraph_from_sql(engine, geofence, to_crs=LATLON_CRS)
    elif map_type == MapType.nxmap:
        print("building road map from sql..")
        cmap = read_tomtom_nxmap_from_sql(engine, geofence, to_crs=LATLON_CRS)
    else:
        raise ValueError(f"Invalid map type {map_type}")

    print("writing road map to file..")
    outpath = Path(args.output_file)
    cmap.to_file(outpath)
