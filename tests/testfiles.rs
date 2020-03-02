#![allow(dead_code)]
extern crate shapefile;

use shapefile::record::{MultipartShape, MultipointShape};
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
pub const POLYGONM_PATH: &str = "./tests/data/polygonm.shp";
pub const POLYGONZ_PATH: &str = "./tests/data/polygonz.shp";

pub const MULTIPOINT_PATH: &str = "./tests/data/multipoint.shp";
pub const MULTIPOINTZ_PATH: &str = "./tests/data/multipointz.shp";

pub const MULTIPATCH_PATH: &str = "./tests/data/multipatch.shp";

pub fn check_line_first_shape(shape: &shapefile::Shape) {
    if let shapefile::Shape::Polyline(shp) = shape {
        assert_eq!(shp.bbox.xmin, 1.0);
        assert_eq!(shp.bbox.ymin, 1.0);
        assert_eq!(shp.bbox.xmax, 5.0);
        assert_eq!(shp.bbox.ymax, 6.0);
        assert_eq!(shp.parts_indices(), vec![0, 5].as_slice());
        let expected_point = vec![
            Point { x: 1.0, y: 5.0 },
            Point { x: 5.0, y: 5.0 },
            Point { x: 5.0, y: 1.0 },
            Point { x: 3.0, y: 3.0 },
            Point { x: 1.0, y: 1.0 },
            Point { x: 3.0, y: 2.0 },
            Point { x: 2.0, y: 6.0 },
        ];
        assert_eq!(shp.points(), expected_point.as_slice());
    } else {
        assert!(false, "The shape is not a Polyline");
    }
}
