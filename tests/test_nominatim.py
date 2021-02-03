from unittest import TestCase, skip

from compass.geoencoding.nominatim import Nominatim


# we're not currently using Nominatim so let's skip this
@skip
class TestNominatim(TestCase):
    def test_response(self):
        coord = Nominatim.get_coordinates("15013 Denver W Pkwy, Golden, CO 80401")
        self.assertAlmostEqual(coord.lat, 39.740, places=2)
        self.assertAlmostEqual(coord.lon, -105.176, places=2)

