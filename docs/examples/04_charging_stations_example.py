# %%
import folium
import geopandas as gpd
from nrel.routee.compass import CompassApp
from nrel.routee.compass.io import generate_compass_dataset, results_to_geopandas
from nrel.routee.compass.plot import plot_route_folium, plot_routes_folium
# %%
import pandas as pd
# %%
vdf = pd.read_csv("./denver_co/vertices-compass.csv.gz")
# %%
vdf
# %%
sub_vdf = vdf.sample(5, random_state=42)
# %%
cdf = sub_vdf[["vertex_id"]].copy()
# %%
cdf["power_type"] = "DCFC"
cdf["power_kw"] = 150
cdf["cost_per_kwh"] = 0.25
# %%
cdf = cdf[["vertex_id", "power_type", "power_kw", "cost_per_kwh"]].reset_index(drop=True)
# %%
cdf.to_csv("./denver_co/charging-stations.csv.gz", index=False, compression="gzip")
# %%
app = CompassApp.from_config_file("./denver_co/osm_default_energy.toml")
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
    "weights": {"trip_distance": 1, "trip_time": 1, "trip_energy": 1},
    "starting_soc_percent": 3,
}
# %%
low_soc_result = app.run(low_soc_query)
# %%
if "error" in low_soc_result:
    print(low_soc_result["error"])
# %%
m = plot_route_folium(low_soc_result)
# %%
sub_vdf
# %%
# plot the charging stations
for t in sub_vdf.itertuples():
    folium.Marker(
        location=[t.y, t.x],
        icon=folium.Icon(color="blue", icon="bolt"),
    ).add_to(m)
# %%
m
# %%
# %%
low_soc_result["route"]["traversal_summary"]
# %%