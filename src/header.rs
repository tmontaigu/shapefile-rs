use super::{Error, ShapeType};

use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use record::BBoxZ;
use std::io::{Read, Write};

pub(crate) const HEADER_SIZE: i32 = 100;
const FILE_CODE: i32 = 9994;
/// Size of reserved bytes in the header, that have do defined use
const SIZE_OF_SKIP: usize = std::mem::size_of::<i32>() * 5;

/// struct representing the Header of a shapefile
/// can be retrieved via the reader used to read
#[derive(Copy, Clone, PartialEq)]
pub struct Header {
    /// Total file length (Header + Shapes) in 16bit word
    pub file_length: i32,
    /// The bbox contained all the shapes in this shapefile
    ///
    /// For shapefiles where the shapes do not have `m` or `z` values
    /// the associated min and max will be `0`s.
    pub bbox: BBoxZ,
    /// Type of all the shapes in the file
    /// (as mixing shapes is not allowed)
    pub shape_type: ShapeType,
    /// Version of the shapefile specification
    pub version: i32,
}

impl Default for Header {
    fn default() -> Self {
        Header {
            bbox: BBoxZ::default(),
            shape_type: ShapeType::NullShape,
            file_length: HEADER_SIZE / 2,
            version: 1000,
        }
    }
}

impl Header {
    pub fn read_from<T: Read>(mut source: &mut T) -> Result<Header, Error> {
        let file_code = source.read_i32::<BigEndian>()?;

        if file_code != FILE_CODE {
            return Err(Error::InvalidFileCode(file_code));
        }

        let mut skip: [u8; SIZE_OF_SKIP] = [0; SIZE_OF_SKIP];
        source.read_exact(&mut skip)?;

        let file_length_16_bit = source.read_i32::<BigEndian>()?;
        let version = source.read_i32::<LittleEndian>()?;
        let shape_type = ShapeType::read_from(&mut source)?;

        let mut hdr = Header::default();
        hdr.shape_type = shape_type;
        hdr.version = version;
        hdr.file_length = file_length_16_bit;

        hdr.bbox.min.x = source.read_f64::<LittleEndian>()?;
        hdr.bbox.min.y = source.read_f64::<LittleEndian>()?;
        hdr.bbox.max.x = source.read_f64::<LittleEndian>()?;
        hdr.bbox.max.y = source.read_f64::<LittleEndian>()?;
        hdr.bbox.min.z = source.read_f64::<LittleEndian>()?;
        hdr.bbox.max.z = source.read_f64::<LittleEndian>()?;
        hdr.bbox.min.m = source.read_f64::<LittleEndian>()?;
        hdr.bbox.max.m = source.read_f64::<LittleEndian>()?;

        Ok(hdr)
    }

    pub(crate) fn write_to<T: Write>(&self, dest: &mut T) -> Result<(), std::io::Error> {
        dest.write_i32::<BigEndian>(FILE_CODE)?;

        let skip: [u8; SIZE_OF_SKIP] = [0; SIZE_OF_SKIP];
        dest.write_all(&skip)?;

        dest.write_i32::<BigEndian>(self.file_length)?;
        dest.write_i32::<LittleEndian>(self.version)?;
        dest.write_i32::<LittleEndian>(self.shape_type as i32)?;

        dest.write_f64::<LittleEndian>(self.bbox.min.x)?;
        dest.write_f64::<LittleEndian>(self.bbox.min.y)?;
        dest.write_f64::<LittleEndian>(self.bbox.max.x)?;
        dest.write_f64::<LittleEndian>(self.bbox.max.y)?;
        dest.write_f64::<LittleEndian>(self.bbox.min.z)?;
        dest.write_f64::<LittleEndian>(self.bbox.max.z)?;
        dest.write_f64::<LittleEndian>(self.bbox.min.m)?;
        dest.write_f64::<LittleEndian>(self.bbox.max.m)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use byteorder::WriteBytesExt;
    use std::io::{Seek, SeekFrom};

    #[test]
    fn wrong_file_code() {
        use std::io::Cursor;

        let mut src = Cursor::new(vec![]);
        src.write_i32::<BigEndian>(42).unwrap();

        src.seek(SeekFrom::Start(0)).unwrap();
        assert!(Header::read_from(&mut src).is_err());
    }
}
