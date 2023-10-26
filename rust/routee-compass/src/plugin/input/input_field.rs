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
    pub fn to_string(&self) -> String {
        self.to_str().to_string()
    }
}
