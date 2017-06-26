use std::mem;
use std::cmp::Ordering;

use data::PushAsBytes;
use data::ToWire;

/**
 * Representation of data elements on-disk.
 */
#[repr(C, packed)]
pub struct DataElement {
    pub time: u32,
    pub index: u32,
    pub val: f32,  // the raw or averaged value
    pub sd: f32,  // the standard deviation (or NaN if value is raw)
}

impl ToWire for DataElement {
    fn wire_space_necessary(&self) -> usize {
        mem::size_of_val(&self.val) + mem::size_of_val(&self.sd)
    }

    fn to_wire(&self, wire: &mut Vec<u8>) {
        wire.push_as_bytes(self.val);
        wire.push_as_bytes(self.sd);
    }
}

/*
 * Ord (and thus Eq, PartialEq, and PartialOrd) implementation for DataElement
 * over their indices (via get_index()) so that they can be put in BTreeSets.
 */
impl Ord for DataElement {
    fn cmp(&self, other: &DataElement) -> Ordering {
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


