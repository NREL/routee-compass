# %%
from nrel.routee.compass import CompassApp 
from nrel.routee.compass.plot import plot_route_folium
from nrel.routee.compass.io import results_to_geopandas

import pandas as pd
import matplotlib.pyplot as plt

import json

# %%
"""
This example is intended to show how to use the TomTomCompassApp to run a query.

It is expected that you're familiar with the general concepts in RouteE Compass.
If you're not familiar with these, please reference [this example](https://nrel.github.io/routee-compass/examples/01_open_street_maps_example.html)
"""
# %%
app = CompassApp.from_config_file(
    "denver_co/osm_default_energy.toml"
)
# %%
"""
Note that in our query we have a "start_time" and "start_weekday".
These are used to keep track of the time during the search and lookup the speed based on the time of day and day of week.

Also note that we set the "weights" to be distance=0, time=1, energy_electric=0.
This only consider time as a weight to be minimized and will return the fastest route.
If you want to minimize energy consumption, you can set the weights to be distance=0, time=0, energy_electric=1.
If you want to minimize all the features, you can set the weights to be distance=1, time=1, energy_electric=1.
"""
# %%
fastest_time_query = {
    "origin_name": "NREL",
    "destination_name": "Comrade Brewing Company",
    "destination_y": 39.62627481432341,
    "destination_x": -104.99460207519721,
    "origin_y": 39.798311884359094,
    "origin_x": -104.86796368632217,
    "starting_soc_percent": 100,
    "model_name": "2017_CHEVROLET_Bolt",
    "weights": {"distance": 0, "time": 1, "energy_electric": 0},
    "start_time": "05:00:00",
    "start_weekday": "monday",
}

# %%
fastest_time_result = app.run(fastest_time_query)


# %%
def pretty_print(dict):
    print(json.dumps(dict, indent=4))


# %%
pretty_print(fastest_time_result["route"]["traversal_summary"])
# %%
least_energy_query = {
    "origin_name": "NREL",
    "destination_name": "Comrade Brewing Company",
    "destination_y": 39.62627481432341,
    "destination_x": -104.99460207519721,
    "origin_y": 39.798311884359094,
    "origin_x": -104.86796368632217,
    "starting_soc_percent": 100,
    "model_name": "2017_CHEVROLET_Bolt",
    "weights": {"distance": 0, "time": 0, "energy_electric": 1},
    "start_time": "05:00:00",
    "start_weekday": "monday",
}
# %%
least_energy_result = app.run(least_energy_query)
# %%
pretty_print(least_energy_result["route"]["traversal_summary"])
# %%
balanced_query = {
    "origin_name": "NREL",
    "destination_name": "Comrade Brewing Company",
    "destination_y": 39.62627481432341,
    "destination_x": -104.99460207519721,
    "origin_y": 39.798311884359094,
    "origin_x": -104.86796368632217,
    "starting_soc_percent": 100,
    "model_name": "2017_CHEVROLET_Bolt",
    "weights": {"distance": 1, "time": 1, "energy_electric": 1},
    "start_time": "05:00:00",
    "start_weekday": "monday",
}
# %%
balanced_result = app.run(balanced_query)
# %%
pretty_print(balanced_result["route"]["traversal_summary"])
pretty_print(balanced_result["route"]["cost"])
# %%
m = plot_route_folium(fastest_time_result, line_kwargs={"color": "red"})
m = plot_route_folium(least_energy_result, line_kwargs={"color": "blue"}, folium_map=m)
m = plot_route_folium(balanced_result, line_kwargs={"color": "green"}, folium_map=m)
m
# %%
# %%
gdf = results_to_geopandas([fastest_time_result, least_energy_result, balanced_result])
# %%
fastest_time_result["label"] = "Fastest Time"
least_energy_result["label"] = "Least Energy"
balanced_result["label"] = "Least Cost"
plot_df = pd.DataFrame(
    [
        {
            "time": r["route"]["traversal_summary"]["time"],
            "energy": r["route"]["traversal_summary"]["energy_electric"],
            "label": r["label"],
            "cost": r["route"]["cost"]["total_cost"],
        }
        for r in [fastest_time_result, least_energy_result, balanced_result]
    ]
)
# %%
fig, ax = plt.subplots(figsize=(8, 6))

for idx, row in plot_df.iterrows():
    if row["label"] == "Fastest Time":
        color = "red"
    elif row["label"] == "Least Energy":
        color = "blue"
    else:
        color = "green"
    ax.scatter(row["time"], row["energy"], label=row["label"], s=100, color=color)
    ax.text(
        row["time"] + 1,
        row["energy"],
        f"${row['cost']:.2f}",
        fontsize=12,
        verticalalignment="top",
    )

ax.grid(True, linestyle="--", alpha=0.7)
# ax.set_xlim(20, 80)
# ax.set_ylim(2.5, 8.5)
ax.set_xlabel("Time (minutes)", fontsize=12)
ax.set_ylabel("Energy Consumption (kWh)", fontsize=12)
ax.set_title("Route Comparison: Time, Energy, Cost Tradeoffs", fontsize=14)
ax.legend()

plt.show()

# %%
