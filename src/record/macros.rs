#[macro_export]
macro_rules! multipoint {
    (
        $(x: $x_value:expr, y: $y_value:expr),* $(,)?
    ) => {
        shapefile::Multipoint::new(
            vec![
                $(
                    shapefile::Point {x: $x_value, y: $y_value }
                ),+
            ]
        )
    };
    (
         $(x: $x_value:expr, y: $y_value:expr, m: $m_value:expr),* $(,)?
    ) => {
        shapefile::MultipointM::new(
            vec![
                $(
                    shapefile::PointM {x: $x_value, y: $y_value, m: $m_value }
                ),+
            ]
        )
    };
    (
        $(x: $x_value:expr, y: $y_value:expr, z: $z_value:expr, m: $m_value:expr),* $(,)?
    ) => {
        shapefile::MultipointZ::new(
            vec![
                $(
                    shapefile::PointZ {x: $x_value, y: $y_value, z: $z_value, m: $m_value }
                ),+
            ]
        )
    }
}

#[macro_export]
macro_rules! multipatch {
    ( $( $patch_type:expr => [ $(x: $x_value:expr, y: $y_value:expr, z: $z_value:expr, m: $m_value:expr),* $(,)? ] $(,)? ),* ) => {
        shapefile::Multipatch::with_parts(
            $(
                vec! [
                    (
                        vec![
                            $(shapefile::PointZ {x: $x_value, y: $y_value, z: $z_value, m: $m_value}),*
                        ],
                        $patch_type
                    )
                ]
            ),*
        )
    };
}

/// Macro that we use to define the macros used to create Polyline, Polygon, etc
macro_rules! define_macros {
    // This is a work around https://github.com/rust-lang/rust/issues/35853
    // This comment was useful https://github.com/rust-lang/rust/issues/35853#issuecomment-443110660
    (
        $macro_name:ident, $base_shape:ident, $m_shape:ident, $z_shape:ident, ($dol:tt)
    ) =>
     {
        #[macro_export]
        macro_rules! $macro_name {
            (
                $dol( [ $dol(x: $x_value:expr, y: $y_value:expr),* $dol(,)? ] $dol(,)? ),*
            ) => {
                shapefile::$base_shape::with_parts(
                    vec![
                        $dol(
                            vec![
                                $dol(
                                    shapefile::Point {x: $x_value, y: $y_value }
                                ),+
                            ]
                        ),+
                    ]
                )

            };
            (
                $dol( [ $dol(x: $x_value:expr, y: $y_value:expr, m: $m_value:expr),* $dol(,)? ] $dol(,)? ),*
            ) => {
                shapefile::$m_shape::with_parts(
                    vec![
                        $dol(
                            vec![
                                $dol(
                                    shapefile::PointM {x: $x_value, y: $y_value, m: $m_value }
                                ),+
                            ]
                        ),+
                    ]
                )
            };
            (
                $dol( [ $dol(x: $x_value:expr, y: $y_value:expr, z: $z_value:expr, m: $m_value:expr),* $dol(,)? ] $dol(,)?),*
            ) => {
                shapefile::$z_shape::with_parts(
                    vec![
                        $dol(
                            vec![
                                $dol(
                                    shapefile::PointZ {x: $x_value, y: $y_value, z: $z_value, m: $m_value }
                                ),+
                            ]
                        ),+
                    ]
                )
            }
        }
    };
    (
        $macro_name:ident, $base_shape:ident, $m_shape:ident, $z_shape:ident
    ) =>
    {
        define_macros!($macro_name, $base_shape, $m_shape, $z_shape, ($));
    }
}

define_macros!(polyline, Polyline, PolylineM, PolylineZ);
define_macros!(polygon, Polygon, PolygonM, PolygonZ);

#[cfg(test)]
mod test {
    // the macros expect the shapefile namespace to be in scope
    use crate as shapefile;

    #[test]
    fn test_multipatch() {
        let multipatch = multipatch!(
            shapefile::PatchType::TriangleStrip => [
                x: 1.0, y: 1.0, z: 1.0, m: 1.0
            ],
        );
        let expected_multipatch = shapefile::Multipatch::new(
            vec![shapefile::PointZ::new(1.0, 1.0, 1.0, 1.0)],
            shapefile::PatchType::TriangleStrip
        );

        assert_eq!(multipatch, expected_multipatch);
    }

    #[test]
    fn test_multipoint_macro() {
        let multipoint = multipoint![
                x: 1.0, y: 1.0,
                x: 2.0, y: 2.0,
        ];
        let expected_multipoint = shapefile::Multipoint::new(
            vec![shapefile::Point::new(1.0, 1.0), shapefile::Point::new(2.0, 2.0)]);
        assert_eq!(multipoint, expected_multipoint);
    }

    #[test]
    fn test_multipoint_m_macro() {
        let multipoint = multipoint![
                x: 1.0, y: 1.0, m: 42.1337,
                x: 2.0, y: 2.0, m: 1337.42
        ];
        let expected_multipoint = shapefile::MultipointM::new(
            vec![
                shapefile::PointM::new(1.0, 1.0, 42.1337),
                shapefile::PointM::new(2.0, 2.0, 1337.42),
            ]
        );
        assert_eq!(multipoint, expected_multipoint);
    }

    #[test]
    fn test_multipoint_z_macro() {
        let multipoint = multipoint![
                x: 1.0, y: 1.0, z: 17.5, m: 42.1337,
                x: 2.0, y: 2.0, z: 14.021, m: 1337.42
        ];
        let expected_multipoint = shapefile::MultipointZ::new(
            vec![
                shapefile::PointZ::new(1.0, 1.0, 17.5, 42.1337),
                shapefile::PointZ::new(2.0, 2.0, 14.021, 1337.42),
            ]
        );
        assert_eq!(multipoint, expected_multipoint);
    }

    #[test]
    fn test_polyline_macro() {
        let poly = polyline!(
            [
                x: 1.0, y: 1.0,
                x: 2.0, y: 2.0
            ],
            [
                x: 3.0, y: 3.0,
                x: 3.0, y: 3.0
            ]
        );

        let expected_points = vec! [
                shapefile::Point { x: 1.0, y: 1.0 },
                shapefile::Point { x: 2.0, y: 2.0 },
                shapefile::Point { x: 3.0, y: 3.0 },
                shapefile::Point { x: 3.0, y: 3.0 },
        ];
        assert_eq!(poly.points, expected_points);
        assert_eq!(poly.parts, vec![0, 2]);
    }


    #[test]
    fn test_polyline_m_macro() {
        let poly = polyline!(
            [
                x: 1.0, y: 1.0, m: 5.0,
                x: 2.0, y: 2.0, m: 42.1337
            ],
            [
                x: 3.0, y: 3.0, m: 17.65,
                x: 3.0, y: 3.0, m: 454.4598
            ]
        );

        let expected_points = vec![
            shapefile::PointM { x: 1.0, y: 1.0, m: 5.0 },
            shapefile::PointM { x: 2.0, y: 2.0, m: 42.1337 },
            shapefile::PointM { x: 3.0, y: 3.0, m: 17.65 },
            shapefile::PointM { x: 3.0, y: 3.0, m: 454.4598 },
        ];
        assert_eq!(poly.points, expected_points);
        assert_eq!(poly.parts, vec![0, 2]);
    }


    #[test]
    fn test_polyline_z_macro() {
        let poly = polyline!(
            [
                x: 1.0, y: 1.0, z: 17.56, m: 5.0,
                x: 2.0, y: 2.0, z: 18.17, m: 42.1337
            ],
            [
                x: 3.0, y: 3.0, z: 54.9, m: 17.65,
                x: 3.0, y: 3.0, z: 7.0, m: 454.4598
            ]
        );

        let expected_points = vec![
            shapefile::PointZ { x: 1.0, y: 1.0, z: 17.56, m: 5.0 },
            shapefile::PointZ { x: 2.0, y: 2.0, z: 18.17, m: 42.1337 },
            shapefile::PointZ { x: 3.0, y: 3.0, z: 54.9, m: 17.65 },
            shapefile::PointZ { x: 3.0, y: 3.0, z: 7.0, m: 454.4598 },
        ];
        assert_eq!(poly.points, expected_points);
        assert_eq!(poly.parts, vec![0, 2]);
    }

    #[test]
    fn test_polygon_macro() {
        let poly = polygon!(
            [
                x: 1.0, y: 1.0,
                x: 2.0, y: 2.0,
                x: 1.0, y: 1.0,
            ],
            [
                x: 3.0, y: 3.0,
                x: 3.0, y: 3.0,
                x: 3.0, y: 3.0,
            ],
        );

        let expected_points = vec![
            shapefile::Point { x: 1.0, y: 1.0 },
            shapefile::Point { x: 2.0, y: 2.0 },
            shapefile::Point { x: 1.0, y: 1.0 },
            shapefile::Point { x: 3.0, y: 3.0 },
            shapefile::Point { x: 3.0, y: 3.0 },
            shapefile::Point { x: 3.0, y: 3.0 },
        ];
        assert_eq!(poly.points, expected_points);
        assert_eq!(poly.parts, vec![0, 3]);
    }


    #[test]
    fn test_polygon_m_macro() {
        let poly = polygon!(
            [
                x: 1.0, y: 1.0, m: 5.0,
                x: 2.0, y: 2.0, m: 42.1337,
                x: 1.0, y: 1.0, m: 5.0,
            ],
            [
                x: 3.0, y: 3.0, m: 17.65,
                x: 3.0, y: 3.0, m: 454.4598,
                x: 3.0, y: 3.0, m: 17.65,
            ],
        );

        let expected_points = vec![
            shapefile::PointM { x: 1.0, y: 1.0, m: 5.0 },
            shapefile::PointM { x: 2.0, y: 2.0, m: 42.1337 },
            shapefile::PointM { x: 1.0, y: 1.0, m: 5.0 },
            shapefile::PointM { x: 3.0, y: 3.0, m: 17.65 },
            shapefile::PointM { x: 3.0, y: 3.0, m: 454.4598 },
            shapefile::PointM { x: 3.0, y: 3.0, m: 17.65 },
        ];
        assert_eq!(poly.points, expected_points);
        assert_eq!(poly.parts, vec![0, 3]);
    }


    #[test]
    fn test_polygon_z_macro() {
        let poly = polygon!(
            [
                x: 1.0, y: 1.0, z: 17.56, m: 5.0,
                x: 2.0, y: 2.0, z: 18.17, m: 42.1337,
                x: 1.0, y: 1.0, z: 17.56, m: 5.0,
            ],
            [
                x: 3.0, y: 3.0, z: 54.9, m: 17.65,
                x: 3.0, y: 3.0, z: 7.0, m: 454.4598,
                x: 3.0, y: 3.0, z: 54.9, m: 17.65,
            ],
        );

        let expected_points = vec![
            shapefile::PointZ { x: 1.0, y: 1.0, z: 17.56, m: 5.0 },
            shapefile::PointZ { x: 2.0, y: 2.0, z: 18.17, m: 42.1337 },
            shapefile::PointZ { x: 1.0, y: 1.0, z: 17.56, m: 5.0 },
            shapefile::PointZ { x: 3.0, y: 3.0, z: 54.9, m: 17.65 },
            shapefile::PointZ { x: 3.0, y: 3.0, z: 7.0, m: 454.4598 },
            shapefile::PointZ { x: 3.0, y: 3.0, z: 54.9, m: 17.65 },
        ];
        assert_eq!(poly.points, expected_points);
        assert_eq!(poly.parts, vec![0, 3]);
    }
}
