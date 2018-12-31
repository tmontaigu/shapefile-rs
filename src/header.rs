use crate::{ShapeType, Error};

use std::io::Read;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};

pub const SHP_HEADER_SIZE: i32 = 100;
const SHP_FILE_CODE: i32 = 9994;

pub struct Header {
    pub file_length: i32,
    pub point_min: [f64; 3],
    pub point_max: [f64; 3],
    pub m_range: [f64; 2],
    pub shape_type: ShapeType,
    pub version: i32,
}

impl Default for Header {
    fn default() -> Self {
        Header {
            point_min: [0.0; 3],
            point_max: [0.0; 3],
            m_range: [0.0; 2],
            shape_type: ShapeType::NullShape,
            file_length: SHP_HEADER_SIZE,
            version: 1000,
        }
    }
}

impl Header {
    pub fn read_from<T: Read>(mut source: &mut T) -> Result<Header, Error> {
        let file_code = source.read_i32::<BigEndian>()?;

        if file_code != SHP_FILE_CODE {
            return Err(Error::InvalidFileCode(file_code));
        }

        const SIZE_OF_SKIP: usize = std::mem::size_of::<i32>() * 5;
        let mut skip: [u8; SIZE_OF_SKIP] = [0; SIZE_OF_SKIP];
        source.read_exact(&mut skip)?;

        let file_length_16_bit = source.read_i32::<BigEndian>()?;
        let version = source.read_i32::<LittleEndian>()?;
        let shape_type = ShapeType::read_from(&mut source)?;

        let mut hdr = Header::default();
        hdr.shape_type = shape_type;
        hdr.version = version;
        hdr.file_length = file_length_16_bit;

        hdr.point_min[0] = source.read_f64::<LittleEndian>()?;
        hdr.point_min[1] = source.read_f64::<LittleEndian>()?;

        hdr.point_max[0] = source.read_f64::<LittleEndian>()?;
        hdr.point_max[1] = source.read_f64::<LittleEndian>()?;

        hdr.point_min[2] = source.read_f64::<LittleEndian>()?;
        hdr.point_max[2] = source.read_f64::<LittleEndian>()?;

        hdr.m_range[0] = source.read_f64::<LittleEndian>()?;
        hdr.m_range[1] = source.read_f64::<LittleEndian>()?;

        Ok(hdr)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    use byteorder::WriteBytesExt;
    use std::io::{SeekFrom, Seek};

    #[test]
    fn wrong_file_code() {
        use std::io::Cursor;

        let mut src = Cursor::new(vec![]);
        src.write_i32::<BigEndian>(42).unwrap();

        src.seek(SeekFrom::Start(0)).unwrap();
        assert!(Header::read_from(&mut src).is_err());
    }
}
