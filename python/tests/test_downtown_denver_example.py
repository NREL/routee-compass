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

        query = {
            "origin_y": 39.742909,
            "origin_x": -104.991595,
            "destination_y": 39.757360,
            "destination_x": -104.988589,
            "model_name": "2016_TOYOTA_Camry_4cyl_2WD",
            "weights": {"distance": 1, "time": 1, "energy_liquid": 1},
        }

        result = app.run(query)

        if not isinstance(result, dict):
            raise ValueError(f"result is not a dict: {result}")

        self.assertTrue(
            "error" not in result,
            msg=f"error in downtown denver test: {result.get('error')}",
        )
