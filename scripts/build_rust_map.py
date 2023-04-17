import pandas as pd
import pickle

from nrel.routee.compass.rust.rust_map import build_rust_map_from_gdf

if __name__ == "__main__":
    gdf = pd.read_pickle("/projects/mbap/amazon_eco/us_network.pickle")
    weight_restrictions_file = "/projects/mbap/amazon_eco/weight_restrictions.pickle"
    with open(weight_restrictions_file, "rb") as f:
        weight_restrictions = pickle.load(f)
    height_restrictions_file = "/projects/mbap/amazon_eco/height_restrictions.pickle"
    with open(height_restrictions_file, "rb") as f:
        height_restrictions = pickle.load(f)
    width_restrictions_file = "/projects/mbap/amazon_eco/width_restrictions.pickle"
    with open(width_restrictions_file, "rb") as f:
        width_restrictions = pickle.load(f)
    length_restrictions_file = "/projects/mbap/amazon_eco/length_restrictions.pickle"
    with open(length_restrictions_file, "rb") as f:
        length_restrictions = pickle.load(f)

    rust_map = build_rust_map_from_gdf(
        gdf,
        weight_restrictions=weight_restrictions,
        height_restrictions=height_restrictions,
        width_restrictions=width_restrictions,
        length_restrictions=length_restrictions,
    )
    rust_map.to_file("/scratch/nreinick/us_network_rust_map.bin")
