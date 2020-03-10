#[macro_export]
macro_rules! multipoint {
    (
        $( {x: $x_value:expr, y: $y_value:expr} ),* $(,)?
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
         $( {x: $x_value:expr, y: $y_value:expr, m: $m_value:expr}),* $(,)?
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
        $( {x: $x_value:expr, y: $y_value:expr, z: $z_value:expr, m: $m_value:expr} ),* $(,)?
    ) => {
        shapefile::MultipointZ::new(
            vec![
                $(
                    shapefile::PointZ {x: $x_value, y: $y_value, z: $z_value, m: $m_value }
                ),+
            ]
        )
    };
    (
        $( ($x_value:expr, $y_value:expr) ),* $(,)?
    ) => {
        shapefile::Multipoint::new(
            vec![
                $(
                    shapefile::Point::new($x_value, $y_value)
                ),+
            ]
        )
    };
     (
         $( ($x_value:expr, $y_value:expr, $m_value:expr) ),* $(,)?
    ) => {
        shapefile::MultipointM::new(
            vec![
                $(
                    shapefile::PointM::new($x_value, $y_value, $m_value)
                ),+
            ]
        )
    };
    (
        $( ($x_value:expr, $y_value:expr, $z_value:expr, $m_value:expr) ),* $(,)?
    ) => {
        shapefile::MultipointZ::new(
            vec![
                $(
                    shapefile::PointZ::new($x_value, $y_value, $z_value, $m_value)
                ),+
            ]
        )
    };
}

#[macro_export]
macro_rules! multipatch {
    (
        $(
            $patch_type:ident( $({x: $x_value:expr, y: $y_value:expr, z: $z_value:expr, m: $m_value:expr}),* $(,)?  $(,)? )
        ),*
    ) => {
        shapefile::Multipatch::with_parts(
            vec! [
                $(
                    shapefile::Patch::$patch_type(
                        vec![
                            $(shapefile::PointZ {x: $x_value, y: $y_value, z: $z_value, m: $m_value}),*
                        ]
                    )
                ),*
            ]
        )
    };
    (
        $(
            $patch_type:ident( $(($x_value:expr, $y_value:expr, $z_value:expr, $m_value:expr)),* $(,)?  $(,)? )
        ),*
    ) => {
        shapefile::Multipatch::with_parts(
            vec! [
                $(
                    shapefile::Patch::$patch_type(
                        vec![
                            $(shapefile::PointZ::new($x_value, $y_value, $z_value, $m_value)),*
                        ]
                    )
                ),*
            ]
        )
    };
}


#[macro_export]
macro_rules! polygon {
    // Polygon rules
    (
        $(
            $ring_type:ident( $({x: $x_value:expr, y: $y_value:expr}),* $(,)? ) $(,)?
        ),*
    ) => {
        shapefile::Polygon::with_rings(
            vec! [
                $(
                    shapefile::PolygonRing::$ring_type(
                        vec![
                            $(shapefile::Point {x: $x_value, y: $y_value}),*
                        ]
                    )
                ),*
            ]
        )
    };
    (
        $(
            $ring_type:ident( $(($x_value:expr, $y_value:expr)),* $(,)? ) $(,)?
        ),*
    ) => {
        polygon!{
            $(
                $ring_type( $({x: $x_value, y: $y_value}),* )
            ),+
        }
    };
    // Polygon M rules
    (
        $(
            $ring_type:ident( $({x: $x_value:expr, y: $y_value:expr, m: $m_value:expr}),* $(,)? ) $(,)?
        ),*
    ) => {
        shapefile::PolygonM::with_rings(
            vec! [
                $(
                    shapefile::PolygonRing::$ring_type(
                        vec![
                            $(shapefile::PointM {x: $x_value, y: $y_value, m: $m_value}),*
                        ]
                    )
                ),*
            ]
        )
    };
    (
        $(
            $ring_type:ident( $(($x_value:expr, $y_value:expr, $m_value:expr)),* $(,)? ) $(,)?
        ),*
    ) => {
        polygon!{
            $(
                $ring_type( $({x: $x_value, y: $y_value, m: $m_value}),* )
            ),+
        }
    };
    //Polygon Z rules
    (
        $(
            $ring_type:ident( $({x: $x_value:expr, y: $y_value:expr, z: $z_value:expr, m: $m_value:expr}),* $(,)? ) $(,)?
        ),*
    ) => {
        shapefile::PolygonZ::with_rings(
            vec! [
                $(
                    shapefile::PolygonRing::$ring_type(
                        vec![
                            $(shapefile::PointZ {x: $x_value, y: $y_value, z: $z_value, m: $m_value}),*
                        ]
                    )
                ),*
            ]
        )
    };
    (
        $(
            $ring_type:ident( $(($x_value:expr, $y_value:expr, $z_value:expr, $m_value:expr)),* $(,)? ) $(,)?
        ),*
    ) => {
        polygon!{
            $(
                $ring_type( $({x: $x_value, y: $y_value, z: $z_value, m: $m_value}),* )
            ),+
        }
    };
}

#[cfg(test)]
mod test {
    // the macros expect the shapefile namespace to be in scope
    use crate as shapefile;
    use crate::{Patch};
    use PolygonRing;

    #[test]
    fn test_multipatch() {
        let multipatch = multipatch!(
            TriangleStrip(
                {x: 1.0, y: 1.0, z: 1.0, m: 1.0}
            )
        );
        let multipatch_2 = multipatch!(
            TriangleStrip(
                (1.0, 1.0, 1.0, 1.0)
            )
        );
        let expected_multipatch = shapefile::Multipatch::new(
            Patch::TriangleStrip(vec![shapefile::PointZ::new(1.0, 1.0, 1.0, 1.0)]),
        );

        assert_eq!(multipatch, expected_multipatch);
        assert_eq!(multipatch_2, expected_multipatch);
    }

    #[test]
    fn test_multipoint_macro() {
        let multipoint = multipoint![
                {x: 1.0, y: 1.0},
                {x: 2.0, y: 2.0},
        ];
        let multipoint_2 = multipoint![(1.0, 1.0), (2.0,2.0)];
        let expected_multipoint = shapefile::Multipoint::new(
            vec![shapefile::Point::new(1.0, 1.0), shapefile::Point::new(2.0, 2.0)]);

        assert_eq!(multipoint, expected_multipoint);
        assert_eq!(multipoint, multipoint_2);
    }

    #[test]
    fn test_multipoint_m_macro() {
        let multipoint = multipoint![
                {x: 1.0, y: 1.0, m: 42.1337},
                {x: 2.0, y: 2.0, m: 1337.42}
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
                {x: 1.0, y: 1.0, z: 17.5, m: 42.1337},
                {x: 2.0, y: 2.0, z: 14.021, m: 1337.42}
        ];
        let expected_multipoint = shapefile::MultipointZ::new(
            vec![
                shapefile::PointZ::new(1.0, 1.0, 17.5, 42.1337),
                shapefile::PointZ::new(2.0, 2.0, 14.021, 1337.42),
            ]
        );
        assert_eq!(multipoint, expected_multipoint);
    }
    //
    // #[test]
    // fn test_polyline_macro() {
    //     let poly = polyline!(
    //         [
    //             x: 1.0, y: 1.0,
    //             x: 2.0, y: 2.0
    //         ],
    //         [
    //             x: 3.0, y: 3.0,
    //             x: 3.0, y: 3.0
    //         ]
    //     );
    //
    //     let expected_points = vec! [
    //             shapefile::Point { x: 1.0, y: 1.0 },
    //             shapefile::Point { x: 2.0, y: 2.0 },
    //             shapefile::Point { x: 3.0, y: 3.0 },
    //             shapefile::Point { x: 3.0, y: 3.0 },
    //     ];
    //     assert_eq!(poly.points, expected_points);
    //     assert_eq!(poly.parts, vec![0, 2]);
    // }
    //
    //
    // #[test]
    // fn test_polyline_m_macro() {
    //     let poly = polyline!(
    //         [
    //             x: 1.0, y: 1.0, m: 5.0,
    //             x: 2.0, y: 2.0, m: 42.1337
    //         ],
    //         [
    //             x: 3.0, y: 3.0, m: 17.65,
    //             x: 3.0, y: 3.0, m: 454.4598
    //         ]
    //     );
    //
    //     let expected_points = vec![
    //         shapefile::PointM { x: 1.0, y: 1.0, m: 5.0 },
    //         shapefile::PointM { x: 2.0, y: 2.0, m: 42.1337 },
    //         shapefile::PointM { x: 3.0, y: 3.0, m: 17.65 },
    //         shapefile::PointM { x: 3.0, y: 3.0, m: 454.4598 },
    //     ];
    //     assert_eq!(poly.points, expected_points);
    //     assert_eq!(poly.parts, vec![0, 2]);
    // }
    //
    //
    // #[test]
    // fn test_polyline_z_macro() {
    //     let poly = polyline!(
    //         [
    //             x: 1.0, y: 1.0, z: 17.56, m: 5.0,
    //             x: 2.0, y: 2.0, z: 18.17, m: 42.1337
    //         ],
    //         [
    //             x: 3.0, y: 3.0, z: 54.9, m: 17.65,
    //             x: 3.0, y: 3.0, z: 7.0, m: 454.4598
    //         ]
    //     );
    //
    //     let expected_points = vec![
    //         shapefile::PointZ { x: 1.0, y: 1.0, z: 17.56, m: 5.0 },
    //         shapefile::PointZ { x: 2.0, y: 2.0, z: 18.17, m: 42.1337 },
    //         shapefile::PointZ { x: 3.0, y: 3.0, z: 54.9, m: 17.65 },
    //         shapefile::PointZ { x: 3.0, y: 3.0, z: 7.0, m: 454.4598 },
    //     ];
    //     assert_eq!(poly.points, expected_points);
    //     assert_eq!(poly.parts, vec![0, 2]);
    // }
    //
    #[test]
    fn test_polygon_macro() {
        let polygon_1 = polygon!(
            Outer(
                {x: 1.0, y: 1.0},
                {x: 2.0, y: 2.0},
                {x: 1.0, y: 1.0},
                {x: 1.0, y: 0.0},
                {x: 1.0, y: 1.0}
            ),
            Inner(
                {x: 1.0, y: 1.0},
                {x: 1.0, y: 0.0},
                {x: 1.0, y: 1.0},
                {x: 2.0, y: 2.0},
                {x: 1.0, y: 1.0},
            )
        );

        let polygon_2 = polygon!(
            Outer(
                (1.0, 1.0),
                (2.0, 2.0),
                (1.0, 1.0),
                (1.0, 0.0),
                (1.0, 1.0),
            ),
            Inner(
                (1.0, 1.0),
                (1.0, 0.0),
                (1.0, 1.0),
                (2.0, 2.0),
                (1.0, 1.0),
            )
        );

        let polygon_3 = shapefile::Polygon::with_rings(vec![
            PolygonRing::Outer(
                vec![
                    shapefile::Point::new(1.0, 1.0),
                    shapefile::Point::new(2.0, 2.0),
                    shapefile::Point::new(1.0, 1.0),
                    shapefile::Point::new(1.0, 0.0),
                    shapefile::Point::new(1.0, 1.0),
                ]
            ),
            PolygonRing::Inner(
                vec![
                    shapefile::Point::new(1.0, 1.0),
                    shapefile::Point::new(1.0, 0.0),
                    shapefile::Point::new(1.0, 1.0),
                    shapefile::Point::new(2.0, 2.0),
                    shapefile::Point::new(1.0, 1.0),
                ]
            )
        ]);
        assert_eq!(polygon_1, polygon_3);
        assert_eq!(polygon_1, polygon_2);
    }


    #[test]
    fn test_polygon_m_macro() {
        let polygon_1 = polygon!(
            Outer(
                {x: 1.0, y: 1.0, m: 5.0},
                {x: 2.0, y: 2.0, m: 42.1337},
                {x: 1.0, y: 1.0, m: 5.0},
                {x: 1.0, y: 0.0, m: 2.2},
                {x: 1.0, y: 1.0, m: 5.0}
            ),
            Inner(
                {x: 1.0, y: 1.0, m: 1.0},
                {x: 1.0, y: 0.0, m: 2.0},
                {x: 1.0, y: 1.0, m: 1.1},
                {x: 2.0, y: 2.0, m: 2.2},
                {x: 1.0, y: 1.0, m: 1.0},
            )
        );

        let polygon_2 = polygon!(
            Outer(
                (1.0, 1.0, 5.0),
                (2.0, 2.0, 42.1337),
                (1.0, 1.0, 5.0),
                (1.0, 0.0, 2.2),
                (1.0, 1.0, 5.0)
            ),
            Inner(
                (1.0, 1.0, 1.0),
                (1.0, 0.0, 2.0),
                (1.0, 1.0, 1.1),
                (2.0, 2.0, 2.2),
                (1.0, 1.0, 1.0),
            )
        );

        let polygon_3 = shapefile::PolygonM::with_rings(vec![
            PolygonRing::Outer(
                vec![
                    shapefile::PointM::new(1.0, 1.0, 5.0),
                    shapefile::PointM::new(2.0, 2.0, 42.1337),
                    shapefile::PointM::new(1.0, 1.0, 5.0),
                    shapefile::PointM::new(1.0, 0.0, 2.2),
                    shapefile::PointM::new(1.0, 1.0, 5.0),
                ]
            ),
            PolygonRing::Inner(
                vec![
                    shapefile::PointM::new(1.0, 1.0, 1.0),
                    shapefile::PointM::new(1.0, 0.0, 2.0),
                    shapefile::PointM::new(1.0, 1.0, 1.1),
                    shapefile::PointM::new(2.0, 2.0, 2.2),
                    shapefile::PointM::new(1.0, 1.0, 1.0),
                ]
            )
        ]);

        assert_eq!(polygon_1, polygon_3);
        assert_eq!(polygon_2, polygon_3);
    }


    #[test]
    fn test_polygon_z_macro() {
        let polygon_1 = polygon!(
            Outer(
                {x: 1.0, y: 1.0, z: 5.0, m: 5.0},
                {x: 2.0, y: 2.0, z: 6.0, m: 42.1337},
                {x: 1.0, y: 1.0, z: 7.0, m: 5.0},
                {x: 1.0, y: 0.0, z: 8.0, m: 2.2},
                {x: 1.0, y: 1.0, z: 5.0, m: 5.0}
            ),
            Inner(
                {x: 1.0, y: 1.0, z: 6.0, m: 1.0},
                {x: 1.0, y: 0.0, z: 7.0,m: 2.0},
                {x: 1.0, y: 1.0, z: 8.0, m: 1.1},
                {x: 2.0, y: 2.0, z: 9.9, m: 2.2},
                {x: 1.0, y: 1.0, z: 6.0, m: 1.0},
            )
        );

        let polygon_2 = polygon!(
            Outer(
                (1.0, 1.0, 5.0, 5.0),
                (2.0, 2.0, 6.0, 42.1337),
                (1.0, 1.0, 7.0, 5.0),
                (1.0, 0.0, 8.0, 2.2),
                (1.0, 1.0, 5.0, 5.0)
            ),
            Inner(
                (1.0, 1.0, 6.0, 1.0),
                (1.0, 0.0, 7.0, 2.0),
                (1.0, 1.0, 8.0, 1.1),
                (2.0, 2.0, 9.9, 2.2),
                (1.0, 1.0, 6.0, 1.0),
            )
        );

        let polygon_3 = shapefile::PolygonZ::with_rings(vec![
            PolygonRing::Outer(
                vec![
                    shapefile::PointZ::new(1.0, 1.0, 5.0,5.0),
                    shapefile::PointZ::new(2.0, 2.0, 6.0,42.1337),
                    shapefile::PointZ::new(1.0, 1.0, 7.0,5.0),
                    shapefile::PointZ::new(1.0, 0.0, 8.0,2.2),
                    shapefile::PointZ::new(1.0, 1.0, 5.0,5.0),
                ]
            ),
            PolygonRing::Inner(
                vec![
                    shapefile::PointZ::new(1.0, 1.0, 6.0,1.0),
                    shapefile::PointZ::new(1.0, 0.0, 7.0,2.0),
                    shapefile::PointZ::new(1.0, 1.0, 8.0,1.1),
                    shapefile::PointZ::new(2.0, 2.0, 9.9,2.2),
                    shapefile::PointZ::new(1.0, 1.0, 6.0,1.0),
                ]
            )
        ]);

        assert_eq!(polygon_1, polygon_3);
        assert_eq!(polygon_2, polygon_3);
    }
}
