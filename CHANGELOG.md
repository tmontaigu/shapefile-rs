# 0.2.2
 - Bumped geo-types optional dependency to allow up to 0.8.0

# 0.1.1
 - Fixed a problem in the Multipatch/Polygon/Polyline::with_parts ctor which resulted in
   wrong parts creation (Github PR #10)
 - Fixed another index file bug (Github PR #8)
 - Fixed a bug in the Polygon::with_parts that would result in inner ring points
   being reordered to outer ring point order (Github PR #12)
 - Added #[derive(Debug, PartialEq, Clone)] for Polylines, Polygons, Multipoints

# 0.1.0

 - Fix index file (.shx) that was incorrect (Github issue #6)
 - Fix reading PointZ shape where the 'M' value is not there at all
 - PointM, PointZ 'std::fmt::Display' implementation now prints 'NO_DATA'
   when the value is NO_DATA (instead of printing the f64 value)
 - Implement MultipointShape for Point, PointM, PointZ
