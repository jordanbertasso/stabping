use std::mem;
use std::cmp::Ordering;

use data::AsBytes;
use data::ToWire;

/**
 * Representation of data elements on-disk.
 */
#[repr(C, packed)]
pub struct DataElement {
    time: u32,
    index: u32,
    val: f32,  // the raw or averaged value
    sd: f32,  // the standard deviation (or NaN if value is raw)
}

impl ToWire for DataElement {
    fn wire_space_necessary(&self) -> usize {
        mem::size_of_val(&self.val) + mem::size_of_val(&self.sd)
    }

    fn to_wire(&self, wire: &mut Vec<u8>) {
        wire.extend_from_slice(self.val.as_bytes());
        wire.extend_from_slice(self.sd.as_bytes());
    }
}

/*
 * Ord (and thus Eq, PartialEq, and PartialOrd) implementation for DataElement
 * over their indices (via get_index()) so that they can be put in BTreeSets.
 */
impl Ord for DataElement {
    fn cmp(&self, other: &DiscreteDataOnDisk) -> Ordering {
        self.index.cmp(&other.index)
    }
}

impl PartialOrd for DataElement {
    fn partial_cmp(&self, other: &DataElement) -> Option<Ordering> {
        self.index.partial_cmp(&other.index)
    }
}

impl PartialEq for DataElement {
    fn eq(&self, other: &DataElement) -> bool {
        self.index == other.index
    }
}

impl Eq for DataElement {}


