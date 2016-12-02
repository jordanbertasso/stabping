use std::mem;
use std::slice;
use std::io;
use std::io::{Write, BufWriter};
use std::sync::Arc;

use memmap::{Mmap, Protection};
use iron::response::{WriteBody, ResponseBody};

use helpers::VecIntoRawBytes;
use persist::{TargetManager, ManagerError};
use options::SENTINEL_NODATA;

#[derive(RustcEncodable, RustcDecodable, Debug)]
pub struct DataRequest {
    nonce: i32,
    lower: i32,
    upper: i32,
}

#[repr(C, packed)]
struct DataElement {
    time: i32,
    index: i32,
    val: i32,
}

pub struct SPDataReader {
    lower: i32,
    upper: i32,
    tm: Arc<TargetManager>,
}

impl SPDataReader {
    pub fn new(tm: Arc<TargetManager>, dr: DataRequest) -> Option<Self> {
        if dr.nonce != tm.options_read().nonce {
            return None;
        }

        Some(SPDataReader{
            lower: dr.lower,
            upper: dr.upper,
            tm: tm,
        })
    }
}

impl WriteBody for SPDataReader {
    fn write_body(&mut self, res: &mut ResponseBody) -> io::Result<()> {
        let (nonce, ordered_list, mut membership) = self.tm.get_current_indices();
        if nonce != self.tm.options_read().nonce {
            println!("Nonce mismatch in WriteBody for SPDatReader!");
            return Ok(())
        }

        let guard = self.tm.data_file_read();
        if let Ok(map) = Mmap::open(&*guard, Protection::Read) {
            let data: &[DataElement] = unsafe {
                let orig = map.as_slice();
                let raw_ptr = orig.as_ptr();

                let orig_len = orig.len();
                if orig_len % mem::size_of::<DataElement>() != 0 {
                    println!("ERROR: data file not a multiple 3 * 4 bytes!");
                    return Err(io::Error::new(io::ErrorKind::Other, "Data file incorrect multiple!"));
                }
                let new_len = orig.len() / mem::size_of::<DataElement>();

                mem::forget(orig);
                slice::from_raw_parts(raw_ptr as *const DataElement, new_len)
            };

            let begin = match data.binary_search_by_key(&self.lower, |d| d.time) {
                Ok(mut i) => {
                    while i >= 0 && data[i].time == self.lower {
                        i -= 1;
                    }
                    i
                },
                Err(i) => i
            };

            let end = match data.binary_search_by_key(&self.upper, |d| d.time) {
                Ok(mut i) => {
                    while i < data.len() && data[i].time == self.upper {
                        i += 1;
                    }
                    i
                },
                Err(i) => i
            };

            let mut writer = BufWriter::new(res);

            let mut buf: Vec<i32> = Vec::with_capacity(1 + ordered_list.len());

            let mut cur = data[begin].time;
            for d in &data[begin..end] {
                if cur != d.time {
                    buf.push(cur);
                    for &i in ordered_list.iter() {
                        buf.push(membership[i as usize]);
                        membership[i as usize] = SENTINEL_NODATA;
                    }
                    writer.write_all(&buf.into_raw_bytes());
                    buf = Vec::with_capacity(1 + ordered_list.len());
                    cur = d.time;
                }

                if membership[d.index as usize] != 0 {
                    membership[d.index as usize] = d.val;
                }
            }
            buf.push(cur);
            for &i in ordered_list.iter() {
                buf.push(membership[i as usize]);
                membership[i as usize] = SENTINEL_NODATA;
            }
            writer.write_all(&buf.into_raw_bytes());
            writer.flush();

            Ok(())
        } else {
            println!("ERROR: Mmap failed!");
            Err(io::Error::new(io::ErrorKind::Other, "Mmap failed!"))
        }
    }
}


