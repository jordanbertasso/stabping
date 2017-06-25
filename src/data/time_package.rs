use std::collections::BTreeSet;

use data::PushAsBytes;
use data::ToWire;

pub struct TimePackage {
    set: BTreeSet<DataElement>,
}

pub enum TimePackageError {
    IncompatibleTimes,
    DuplicateEntryForIndex,
}
use TimePackageError as TPE;

impl TimePackage {
    fn new() -> Self {
        TimePackage {
            set: BTreeSet::new()
        }
    }

    fn first(&self) -> Option<&DataElement> {
        self.set.iter().next()
    }

    fn insert(&mut self, v: DataElement) -> Result<(), TimePackageError> {
        match self.first() {
            Some(d) if v.time == d.time | None => {
                if self.set.insert(d) {
                    Ok(())
                } else {
                    Err(TPE::DuplicateEntryForIndex)
                }
            }
            _ => Err(TPE::IncompatibleTimes)
        }
    }
}

impl ToWire for TimePackage {
    fn wire_space_necessary(&self) -> usize {
        let len = self.set.len();
        self.first().map(|d| len * d.wire_space_necessary()).unwrap_or(0)
    }

    fn to_wire(&self, wire: &mut Vec<u8>) {
        wire.push_as_bytes(self.first().map(|d| d.time).unwrap_or(()));
        for d in self.set.iter() {
            wire.push_as_bytes(d.val);
            wire.push_as_bytes(d.sd);
        }
    }
}

