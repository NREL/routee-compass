use crate::model::road_network::vertex_id::VertexId;
use geo::{coord, Coord};
use serde::de;

/// represents a vertex in a Graph
/// this struct implements Serialize and Deserialize to support reading
/// vertex records from CSV files.
#[derive(Copy, Clone, Default, Debug)]
pub struct Vertex {
    pub vertex_id: VertexId,
    pub coordinate: Coord,
}

impl Vertex {
    pub fn new(vertex_id: usize, x: f64, y: f64) -> Self {
        Self {
            vertex_id: VertexId(vertex_id),
            coordinate: coord! {x: x, y: y},
        }
    }
    pub fn to_tuple_underlying(&self) -> (f64, f64) {
        (self.coordinate.x, self.coordinate.y)
    }

    pub fn x(&self) -> f64 {
        self.coordinate.x
    }

    pub fn y(&self) -> f64 {
        self.coordinate.y
    }
}

const VERTEX_ID: &str = "vertex_id";
const X_COORDINATE: &str = "x";
const Y_COORDINATE: &str = "y";

impl<'de> de::Deserialize<'de> for Vertex {
    /// specialized deserialization for `Vertex` that creates a Vertex from a CSV
    /// that has vertex_id, x, and y columns.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct VertexVisitor;

        impl<'de> de::Visitor<'de> for VertexVisitor {
            type Value = Vertex;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a vertex_id, x, and y field")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                // written to allow fields to appear in arbitrary order. we store each expected, deserialized value
                // as we find them, ignoring unexpected fields. if we find all three, we build a Vertex and end the loop.
                let mut vertex_id_result: Option<usize> = None;
                let mut x_result: Option<f64> = None;
                let mut y_result: Option<f64> = None;
                let mut vertex_result: Option<Vertex> = None;
                let mut next: Option<(&str, &str)> = map.next_entry()?;
                while next.is_some() {
                    // match on expected key names
                    if let Some((key, value)) = next {
                        match key {
                            VERTEX_ID => {
                                let id: usize = value.parse().map_err(|e| {
                                    de::Error::custom(format!(
                                        "unable to parse vertex_id '{}': {}",
                                        &value, e
                                    ))
                                })?;
                                vertex_id_result = Some(id);
                            }
                            X_COORDINATE => {
                                let x_coord: f64 = value.parse().map_err(|e| {
                                    de::Error::custom(format!(
                                        "unable to parse x '{}': {}",
                                        &value, e
                                    ))
                                })?;
                                x_result = Some(x_coord);
                            }
                            Y_COORDINATE => {
                                let y_coord: f64 = value.parse().map_err(|e| {
                                    de::Error::custom(format!(
                                        "unable to parse y '{}': {}",
                                        &value, e
                                    ))
                                })?;
                                y_result = Some(y_coord);
                            }
                            &_ => {} // ignore unknown key/value pairs
                        }
                    } else {
                        return Err(de::Error::custom("internal error"));
                    }
                    match vertex_id_result.zip(x_result).zip(y_result) {
                        Some(((vertex_id, x), y)) => {
                            // we're done; build the vertex and short-circuit the loop
                            vertex_result = Some(Vertex::new(vertex_id, x, y));
                            next = None;
                        }
                        None => {
                            // pull another k:v pair from the map
                            next = map.next_entry()?;
                        }
                    }
                }

                match vertex_result {
                    None => Err(de::Error::custom("failed to deserialize Vertex")),
                    Some(vertex) => Ok(vertex),
                }
            }
        }

        deserializer.deserialize_map(VertexVisitor {})
    }
}

#[cfg(test)]
mod tests {

    use std::path::PathBuf;

    use csv;
    use geo::Coord;

    use crate::model::{property::vertex::Vertex, road_network::vertex_id::VertexId};

    #[test]
    fn test_deserialize_csv() {
        let filepath = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("model")
            .join("property")
            .join("test")
            .join("vertices.csv");

        let expected: Vec<Vertex> = vec![
            Vertex {
                vertex_id: VertexId(5),
                coordinate: Coord {
                    x: -105.2042387,
                    y: 39.712214,
                },
            },
            Vertex {
                vertex_id: VertexId(10),
                coordinate: Coord {
                    x: -105.2292743,
                    y: 39.7584272,
                },
            },
            Vertex {
                vertex_id: VertexId(15),
                coordinate: Coord {
                    x: -105.1972541,
                    y: 39.760731,
                },
            },
        ];

        let mut reader = csv::Reader::from_path(filepath).unwrap();
        let mut result: Vec<Vertex> = vec![];
        for row in reader.deserialize() {
            let vertex: Vertex = row.unwrap();
            result.push(vertex);
        }

        for idx in [0, 1, 2] {
            assert_eq!(
                result[idx].vertex_id, expected[idx].vertex_id,
                "vertex ids didn't match expected"
            );
            assert_eq!(
                result[idx].coordinate, expected[idx].coordinate,
                "coordinate didn't match expected"
            );
        }
    }
}
