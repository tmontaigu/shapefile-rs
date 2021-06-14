use geo::prelude::Contains;
fn main() {
    let polygons = shapefile::read_as::<_, shapefile::Polygon, shapefile::dbase::Record>(
        "./tests/data/polygons.shp",
    )
    .expect("Could not open polygon-shapefile");
    let points = shapefile::read_as::<_, shapefile::Point, shapefile::dbase::Record>(
        "./tests/data/points.shp",
    )
    .expect("Could not open point shapefiles");
    for (polygon, polygon_record) in polygons {
        let geo_polygon: geo::MultiPolygon<f64> = polygon.into();
        for (point, point_record) in points.iter() {
            let geo_point: geo::Point<f64> = point.clone().into();
            if geo_polygon.contains(&geo_point) {
                let point_id = match point_record.get("id") {
                    Some(shapefile::dbase::FieldValue::Numeric(Some(x))) => x,
                    Some(_) => panic!("Expected 'id' to be a numeric in point-dataset"),
                    None => panic!("Field 'id' is not within point-dataset"),
                };
                let polygon_id = match polygon_record.get("id") {
                    Some(shapefile::dbase::FieldValue::Numeric(Some(x))) => x,
                    Some(_) => panic!("Expected 'id' to be a numeric in polygon-dataset"),
                    None => panic!("Field 'id' is not within polygon-dataset"),
                };
                println!(
                    "Point with id {} is within polygon with id {}",
                    point_id, polygon_id
                );
            }
        }
    }
}
