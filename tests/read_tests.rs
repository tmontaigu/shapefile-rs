extern crate shapefile;

use std::io::Cursor;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;

mod testfiles;

use shapefile::{Multipatch, Patch, Point, PointM, PointZ, PolygonRing};
use shapefile::{Multipoint, MultipointZ};
use shapefile::{Polygon, PolygonM, PolygonZ};
use shapefile::{Polyline, PolylineM, PolylineZ};

fn check_line<T: Read>(reader: shapefile::Reader<T>) {
    {
        let header = reader.header();
        assert_eq!(header.file_length, 136);
        assert_eq!(header.shape_type, shapefile::ShapeType::Polyline);
        assert_eq!(header.bbox.min, PointZ::new(1.0, 1.0, 0.0, 0.0));
        assert_eq!(header.bbox.max, PointZ::new(5.0, 6.0, 0.0, 0.0));
    }

    let shapes = reader.read().unwrap();

    assert_eq!(shapes.len(), 1);
    testfiles::check_line_first_shape(&shapes[0]);
}

fn check_linem<T: Read>(reader: shapefile::Reader<T>) {
    use shapefile::NO_DATA;
    {
        let header = reader.header();
        assert_eq!(header.file_length, 172);
        assert_eq!(header.shape_type, shapefile::ShapeType::PolylineM);
        assert_eq!(header.bbox.min, PointZ::new(1.0, 1.0, 0.0, 0.0));
        assert_eq!(header.bbox.max, PointZ::new(5.0, 6.0, 0.0, 3.0));
    }

    let shapes = reader.read().unwrap();
    assert_eq!(shapes.len(), 1);

    if let shapefile::Shape::PolylineM(shape) = &shapes[0] {
        assert_eq!(shape.bbox().min.x, 1.0);
        assert_eq!(shape.bbox().min.y, 1.0);
        assert_eq!(shape.bbox().max.x, 5.0);
        assert_eq!(shape.bbox().max.y, 6.0);
        let first_part = vec![
            PointM {
                x: 1.0,
                y: 5.0,
                m: 0.0,
            },
            PointM {
                x: 5.0,
                y: 5.0,
                m: NO_DATA,
            },
            PointM {
                x: 5.0,
                y: 1.0,
                m: 3.0,
            },
            PointM {
                x: 3.0,
                y: 3.0,
                m: NO_DATA,
            },
            PointM {
                x: 1.0,
                y: 1.0,
                m: 0.0,
            },
        ];
        let second_part = vec![
            PointM {
                x: 3.0,
                y: 2.0,
                m: NO_DATA,
            },
            PointM {
                x: 2.0,
                y: 6.0,
                m: NO_DATA,
            },
        ];
        assert_eq!(shape.parts()[0], first_part.as_slice());
        assert_eq!(shape.parts()[1], second_part.as_slice());
    } else {
        assert!(false, "The shape is not a PolylineM");
    }
}

fn check_linez<T: Read>(reader: shapefile::Reader<T>) {
    use shapefile::NO_DATA;
    {
        let header = reader.header();
        assert_eq!(header.file_length, 258);
        assert_eq!(header.shape_type, shapefile::ShapeType::PolylineZ);
        assert_eq!(header.bbox.min, PointZ::new(1.0, 1.0, 0.0, 0.0));
        assert_eq!(header.bbox.max, PointZ::new(5.0, 9.0, 22.0, 3.0));
    }

    let shapes = reader.read().unwrap();
    assert_eq!(shapes.len(), 1);

    for shape in shapes {
        if let shapefile::Shape::PolylineZ(shp) = shape {
            let first_part = vec![
                PointZ {
                    x: 1.0,
                    y: 5.0,
                    z: 18.0,
                    m: NO_DATA,
                },
                PointZ {
                    x: 5.0,
                    y: 5.0,
                    z: 20.0,
                    m: NO_DATA,
                },
                PointZ {
                    x: 5.0,
                    y: 1.0,
                    z: 22.0,
                    m: NO_DATA,
                },
                PointZ {
                    x: 3.0,
                    y: 3.0,
                    z: 0.0,
                    m: NO_DATA,
                },
                PointZ {
                    x: 1.0,
                    y: 1.0,
                    z: 0.0,
                    m: NO_DATA,
                },
            ];
            let second_part = vec![
                PointZ {
                    x: 3.0,
                    y: 2.0,
                    z: 0.0,
                    m: NO_DATA,
                },
                PointZ {
                    x: 2.0,
                    y: 6.0,
                    z: 0.0,
                    m: NO_DATA,
                },
            ];
            let third_part = vec![
                PointZ {
                    x: 3.0,
                    y: 2.0,
                    z: 15.0,
                    m: 0.0,
                },
                PointZ {
                    x: 2.0,
                    y: 6.0,
                    z: 13.0,
                    m: 3.0,
                },
                PointZ {
                    x: 1.0,
                    y: 9.0,
                    z: 14.0,
                    m: 2.0,
                },
            ];
            assert_eq!(shp.parts()[0], first_part.as_slice());
            assert_eq!(shp.parts()[1], second_part.as_slice());
            assert_eq!(shp.parts()[2], third_part.as_slice());
        //assert_eq!(shp.z_range, [0.0, 22.0]);
        //assert_eq!(shp.m_range, [0.0, 3.0]);
        } else {
            assert!(false, "The shape is not a PolylineZ");
        }
    }
}

fn check_first_point(point: &shapefile::Point) {
    assert_eq!(point.x, 122.0);
    assert_eq!(point.y, 37.0);
}

fn check_point<T: Read>(reader: shapefile::Reader<T>) {
    {
        let header = reader.header();
        assert_eq!(header.file_length, 64);
        assert_eq!(header.shape_type, shapefile::ShapeType::Point);
        assert_eq!(header.bbox.min, PointZ::new(122.0, 37.0, 0.0, 0.0));
        assert_eq!(header.bbox.max, PointZ::new(122.0, 37.0, 0.0, 0.0));
    }
    let shapes = reader.read().unwrap();
    assert_eq!(shapes.len(), 1, "Wrong number of shapes");

    let points = shapefile::record::convert_shapes_to_vec_of::<Point>(shapes).unwrap();
    assert_eq!(points.len(), 1, "Wrong number of points");

    check_first_point(&points[0]);
}

fn _check_first_point_m(point: &shapefile::PointM) {
    assert_eq!(point.x, 160477.9000324604);
    assert_eq!(point.y, 5403959.561417906);
    assert_eq!(point.m, 0.0);
}

pub fn check_first_point_m(shape: &shapefile::Shape) {
    if let shapefile::Shape::PointM(shp) = shape {
        _check_first_point_m(shp);
    } else {
        assert!(false, "The first shape is not a PointZ");
    }
}

fn _check_second_point_m(point: &shapefile::PointM) {
    assert_eq!(point.x, 160467.63787299366);
    assert_eq!(point.y, 5403971.985031904);
    assert_eq!(point.m, 0.0);
}

pub fn check_second_point_m(shape: &shapefile::Shape) {
    if let shapefile::Shape::PointM(shp) = shape {
        _check_second_point_m(shp);
    } else {
        assert!(false, "The second shape is not a PointZ");
    }
}

fn check_pointm<T: Read>(reader: shapefile::Reader<T>) {
    {
        let header = reader.header();
        assert_eq!(header.file_length, 86);
        assert_eq!(header.shape_type, shapefile::ShapeType::PointM);
        assert_eq!(
            header.bbox.min,
            PointZ::new(160467.63787299366, 5403959.561417906, 0.0, 0.0)
        );
        assert_eq!(
            header.bbox.max,
            PointZ::new(160477.9000324604, 5403971.985031904, 0.0, 0.0)
        );
    }
    let shapes = reader.read().unwrap();
    assert_eq!(shapes.len(), 2, "Wrong number of shapes");

    check_first_point_m(&shapes[0]);
    check_second_point_m(&shapes[1]);
}

fn _check_first_point_z(point: &shapefile::PointZ) {
    assert_eq!(point.x, 1422464.3681007193);
    assert_eq!(point.y, 4188962.3364355816);
    assert_eq!(point.z, 72.40956470558095);
    assert_eq!(point.m, shapefile::NO_DATA);
}

fn _check_second_point_z(point: &shapefile::PointZ) {
    assert_eq!(point.x, 1422459.0908050265);
    assert_eq!(point.y, 4188942.211755641);
    assert_eq!(point.z, 72.58286959604922);
    assert_eq!(point.m, shapefile::NO_DATA);
}

fn check_pointz<T: Read>(reader: shapefile::Reader<T>) {
    {
        let header = reader.header();
        assert_eq!(header.file_length, 94);
        assert_eq!(header.shape_type, shapefile::ShapeType::PointZ);
        assert_eq!(
            header.bbox.min,
            PointZ::new(
                1422459.0908050265,
                4188942.211755641,
                72.40956470558095,
                0.0
            )
        );
        assert_eq!(
            header.bbox.max,
            PointZ::new(
                1422464.3681007193,
                4188962.3364355816,
                72.58286959604922,
                0.0
            )
        );
    }
    let shapes = reader.read().unwrap();
    assert_eq!(shapes.len(), 2, "Wrong number of shapes");

    if let shapefile::Shape::PointZ(shp) = &shapes[0] {
        _check_first_point_z(shp);
    } else {
        assert!(false, "The first shape is not a PointZ");
    }

    if let shapefile::Shape::PointZ(shp) = &shapes[1] {
        _check_second_point_z(shp);
    } else {
        assert!(false, "The second shape is not a PointZ");
    }
}

fn check_polygon<T: Read>(reader: shapefile::Reader<T>) {
    {
        let header = reader.header();
        assert_eq!(header.file_length, 170);
        assert_eq!(header.shape_type, shapefile::ShapeType::Polygon);
        assert_eq!(header.bbox.min, PointZ::new(15.0, 2.0, 0.0, 0.0));
        assert_eq!(header.bbox.max, PointZ::new(122.0, 37.0, 0.0, 0.0));
    }
    let shapes = reader.read().unwrap();
    assert_eq!(shapes.len(), 1, "Wrong number of shapes");

    if let shapefile::Shape::Polygon(shp) = &shapes[0] {
        let first_part = PolygonRing::Inner(vec![
            Point { x: 122.0, y: 37.0 },
            Point { x: 117.0, y: 36.0 },
            Point { x: 115.0, y: 32.0 },
            Point { x: 118.0, y: 20.0 },
            Point { x: 113.0, y: 24.0 },
        ]);
        let second_part = PolygonRing::Outer(vec![
            Point { x: 15.0, y: 2.0 },
            Point { x: 17.0, y: 6.0 },
            Point { x: 22.0, y: 7.0 },
        ]);
        let third_part = PolygonRing::Inner(vec![
            Point { x: 122.0, y: 37.0 },
            Point { x: 117.0, y: 36.0 },
            Point { x: 115.0, y: 32.0 },
        ]);
        assert_eq!(shp.ring(0), Some(&first_part));
        assert_eq!(shp.ring(1), Some(&second_part));
        assert_eq!(shp.ring(2), Some(&third_part));
        assert_eq!(shp.rings().len(), 3);
    } else {
        assert!(false, "The second shape is not a Polygon");
    }
}

fn check_polygonm<T: Read>(reader: shapefile::Reader<T>) {
    {
        let header = reader.header();
        assert_eq!(header.file_length, 134);
        assert_eq!(header.shape_type, shapefile::ShapeType::PolygonM);
        assert_eq!(
            header.bbox.min,
            PointZ::new(159374.30785312195, 5403473.287488617, 0.0, 0.0)
        );
        assert_eq!(
            header.bbox.max,
            PointZ::new(160420.36722814097, 5404314.139043656, 0.0, 0.0)
        );
    }
    let shapes = reader.read().unwrap();
    assert_eq!(shapes.len(), 1, "Wrong number of shapes");

    if let shapefile::Shape::PolygonM(shp) = &shapes[0] {
        let first_ring = PolygonRing::Outer(vec![
            PointM {
                x: 159814.75390576152,
                y: 5404314.139043656,
                m: 0.0,
            },
            PointM {
                x: 160420.36722814097,
                y: 5403703.520652497,
                m: 0.0,
            },
            PointM {
                x: 159374.30785312195,
                y: 5403473.287488617,
                m: 0.0,
            },
            PointM {
                x: 159814.75390576152,
                y: 5404314.139043656,
                m: 0.0,
            },
        ]);
        assert_eq!(shp.ring(0), Some(&first_ring));
        assert_eq!(shp.rings().len(), 1);
    } else {
        assert!(false, "The second shape is not a PolygonZ");
    }
}

fn check_polygonz<T: Read>(reader: shapefile::Reader<T>) {
    {
        let header = reader.header();
        assert_eq!(header.file_length, 1262);
        assert_eq!(header.shape_type, shapefile::ShapeType::PolygonZ);
        //FIXME input test file is wrong
        //assert_eq!(header.point_min, [1422691.1637959871, 4188837.293869424, 0.0]);
        //assert_eq!(header.point_max, [1422692.1644789441, 4188838.2945523816, 0.0]);
        //assert_eq!(header.m_range, [0.0, 0.0]);
    }
    let shapes = reader.read().unwrap();
    assert_eq!(shapes.len(), 1, "Wrong number of shapes");

    //FIXME find a file with less values
    if let shapefile::Shape::PolygonZ(shp) = &shapes[0] {
        assert_eq!(shp.rings().len(), 1);
    } else {
        assert!(false, "The second shape is not a PolygonZ");
    }
}

fn check_multipoint<T: Read>(reader: shapefile::Reader<T>) {
    {
        let header = reader.header();
        assert_eq!(header.file_length, 90);
        assert_eq!(header.shape_type, shapefile::ShapeType::Multipoint);
        assert_eq!(header.bbox.min, PointZ::new(122.0, 32., 0.0, 0.0));
        assert_eq!(header.bbox.max, PointZ::new(124.0, 37.0, 0.0, 0.0));
    }
    let shapes = reader.read().unwrap();
    assert_eq!(shapes.len(), 1, "Wrong number of shapes");

    if let shapefile::Shape::Multipoint(shp) = &shapes[0] {
        let expected_points = vec![Point { x: 122.0, y: 37.0 }, Point { x: 124.0, y: 32.0 }];
        assert_eq!(shp.points(), expected_points.as_slice());
    } else {
        assert!(false, "Shape is not a Multipoint");
    }
}

fn check_multipointz<T: Read>(reader: shapefile::Reader<T>) {
    {
        let header = reader.header();
        assert_eq!(header.file_length, 154);
        assert_eq!(header.shape_type, shapefile::ShapeType::MultipointZ);
        assert_eq!(
            header.bbox.min,
            PointZ::new(
                1422671.7232666016,
                4188903.4295959473,
                71.99445343017578,
                -1e38
            )
        );
        assert_eq!(
            header.bbox.max,
            PointZ::new(
                1422672.1022949219,
                4188903.7578430176,
                72.00995635986328,
                -1e38
            )
        );
    }
    let shapes = reader.read().unwrap();
    assert_eq!(shapes.len(), 1, "Wrong number of shapes");

    if let shapefile::Shape::MultipointZ(shp) = &shapes[0] {
        let expected_points = vec![
            PointZ {
                x: 1422671.7232666016,
                y: 4188903.4295959473,
                z: 72.00995635986328,
                m: -1e38,
            },
            PointZ {
                x: 1422672.1022949219,
                y: 4188903.4295959473,
                z: 72.0060806274414,
                m: -1e38,
            },
            PointZ {
                x: 1422671.9127807617,
                y: 4188903.7578430176,
                z: 72.00220489501953,
                m: -1e38,
            },
            PointZ {
                x: 1422671.9127807617,
                y: 4188903.539001465,
                z: 71.99445343017578,
                m: -1e38,
            },
        ];
        assert_eq!(shp.points(), expected_points.as_slice());
    } else {
        assert!(false, "Shape is not a Multipoint");
    }
}

fn check_multipatch<T: Read>(reader: shapefile::Reader<T>) {
    {
        let header = reader.header();
        assert_eq!(header.file_length, 356, "Wrong file length");
        assert_eq!(header.shape_type, shapefile::ShapeType::Multipatch);
        assert_eq!(header.bbox.min, PointZ::new(0.0, 0.0, 0.0, -10e38));
        // FIXME max z in header of original file is wrong
        // assert_eq!(header.bbox.max, PointZ::new(5.0, 5.0, 5.0, -10e38));
    }
    use shapefile::NO_DATA;
    let shapes = reader.read().unwrap();
    assert_eq!(shapes.len(), 1, "Wrong number of shapes");

    if let shapefile::Shape::Multipatch(shp) = &shapes[0] {
        let first_patch = Patch::TriangleStrip(vec![
            PointZ {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                m: NO_DATA,
            },
            PointZ {
                x: 0.0,
                y: 0.0,
                z: 3.0,
                m: NO_DATA,
            },
            PointZ {
                x: 5.0,
                y: 0.0,
                z: 0.0,
                m: NO_DATA,
            },
            PointZ {
                x: 5.0,
                y: 0.0,
                z: 3.0,
                m: NO_DATA,
            },
            PointZ {
                x: 5.0,
                y: 5.0,
                z: 0.0,
                m: NO_DATA,
            },
            PointZ {
                x: 5.0,
                y: 5.0,
                z: 3.0,
                m: NO_DATA,
            },
            PointZ {
                x: 0.0,
                y: 5.0,
                z: 0.0,
                m: NO_DATA,
            },
            PointZ {
                x: 0.0,
                y: 5.0,
                z: 3.0,
                m: NO_DATA,
            },
            PointZ {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                m: NO_DATA,
            },
            PointZ {
                x: 0.0,
                y: 0.0,
                z: 3.0,
                m: NO_DATA,
            },
        ]);
        let second_patch = Patch::TriangleFan(vec![
            PointZ {
                x: 2.5,
                y: 2.5,
                z: 5.0,
                m: NO_DATA,
            },
            PointZ {
                x: 0.0,
                y: 0.0,
                z: 3.0,
                m: NO_DATA,
            },
            PointZ {
                x: 5.0,
                y: 0.0,
                z: 3.0,
                m: NO_DATA,
            },
            PointZ {
                x: 5.0,
                y: 5.0,
                z: 3.0,
                m: NO_DATA,
            },
            PointZ {
                x: 0.0,
                y: 5.0,
                z: 3.0,
                m: NO_DATA,
            },
            PointZ {
                x: 0.0,
                y: 0.0,
                z: 3.0,
                m: NO_DATA,
            },
        ]);
        assert_eq!(shp.patch(0), Some(&first_patch));
        assert_eq!(shp.patch(1), Some(&second_patch));
    } else {
        assert!(false, "Shape is not a Multipatch");
    }
}

macro_rules! read_test {
    ($func:ident, $check_func:ident, $src_file:expr) => {
        #[test]
        fn $func() {
            let reader = shapefile::Reader::from_path($src_file).unwrap();
            $check_func(reader);
        }
    };
}

macro_rules! read_write_read_test {
    ($func:ident, $concrete_type:ident, $check_func:ident, $src_file:expr) => {
        #[test]
        fn $func() {
            let reader = shapefile::Reader::from_path($src_file).unwrap();
            let shapes = reader.read().unwrap();
            let shapes =
                shapefile::record::convert_shapes_to_vec_of::<$concrete_type>(shapes).unwrap();

            let v = Vec::<u8>::new();
            let mut cursor = Cursor::new(v);
            let mut writer = shapefile::writer::Writer::new(cursor);
            writer.write_shapes(&shapes).unwrap();

            cursor = writer.dest;

            let pos_at_end = cursor.seek(SeekFrom::Current(0)).unwrap();
            cursor.seek(SeekFrom::Start(0)).unwrap();
            let hdr = shapefile::header::Header::read_from(&mut cursor).unwrap();
            assert_eq!(
                (hdr.file_length * 2) as u64,
                pos_at_end,
                "Not at expected pos"
            );

            cursor.seek(SeekFrom::Start(0)).unwrap();
            let reader = shapefile::Reader::new(cursor).unwrap();
            $check_func(reader);
        }
    };
}

/* Read tests on Polylines */
read_test!(read_line, check_line, testfiles::LINE_PATH);
read_test!(read_linem, check_linem, testfiles::LINEM_PATH);
read_test!(read_linez, check_linez, testfiles::LINEZ_PATH);

/* Read tests on Points */
read_test!(read_point, check_point, testfiles::POINT_PATH);
read_test!(read_pointm, check_pointm, testfiles::POINTM_PATH);
read_test!(read_pointz, check_pointz, testfiles::POINTZ_PATH);

/* Read tests on Polygon */
read_test!(read_polygon, check_polygon, testfiles::POLYGON_PATH);
read_test!(read_polygonm, check_polygonm, testfiles::POLYGONM_PATH);
read_test!(read_polygonz, check_polygonz, testfiles::POLYGONZ_PATH);

/* Read tests on Multipoint */
read_test!(
    read_multipoint,
    check_multipoint,
    testfiles::MULTIPOINT_PATH
);
read_test!(
    read_multipointz,
    check_multipointz,
    testfiles::MULTIPOINTZ_PATH
);

/* Read tests on Multipatch */
read_test!(
    read_multipatch,
    check_multipatch,
    testfiles::MULTIPATCH_PATH
);

/* Read-Write-Read tests on Polylines */
read_write_read_test!(
    read_write_read_line,
    Polyline,
    check_line,
    testfiles::LINE_PATH
);
read_write_read_test!(
    read_write_read_linem,
    PolylineM,
    check_linem,
    testfiles::LINEM_PATH
);
read_write_read_test!(
    read_write_read_linez,
    PolylineZ,
    check_linez,
    testfiles::LINEZ_PATH
);

/* Read-Write-Read tests on Points */
read_write_read_test!(
    read_write_read_point,
    Point,
    check_point,
    testfiles::POINT_PATH
);
read_write_read_test!(
    read_write_read_pointm,
    PointM,
    check_pointm,
    testfiles::POINTM_PATH
);
read_write_read_test!(
    read_write_read_pointz,
    PointZ,
    check_pointz,
    testfiles::POINTZ_PATH
);

/* Read-Write-Read tests on Polygons */
read_write_read_test!(
    read_write_read_polygon,
    Polygon,
    check_polygon,
    testfiles::POLYGON_PATH
);
read_write_read_test!(
    read_write_read_polygonm,
    PolygonM,
    check_polygonm,
    testfiles::POLYGONM_PATH
);
read_write_read_test!(
    read_write_read_polygonz,
    PolygonZ,
    check_polygonz,
    testfiles::POLYGONZ_PATH
);

/* Read-Write-Read tests on Multipoint */
read_write_read_test!(
    read_write_read_multipoint,
    Multipoint,
    check_multipoint,
    testfiles::MULTIPOINT_PATH
);
read_write_read_test!(
    read_write_read_multipointz,
    MultipointZ,
    check_multipointz,
    testfiles::MULTIPOINTZ_PATH
);

/* Read-Write-Read tests on Multipatch */
read_write_read_test!(
    read_write_read_multipatch,
    Multipatch,
    check_multipatch,
    testfiles::MULTIPATCH_PATH
);

#[test]
fn read_as_point() {
    let points = shapefile::read_as::<&str, shapefile::Point>(testfiles::POINT_PATH);
    assert_eq!(points.is_ok(), true);

    let points = points.unwrap();
    assert_eq!(points.len(), 1);
    check_first_point(&points[0]);
}

#[test]
fn read_as_point_m() {
    let points_m = shapefile::read_as::<&str, shapefile::PointM>(testfiles::POINTM_PATH);
    assert_eq!(points_m.is_ok(), true);

    let points_m = points_m.unwrap();
    assert_eq!(points_m.len(), 2);
    _check_first_point_m(&points_m[0]);
    _check_second_point_m(&points_m[1]);
}

#[test]
fn read_as_point_z() {
    let points_z = shapefile::read_as::<&str, shapefile::PointZ>(testfiles::POINTZ_PATH);
    //assert_eq!(points_z.is_ok(), true, "Reading result is not Ok");

    let points_z = points_z.unwrap();
    assert_eq!(points_z.len(), 2);
    _check_first_point_z(&points_z[0]);
    _check_second_point_z(&points_z[1]);
}

#[test]
fn read_point_as_wrong_type() {
    use shapefile::{Error, ShapeType};
    let points = shapefile::read_as::<&str, shapefile::PointM>(testfiles::POINT_PATH);

    if let Err(error) = points {
        match error {
            Error::MismatchShapeType {
                requested: ShapeType::PointM,
                actual: ShapeType::Point,
            } => {}
            _ => assert!(false),
        }
    } else {
        assert!(false);
    }
}
