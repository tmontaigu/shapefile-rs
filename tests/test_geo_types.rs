#[cfg(feature = "geo-types")]
mod geo_types_conversions {
    extern crate geo_types;
    use std::convert::TryFrom;

    #[test]
    fn test_polygon_conversion() {
        let shp_polygons =
            shapefile::read_as::<_, shapefile::Polygon>("tests/data/multi_polygon.shp").unwrap();

        let mut multi_polygons = shp_polygons
            .into_iter()
            .map(|polygon| geo_types::MultiPolygon::<f64>::try_from(polygon).unwrap())
            .collect::<Vec<geo_types::MultiPolygon<f64>>>();

        let multi_polygon = multi_polygons.pop().unwrap();
        let geo_polygons = multi_polygon
            .into_iter()
            .collect::<Vec<geo_types::Polygon<f64>>>();
        assert_eq!(geo_polygons.len(), 4);
        for gp in &geo_polygons {
            assert_eq!(gp.interiors().len(), 0);
        }

        let multi_polygon = geo_types::MultiPolygon::from(geo_polygons);
        let polygon = shapefile::Polygon::from(multi_polygon);

        let shp_polygons =
            shapefile::read_as::<_, shapefile::Polygon>("tests/data/multi_polygon.shp").unwrap();

        assert_eq!(&polygon.points, &shp_polygons[0].points);
        assert_eq!(&polygon.parts, &shp_polygons[0].parts);
    }
}
