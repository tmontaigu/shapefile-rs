#![allow(dead_code)]
extern crate shapefile;

use shapefile::Point;

pub const LINE_PATH: &str = "./tests/data/line.shp";
pub const LINE_SHX_PATH: &str = "./tests/data/line.shx";
pub const LINEM_PATH: &str = "./tests/data/linem.shp";
pub const LINEZ_PATH: &str = "./tests/data/linez.shp";

pub const POINT_PATH: &str = "./tests/data/point.shp";
pub const POINT_SHX_PATH: &str = "./tests/data/point.shx";
pub const POINTM_PATH: &str = "./tests/data/pointm.shp";
pub const POINTZ_PATH: &str = "./tests/data/pointz.shp";

pub const POLYGON_PATH: &str = "./tests/data/polygon.shp";
pub const POLYGON_HOLE_PATH: &str = "./tests/data/polygon_hole.shp";
pub const POLYGON_HOLE_SHX_PATH: &str = "./tests/data/polygon_hole.shx";
pub const POLYGONM_PATH: &str = "./tests/data/polygonm.shp";
pub const POLYGONZ_PATH: &str = "./tests/data/polygonz.shp";

pub const MULTIPOINT_PATH: &str = "./tests/data/multipoint.shp";
pub const MULTIPOINTZ_PATH: &str = "./tests/data/multipointz.shp";

pub const MULTIPATCH_PATH: &str = "./tests/data/multipatch.shp";

pub fn check_line_first_shape(shape: &shapefile::Shape) {
    if let shapefile::Shape::Polyline(shp) = shape {
        assert_eq!(shp.bbox().min.x, 1.0);
        assert_eq!(shp.bbox().min.y, 1.0);
        assert_eq!(shp.bbox().max.x, 5.0);
        assert_eq!(shp.bbox().max.y, 6.0);
        let first_part = vec![
            Point { x: 1.0, y: 5.0 },
            Point { x: 5.0, y: 5.0 },
            Point { x: 5.0, y: 1.0 },
            Point { x: 3.0, y: 3.0 },
            Point { x: 1.0, y: 1.0 },
        ];
        let second_part = vec![
            Point { x: 3.0, y: 2.0 },
            Point { x: 2.0, y: 6.0 },
        ];
        assert_eq!(shp.parts()[0], first_part.as_slice());
        assert_eq!(shp.parts()[1], second_part.as_slice());
    } else {
        assert!(false, "The shape is not a Polyline");
    }
}
