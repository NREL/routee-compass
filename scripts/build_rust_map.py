import pandas as pd

from nrel.routee.compass.rust.rust_map import build_rust_map_from_gdf

if __name__ == "__main__":
    gdf = pd.read_pickle("/scratch/nreinick/us_network.pickle")
    rust_map = build_rust_map_from_gdf(gdf)
    rust_map.to_file("/scratch/nreinick/us_network_rust_map.bin")