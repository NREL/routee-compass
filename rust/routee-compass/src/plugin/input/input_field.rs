use std::fmt::Display;

pub enum InputField {
    OriginX,
    OriginY,
    DestinationX,
    DestinationY,
    OriginVertex,
    DestinationVertex,
    OriginEdge,
    DestinationEdge,
    GridSearch,
}

impl InputField {
    pub fn to_str(&self) -> &'static str {
        use InputField as I;
        match self {
            I::OriginX => "origin_x",
            I::OriginY => "origin_y",
            I::DestinationX => "destination_x",
            I::DestinationY => "destination_y",
            I::OriginVertex => "origin_vertex",
            I::DestinationVertex => "destination_vertex",
            I::OriginEdge => "origin_edge",
            I::DestinationEdge => "destination_edge",
            I::GridSearch => "grid_search",
        }
    }
}

impl Display for InputField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}
