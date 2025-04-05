# 0.7.0
 - Bumped dbase to 0.6
 - Added `yore` and `encoding_rs` features which are forwarded to `dbase`
   allowing to read dbf files with special encodings.
 - Added `finalize` method to writer, to be able to explicitly handle errors when the file
   is finalized (instead of  silently ignoring errors by relying on the drop mechanism)
 - Fixed overflow that could happen on large files
 - Fixes performance issue for shapefiles which had a .shx index file

# 0.6.0
 - Bumped dbase to 0.5.0

# 0.5.0
 - Bumped dbase to 0.4.0

# 0.4.0
 - Added `shape_count` to the reader
 - Bumped dbase to 0.3.0 to bring code page support
 - Fixed: Use the .shx (index file, if present) when iterating over the shapes
   contained in the file, as some files may have padding bytes between shapes.
 - Changed, the `Reader::with_shx` now can work when the source for the shx file
   can is a different type than the shp source type (can mix io::Cursor and fs::File for example).
 - Changed the `Reader` to be able to use different type for the sources of the dbase and shape file
   (e.g. dbase source could be a fs::File whil the shape is a io::Cursor)

# 0.3.0
 - Updated dbase dependency to 0.2.x
 - Added `Writer::write_shape` to write one shape at a time
 - Changed `Write<T>` the `T` now must implement `std::io::Seek` and `std::io::Write`.
   `std::fs::File` and `std::io::Cursor` are valid `T`.
 - Changed `ShapeWriter::write_shapes` to take as input any type that implements
   `IntoIterator<Item=&ShapeType>`.
 - Fixed `ShapeType::Multipatch` wasn't considered as a type with Z coordinates.
 - Added a `ShapeReader` &`ShapeWriter` struct that only read/write the .shp and .shx
 - Changed the `Reader`, it now requires the .dbf to exist
 - Changed the `Writer`, it requires more information to be able to write the .dbf file
   (Examples are in the docs)
 - Changed the `Reader` `iter_*` & `read` to take `&mut self` instead of `self` 
 - Changed `shapefile::read` now returns a `Vec<(Shape, Record)>` 
   `shapefile::read_shapes` returns `Vec<Shape>`

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
