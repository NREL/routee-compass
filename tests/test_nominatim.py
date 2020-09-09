from unittest import TestCase

from eor.geoencoding.nominatim import Nominatim


class TestNominatim(TestCase):
    def test_response(self):
        encoder = Nominatim()
        coord = encoder.get_coordinates("15013 Denver W Pkwy, Golden, CO 80401")
        self.assertAlmostEqual(coord.lat, 39.740, places=2)
        self.assertAlmostEqual(coord.lon, -105.176, places=2)

