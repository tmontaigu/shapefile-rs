use byteorder::{LittleEndian, BigEndian, ReadBytesExt};

use super::{ShapeType, ShpError};
use std::io::Read;

struct BBox {
    xmin: f64,
    ymin: f64,
    xmax: f64,
    ymax: f64
}

impl BBox {
    pub fn read_from<T: Read>(mut source: T) -> Result<BBox, std::io::Error>{
        let xmin = source.read_f64::<LittleEndian>()?;
        let ymin = source.read_f64::<LittleEndian>()?;
        let xmax = source.read_f64::<LittleEndian>()?;
        let ymax = source.read_f64::<LittleEndian>()?;
        Ok(BBox{xmin, ymin, xmax, ymax})
    }
}

struct RecordHeader {
    record_number: i32,
    record_size: i32,
    shape_type: ShapeType,
}

impl RecordHeader {
    pub fn read_from<T: Read>(mut source: T) -> Result<RecordHeader, ShpError> {
        let record_number = source.read_i32::<BigEndian>()?;
        let record_size = source.read_i32::<BigEndian>()?;
        let shape_type = ShapeType::read_from(&mut source)?;
        Ok(RecordHeader{record_number, record_size, shape_type})
    }
}

struct Polyline {
    bbox: BBox,
    num_parts: i32,
    num_points: i32,
    parts: Vec<i32>,
    xs: Vec<i32>,
    ys: Vec<i32>,
}

