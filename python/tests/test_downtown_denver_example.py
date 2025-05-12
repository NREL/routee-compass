import json
from unittest import TestCase
from nrel.routee.compass import package_root
from nrel.routee.compass.compass_app import CompassApp


class TestDowntownDenverExample(TestCase):
    def test_downtown_denver_example(self) -> None:
        app = CompassApp.from_config_file(
            package_root()
            / "resources"
            / "downtown_denver_example"
            / "osm_default_energy.toml"
        )

        energy_key = "trip_energy_electric"

        base_query = {
            "origin_name": "NREL",
            "destination_name": "Comrade Brewing Company",
            "destination_y": 39.62627481432341,
            "destination_x": -104.99460207519721,
            "origin_y": 39.798311884359094,
            "origin_x": -104.86796368632217,
            "starting_soc_percent": 100,
            "model_name": "2017_CHEVROLET_Bolt",
        }

        time_optimal_query = dict(base_query)
        time_optimal_query["weights"] = {
            "trip_distance": 0,
            "trip_time": 1,
            energy_key: 1,
        }

        energy_optimal_query = dict(base_query)
        energy_optimal_query["weights"] = {
            "trip_distance": 0,
            "trip_time": 0,
            energy_key: 1,
        }

        balanced_query = dict(base_query)
        balanced_query["weights"] = {
            "trip_distance": 1,
            "trip_time": 1,
            energy_key: 1,
        }

        time_optimal_result = app.run(time_optimal_query)
        energy_optimal_result = app.run(energy_optimal_query)
        balanced_result = app.run(balanced_query)

        if not isinstance(time_optimal_result, dict):
            self.fail(
                f"Time optimal response is not a dict. response: {json.dumps(time_optimal_result, indent=2)}"
            )
        if not isinstance(energy_optimal_result, dict):
            self.fail(
                f"Energy optimal response is not a dict. response: {json.dumps(energy_optimal_result, indent=2)}"
            )
        if not isinstance(balanced_result, dict):
            self.fail(
                f"Balanced response is not a dict. response: {json.dumps(balanced_result, indent=2)}",
            )

        self.assertIsNotNone(
            time_optimal_result.get("route"),
            f"Time optimal result missing route key. \nerror: {time_optimal_result.get('error')}\nresponse: {json.dumps(time_optimal_result, indent=2)}",
        )
        self.assertIsNotNone(
            energy_optimal_result.get("route"),
            f"Energy optimal result missing route key. \nerror: {energy_optimal_result.get('error')}\nresponse: {json.dumps(energy_optimal_result, indent=2)}",
        )
        self.assertIsNotNone(
            balanced_result.get("route"),
            f"Balanced result missing route key. \nerror: {balanced_result.get('error')}\nresponse: {json.dumps(balanced_result, indent=2)}",
        )

        time_optimal_time = time_optimal_result["route"]["traversal_summary"][
            "trip_time"
        ]
        time_optimal_energy = time_optimal_result["route"]["traversal_summary"][
            energy_key
        ]
        time_optimal_cost = time_optimal_result["route"]["cost"]["total_cost"]

        energy_optimal_time = energy_optimal_result["route"]["traversal_summary"][
            "trip_time"
        ]
        energy_optimal_energy = energy_optimal_result["route"]["traversal_summary"][
            energy_key
        ]
        energy_optimal_cost = energy_optimal_result["route"]["cost"]["total_cost"]

        balanced_cost = balanced_result["route"]["cost"]["total_cost"]

        # Test 1: Time optimal has shortest time and greatest energy
        self.assertLessEqual(time_optimal_time, energy_optimal_time)
        self.assertGreaterEqual(time_optimal_energy, energy_optimal_energy)

        # Test 2: Energy optimal has least energy and longest time
        self.assertLessEqual(energy_optimal_energy, time_optimal_energy)
        self.assertGreaterEqual(energy_optimal_time, time_optimal_time)

        # Test 3: Cost optimal has the least total cost
        self.assertLessEqual(balanced_cost, time_optimal_cost)
        self.assertLessEqual(balanced_cost, energy_optimal_cost)

        # Test 4: Monotonicity across weight spectrum
        # Run range of cases from weight p=0 to p=1 for time, with energy=(1-p)
        weight_results = []
        steps = 6  # Test 0, 0.2, 0.4, 0.6, 0.8, 1.0

        for i in range(steps):
            p = i / (steps - 1)  # From 0 to 1
            query = dict(base_query)
            query["weights"] = {"trip_distance": 0, "trip_time": p, energy_key: 1 - p}
            result = app.run(query)
            if not isinstance(result, dict):
                self.fail(
                    f"monotonicity response is not a dict. response: {json.dumps(result, indent=2)}",
                )

            error = result.get("error")
            if error is not None:
                raise Exception(
                    f"Error running query with p={p}: {error}\nresponse: {json.dumps(result, indent=2)}"
                )
            weight_results.append(
                {
                    "p": p,
                    "time": result["route"]["traversal_summary"]["trip_time"],
                    "energy": result["route"]["traversal_summary"][energy_key],
                }
            )

        # As p increases (more weight on time), time should decrease, energy should increase
        for i in range(1, len(weight_results)):
            self.assertLessEqual(
                weight_results[i]["time"],
                weight_results[i - 1]["time"],
                f"Time not decreasing as weight p increases from {weight_results[i - 1]['p']} to {weight_results[i]['p']}",
            )
            self.assertGreaterEqual(
                weight_results[i]["energy"],
                weight_results[i - 1]["energy"],
                f"Energy not increasing as weight p increases from {weight_results[i - 1]['p']} to {weight_results[i]['p']}",
            )
