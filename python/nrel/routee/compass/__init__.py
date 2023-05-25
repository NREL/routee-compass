from pathlib import Path

from nrel.routee.compass.compass_rust import (
    Link,
    Node,
    Graph,
    RustMap,
    SearchInput,
    SearchResult,
    SearchType,
    TimeOfDaySpeeds,
    VehicleParameters,
    extract_largest_scc,
)


def root() -> Path:
    return Path(__file__).parent.parent
