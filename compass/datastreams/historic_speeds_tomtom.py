import json
import logging
import time
from datetime import datetime, timedelta
from typing import Optional, Callable

import geopandas as gpd
import requests

from compass.datastreams.base import DataStream
from compass.road_network.constructs.link import Link
from compass.utils.geo_utils import GeoJsonFeatures, BoundingBox
from compass.utils.units import *

log = logging.getLogger(__name__)


class HistoricSpeedsTomTomStream(DataStream):
    """
    pings the TomTom traffic stats api and updates the road network with new speeds every hour.
    """
    _observers = []

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

    def __init__(self, timezone_str: str, bounding_box: BoundingBox):
        json_body = json.loads(self.json_skeleton)

        json_body["network"]["timeZoneId"] = timezone_str
        json_body["network"]["boundingBox"] = bounding_box.as_tomtom_json()
        json_body["network"]["name"] = bounding_box.bbox_id

        self.json_body = json_body

        self.bounding_box = bounding_box

        self.links = []

    def bind_to(self, callback: Callable):
        self._observers.append(callback)

    def _notify_observers(self):
        for callback in self._observers:
            callback(tuple(self.links))

    def _post_query(self) -> requests.Response:
        last_week = datetime.now() - timedelta(days=7)

        json_body = self.json_body

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

        timeout_steps = 25  # about 50 minutes
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

    def collect(self) -> int:
        """
        this function pulls speeds from 1 week ago and updates the speeds across the road network.

        #TODO: this is a very costly io operation so we should make this async eventually

        :return:
        """
        start_time = time.time()

        # reset links data store
        self.links = []

        log.info("posting query..")
        post_response = self._post_query()
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

        for t in gdf.itertuples():
            link_id = t.segmentId

            new_speed_kph = t.segmentTimeResults[0]["harmonicAverageSpeed"]
            speed_limit_kph = t.speedLimit

            # default to speed limit
            if new_speed_kph < 1 or new_speed_kph > speed_limit_kph:
                new_speed_kph = speed_limit_kph

            meters = t.distance
            new_time_hr = (meters * METERS_TO_KILOMETERS) / new_speed_kph

            attributes = {
                'kph': new_speed_kph,
                'minutes': new_time_hr * HOURS_TO_MINUTES,
            }

            self.links.append(Link(
                link_id=link_id,
                attributes=attributes,
            ))

        end_time = time.time()

        log.info(f"finished collecting updated links! took {end_time - start_time} seconds")

        self._notify_observers()

        return 1
