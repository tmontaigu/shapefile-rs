# 0.1.0

 - Fix index file (.shx) that was incorrect (Github issue #6)
 - Fix reading PointZ shape where the 'M' value is not there at all
 - PointM, PointZ 'std::fmt::Display' implementation now prints 'NO_DATA'
   when the value is NO_DATA (instead of printing the f64 value)
 - Implement MultipointShape for Point, PointM, PointZ
