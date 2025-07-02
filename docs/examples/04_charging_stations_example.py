# %%
import folium
from nrel.routee.compass import CompassApp
from nrel.routee.compass.io.convert_results import results_to_geopandas
from nrel.routee.compass.plot import plot_route_folium

import pandas as pd
import matplotlib.pyplot as plt

# %%
# %%
# %%
app = CompassApp.from_config_file("./denver_co/osm_default_charging.toml")
# %%
query = {
    "origin_x": -104.969307,
    "origin_y": 39.779021,
    "destination_x": -104.975360,
    "destination_y": 39.693005,
    "model_name": "2017_CHEVROLET_Bolt",
    "weights": {"trip_distance": 1, "trip_time": 1, "trip_energy": 1},
}
# %%
result = app.run(query)
# %%
if "error" in result:
    print(result["error"])
# %%
result["route"]["traversal_summary"]
# %%
plot_route_folium(result)
# %%
low_soc_query = {
    "origin_x": -104.969307,
    "origin_y": 39.779021,
    "destination_x": -104.975360,
    "destination_y": 39.693005,
    "model_name": "2017_CHEVROLET_Bolt",
    "weights": {"trip_distance": 0, "trip_time": 1, "trip_energy": 0},
    "starting_soc_percent": 3,
    "full_soc_percent": 80,
    "valid_power_types": ["DCFC", "L2"],
}
# %%
low_soc_result = app.run(low_soc_query)
# %%
if "error" in low_soc_result:
    print(low_soc_result["error"])
# %%
low_soc_result["route"]["traversal_summary"]
# %%
# %%
m = plot_route_folium(low_soc_result)
m
# %%
cdf = pd.read_csv("./denver_co/charging-stations.csv.gz")
# %%
# plot the charging_stations on the map
for station in cdf.itertuples():
    m.add_child(
        folium.Marker(
            location=[station.y, station.x],
            popup=f"{station.power_type}",
            icon=folium.Icon(color="blue", icon="bolt"),
        )
    )
# %%
m
# %%
# %%
socs = []
time = []
distance = []
for feature in low_soc_result["route"]["path"]["features"]:
    socs.append(feature["properties"]["state"]["trip_soc"])
    time.append(feature["properties"]["state"]["trip_time"])
    distance.append(feature["properties"]["state"]["trip_distance"])
# %%
plt.plot(time, socs)
# %%
plt.plot(distance, socs)
# %%
route_gdf, tree_gdf = results_to_geopandas(low_soc_result)
# %%
tree_gdf
# %%
tree_gdf["trip_soc"] = tree_gdf["state"].apply(lambda x: x["trip_soc"])
tree_gdf["trip_time"] = tree_gdf["state"].apply(lambda x: x["trip_time"])
# %%
tree_gdf = tree_gdf[["trip_soc", "trip_time", "geometry"]]
# %%
tree_gdf.explore(column="trip_time")
# %%
tree_gdf.explore(column="trip_soc")
# %%
