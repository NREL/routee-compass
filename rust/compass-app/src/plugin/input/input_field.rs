pub enum InputField {
    OriginX,
    OriginY,
    DestinationX,
    DestinationY,
    OriginVertex,
    DestinationVertex,
    GridSearch,
}

impl InputField {
    pub fn to_str(&self) -> &'static str {
        match self {
            InputField::OriginX => "origin_x",
            InputField::OriginY => "origin_y",
            InputField::DestinationX => "destination_x",
            InputField::DestinationY => "destination_y",
            InputField::OriginVertex => "origin_vertex",
            InputField::DestinationVertex => "destination_vertex",
            InputField::GridSearch => "grid_search",
        }
    }
    pub fn to_string(&self) -> String {
        self.to_str().to_string()
    }
}
