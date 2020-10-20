from typing import NamedTuple

import numpy as np

NodeId = np.int64
LinkId = np.int64


class Link(NamedTuple):
    link_id: LinkId

    # TODO: make attributes explicit
    attributes: dict
