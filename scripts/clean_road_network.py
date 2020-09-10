import networkx as nx
import osmnx as ox
import logging as log

ox.utils.config(all_oneway=True)

log.basicConfig(level=log.INFO)

DEFAULT_MPH = 30
_unit_conversion = {
    'mph': 1,
    'kmph': 0.621371,
}
METERS_TO_MILES = 0.0006213712


def parse_osm_speed(osm_speed):
    def _parse_speed_string(speed_string):
        if not any(char.isdigit() for char in speed_string):
            # no numbers in string, set as defualt
            return DEFAULT_MPH
        else:
            # try to parse the string assuming the format '{speed} {units}'
            try:
                speed = float(speed_string.split(' ')[0])
            except ValueError:
                log.warning(f"attempted to parse speed {speed_string} but was unable to convert to a number. "
                            f"setting as default speed of {DEFAULT_MPH} mph")
                return DEFAULT_MPH

            try:
                units = speed_string.split(' ')[1]
                unit_conversion = _unit_conversion[units]
            except IndexError:
                log.warning(f"attempted to parse speed {speed_string} but was unable to discern units. "
                            f"setting as default speed of {DEFAULT_MPH} mph")
                return DEFAULT_MPH

            return speed * unit_conversion

    # capture any strings that should be lists
    if '[' in osm_speed:
        osm_speed = eval(osm_speed)

    if isinstance(osm_speed, list):
        # if the speed is a list, we'll take the first speed.
        if isinstance(osm_speed[0], str):
            speed_kmph = _parse_speed_string(osm_speed[0])
        else:
            # the first element is not a string, set to default since we don't know how to handle
            speed_kmph = DEFAULT_MPH
    elif isinstance(osm_speed, str):
        speed_kmph = _parse_speed_string(osm_speed)
    else:
        # if the speed neither a list nor a string (i.e. None), we set as default
        speed_kmph = DEFAULT_MPH

    return speed_kmph


def parse_road_network_graph(g):
    osm_speed = nx.get_edge_attributes(g, 'maxspeed')
    parsed_speed = {k: parse_osm_speed(v) for k, v in osm_speed.items()}
    nx.set_edge_attributes(g, parsed_speed, 'gpsspeed')

    length_meters = nx.get_edge_attributes(g, 'length')
    length_miles = {k: v * METERS_TO_MILES for k, v in length_meters.items()}
    nx.set_edge_attributes(g, length_miles, 'miles')

    # TODO add real grade here -ndr
    nx.set_edge_attributes(g, name="grade", values=0)

    return g


if __name__ == "__main__":
    log.info("pulling raw osm network..")
    # this only grabs denver county.. probably want to get denver metro
    G = ox.graph_from_place("Denver, CO", network_type="drive")

    log.info("computing largest strongly connected component..")
    # this makes sure there are no graph 'dead-ends'
    G = ox.utils_graph.get_largest_component(G, strongly=True)

    log.info("parsing speeds and computing travel times..")
    G = ox.add_edge_speeds(G)
    G = ox.add_edge_travel_times(G)
    G = parse_road_network_graph(G)

    log.info("writing to file..")
    nx.write_gpickle(G, "../resources/denver_roadnetwork.pickle")
