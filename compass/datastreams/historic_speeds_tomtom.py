import json
import logging
import time
import requests
import geopandas as gpd

from datetime import datetime, timedelta
from typing import Optional


from compass.datastreams.base import DataStream
from compass.road_network.base import RoadNetwork
from compass.utils.geo_utils import BoundingBox, GeoJsonFeatures
from compass.utils.units import *

log = logging.getLogger(__name__)


class HistoricSpeedsTomTomStream(DataStream):
    """
    pings the TomTom traffic stats api and updates the road network with new speeds every hour.
    """

    api_key = "j8DuYC17MaGl0UcKLcI9gGSd1e3rpvaJ"
    post_https_base = "https://api.tomtom.com/traffic/trafficstats/caa/1"
    get_https_base = "https://api.tomtom.com/traffic/trafficstats/status/1/"
    json_skeleton = """
    {
        "jobName":"",
        "distanceUnit":"KILOMETERS",
        "network": {
          "name": "",
          "boundingBox" : {},
          "timeZoneId": "",
          "frcs": ["0","1","2","3","4","5","6"],
          "probeSource":"ALL"
        },
        "dateRange": {},
        "timeSets":[]
     }
    """

    def __init__(self, timezone_str: str = "US/Mountain"):
        json_body = json.loads(self.json_skeleton)

        # TODO: get this from the road network -ndr
        json_body["network"]["timeZoneId"] = timezone_str

        self.json_body = json_body

    def _post_query(self, road_network: RoadNetwork) -> requests.Response:
        last_week = datetime.now() - timedelta(days=7)

        json_body = self.json_body

        json_body["network"]["name"] = road_network.bbox.bbox_id
        json_body["network"]["boundingBox"] = road_network.bbox.as_tomtom_json()

        json_body["jobName"] = last_week.strftime("%Y-%m-%d-%H-%M")
        json_body["dateRange"] = {
            "name": last_week.strftime("%Y-%m-%d"),
            "from": last_week.strftime("%Y-%m-%d"),
            "to": (last_week + timedelta(days=1)).strftime("%Y-%m-%d"),
        }

        current_hour = last_week.strftime('%H:00')
        next_hour = (last_week + timedelta(hours=1)).strftime('%H:00')

        json_body["timeSets"] = [{
            'name': last_week.strftime("%H_%p"),
            'timeGroups': [{
                "days": [last_week.strftime("%a").upper()],
                "times": [f"{current_hour}-{next_hour}"]}
            ]
        }]

        post_https = self.post_https_base + f"?key={self.api_key}"

        # TODO: setting verify=False allow this to run with the VPN for testing but we should re-enable SSL certs
        #  when going live.
        log.debug(f"posting url: {post_https}")
        log.debug(f"posting json: {json_body}")
        r = requests.post(post_https, json=json_body, verify=False)

        return r

    def _get_query_results(self, job_id: str) -> Optional[GeoJsonFeatures]:
        def _download_result(url: str) -> Optional[GeoJsonFeatures]:
            log.info("downloading results..")
            download_result = requests.get(url)

            if download_result.status_code != 200:
                log.error(
                    f"error code {download_result.status_code} with tomtom api get, json: {download_result.json()}"
                )
                return None

            geojson = download_result.json()
            features = geojson.get('features')
            if not features:
                log.error(f"couldn't find features in: {geojson}")
                return None

            # the current TomTom api returns a header as the first feature
            return features[1:]

        timeout_steps = 50
        t = 1

        get_https = self.get_https_base + f"{job_id}?key={self.api_key}"

        while t < timeout_steps:

            # TODO: remove verify=False when going live
            get_response = requests.get(get_https, verify=False)
            if get_response.status_code != 200:
                log.error(
                    f"error code {get_response.status_code} with tomtom api get, json: {get_response.json()}"
                )
                return None

            get_response_json = get_response.json()
            job_state = get_response_json.get('jobState')

            if not job_state:
                log.error(f"couldn't find tomtom job id in {get_response_json}")
                return None

            if job_state == "DONE":
                log.info("tomtom api job finished!")
                urls = get_response_json.get('urls')
                if not urls:
                    log.error(f"couldn't find tomtom download urls in {get_response_json}")
                    return None

                # TODO: confirm the geojson url is always in the first position
                return _download_result(urls[0])
            else:
                sleep_time = t * 10  # seconds
                log.info(f"tomtom api job still in progress: {job_state}, trying again in {sleep_time} seconds")
                time.sleep(sleep_time)
                t += 1

    def update(self, road_network: RoadNetwork) -> int:
        """
        this function pulls speeds from 1 week ago and updates the speeds across the road network.

        #TODO: this is a very costly io operation so we should make this async eventually

        :param road_network:
        :return:
        """
        start_time = time.time()
        try:
            network_type = road_network.G.graph["compass_network_type"]

            # TODO: make this an enum for network types rather than a string. -ndr
            if network_type != "tomtom":
                raise IOError("can only update tomtom road networks")

        except KeyError:
            log.warning("attempting to update speeds on a road network without a type, could result in failure")

        log.info("posting query..")
        post_response = self._post_query(road_network)
        if post_response.status_code != 200:
            log.error(f"error code {post_response.status_code} with tomtom api query, json: {post_response.json()} ")
            return 0

        job_id = post_response.json().get('jobId')
        if not job_id:
            log.error(f"couldn't find tomtom job id in {post_response.json()}")
            return 0

        log.info("getting query..")
        geojson_features = self._get_query_results(job_id)
        if not geojson_features:
            log.error("failed to download updated speeds..")
            return 0

        gdf = gpd.GeoDataFrame.from_features(geojson_features)

        log.info("updating graph edge speeds..")
        for _, _, k, d in road_network.G.edges(data=True, keys=True):
            segment = gdf[gdf["segmentId"] == k]
            if len(segment) < 1:
                log.debug(f"skipping segemnt {k}, couldn't find in speed data")
                continue
            elif len(segment) > 1:
                log.warning(f"found multiple instances of segment {k} in speed data, skipping")
                continue

            new_speed_kph = segment["segmentTimeResults"].values[0][0]["harmonicAverageSpeed"]
            speed_limit_kph = segment["speedLimit"].values[0]

            # default to speed limit
            if new_speed_kph < 1 or new_speed_kph > speed_limit_kph:
                new_speed_kph = speed_limit_kph

            meters = d["meters"]
            if meters == 0:
                log.warning(f"found segment {k} with zero distance, skipping")
                continue

            new_time_hr = (meters * METERS_TO_KILOMETERS) / new_speed_kph

            # TODO: 🚨side effect alert🚨 not sure if we should do this another way? -ndr
            d["kph"] = new_speed_kph
            d["minutes"] = new_time_hr * HOURS_TO_MINUTES

        end_time = time.time()

        road_network.update()

        log.info(f"finished updating graph! took {end_time-start_time} seconds")

        return 1
