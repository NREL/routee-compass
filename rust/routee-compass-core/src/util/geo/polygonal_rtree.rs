use geo::{Area, BooleanOps, BoundingRect, Geometry, Intersects, Polygon};
use rstar::{primitives::Rectangle, RTree, RTreeObject, AABB};
use wkt::ToWkt;

pub struct Node<D> {
    pub geometry: Geometry,
    pub rectangle: Rectangle<(f64, f64)>,
    pub data: D,
}

impl<D> Node<D> {
    pub fn new(geometry: Geometry, data: D) -> Result<Node<D>, String> {
        let rectangle = rect_from_geometry(&geometry)?;
        let result = Node {
            geometry,
            rectangle,
            data,
        };
        Ok(result)
    }
}

impl<D> RTreeObject for Node<D> {
    type Envelope = AABB<(f64, f64)>;

    fn envelope(&self) -> Self::Envelope {
        self.rectangle.envelope()
    }
}

pub struct PolygonalRTree<D>(RTree<Node<D>>);

impl<D> PolygonalRTree<D> {
    pub fn new(data: Vec<(Geometry, D)>) -> Result<PolygonalRTree<D>, String> {
        let nodes = data
            .into_iter()
            .map(|(g, d)| Node::new(g, d))
            .collect::<Result<Vec<_>, _>>()?;
        let tree = RTree::bulk_load(nodes);
        Ok(PolygonalRTree(tree))
    }

    /// tests for intersection with polygonal data in this tree. this involves two steps:
    ///   1. finding rtree rectangle envelopes that intersect the incoming geometry
    ///   2. testing intersection for each discovered geometry bounded by it's rectangle
    pub fn intersection<'a>(
        &'a self,
        g: &'a Geometry,
    ) -> Result<Box<dyn Iterator<Item = &'a Node<D>> + 'a>, String> {
        let query = rect_from_geometry(g)?;
        let iter = self
            .0
            .locate_in_envelope_intersecting(&query.envelope())
            .filter(|node| node.geometry.intersects(g));
        Ok(Box::new(iter))
    }

    pub fn intersection_with_overlap_area<'a>(
        &'a self,
        query: &'a Geometry,
    ) -> Result<Vec<(&'a Node<D>, f64)>, String> {
        // get all polygons in the query geometry
        let query_polygons: Vec<Polygon> = match query {
            Geometry::Polygon(p) => Ok(vec![p.clone()]),
            Geometry::MultiPolygon(mp) => Ok(mp.0.clone()),
            // Geometry::GeometryCollection(geometry_collection) => todo!(),
            _ => Err(String::from(
                "areal proportion query must be performed on polygonal data",
            )),
        }?;

        // compute the overlap area for each query polygon for each geometry
        // found to intersect the query in the rtree
        let result = self
            .intersection(query)?
            .map(|node| {
                let overlap_areas = query_polygons
                    .iter()
                    .map(|p| {
                        let area = overlap_area(p, &node.geometry)?;
                        Ok(area)
                    })
                    .collect::<Result<Vec<f64>, String>>()?;
                let overlap_area: f64 = overlap_areas.into_iter().sum();
                Ok((node, overlap_area))
            })
            .collect::<Result<Vec<(&Node<D>, f64)>, String>>()?;

        Ok(result)
    }
}

/// helper function to create a rectangular rtree envelope for a given geometry
fn rect_from_geometry(g: &Geometry) -> Result<Rectangle<(f64, f64)>, String> {
    let bbox_vec = g.bounding_rect().ok_or_else(|| {
        format!(
            "internal error: cannot get bounds of geometry: '{}'",
            g.to_wkt()
        )
    })?;

    let envelope = Rectangle::from_corners(bbox_vec.min().x_y(), bbox_vec.max().x_y());
    Ok(envelope)
}

/// helper for computing the overlap area, which is the area that two geometries have in common
/// (the intersection).
fn overlap_area(query: &Polygon, overlapping: &Geometry) -> Result<f64, String> {
    match overlapping {
        Geometry::Polygon(overlap_p) => {
            let overlap = query.intersection(overlap_p);
            let overlap_area: f64 = overlap.iter().map(|p| p.unsigned_area()).sum();
            Ok(overlap_area)
        }
        Geometry::MultiPolygon(mp) => {
            let overlaps = mp
                .iter()
                .map(|overlapping_p| {
                    overlap_area(query, &geo::Geometry::Polygon(overlapping_p.clone()))
                })
                .collect::<Result<Vec<_>, String>>()?;
            let overlap_area: f64 = overlaps.into_iter().sum();
            Ok(overlap_area)
        }
        _ => Err(String::from("polygonal rtree node must be polygonal!")),
    }
}
