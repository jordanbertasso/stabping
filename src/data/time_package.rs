use std::collections::BTreeSet;

use data::{DataElement, ToWire, PushAsBytes};
use workers::Kind;

pub struct TimePackage {
    pub kind: Kind,
    set: BTreeSet<DataElement>,
}

pub enum TimePackageError {
    IncompatibleTimes,
    DuplicateEntryForIndex,
}
use self::TimePackageError as TPE;

impl TimePackage {
    pub fn new(kind: Kind) -> Self {
        TimePackage {
            kind: kind,
            set: BTreeSet::new()
        }
    }

    fn first(&self) -> Option<&DataElement> {
        self.set.iter().next()
    }

    pub fn insert(&mut self, v: DataElement) -> Result<(), TimePackageError> {
        let perform_insert = || {
            if self.set.insert(v) {
                Ok(())
            } else {
                Err(TPE::DuplicateEntryForIndex)
            }
        };

        match self.first() {
            Some(d) if v.time == d.time => perform_insert(),
            None => perform_insert(),
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
        match self.first() {
            Some(d) => wire.push_as_bytes(d.time),
            None => ()
        };

        for d in self.set.iter() {
            wire.push_as_bytes(d.val);
            wire.push_as_bytes(d.sd);
        }
    }
}

