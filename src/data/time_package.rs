use std::collections::{btree_set, BTreeSet};

use data::{DataElement, ToWire, PushAsBytes};
use manager::Feed;
use workers::Kind;

pub struct TimePackage {
    pub kind: Kind,
    feed: Feed,
    time: Option<u32>,
    set: BTreeSet<DataElement>,
}

#[derive(Debug)]
pub enum TimePackageError {
    IncompatibleTimes,
    DuplicateEntryForIndex,
}
use self::TimePackageError as TPE;

impl TimePackage {
    pub fn new(kind: Kind, feed: Feed) -> Self {
        TimePackage {
            kind: kind,
            feed: feed,
            time: None,
            set: BTreeSet::new()
        }
    }

    pub fn insert(&mut self, d: DataElement) -> Result<(), TimePackageError> {
        macro_rules! perform_insert {
            () => {
                if self.set.insert(d) {
                    Ok(())
                } else {
                    Err(TPE::DuplicateEntryForIndex)
                }
            }
        };

        match self.time {
            None => perform_insert!(),
            Some(t) if d.time == t => perform_insert!(),
            _ => Err(TPE::IncompatibleTimes)
        }
    }

    pub fn iter(&self) -> btree_set::Iter<DataElement> {
        self.set.iter()
    }
}

impl ToWire for TimePackage {
    fn wire_space_necessary(&self) -> usize {
        let len = self.set.len();
        self.set.iter().next().map(|d| len * d.wire_space_necessary()).unwrap_or(0)
    }

    fn to_wire(&self, wire: &mut Vec<u8>) {
        match self.time {
            None => (),
            Some(t) => {
                wire.push_as_bytes(self.kind.id());
                wire.push_as_bytes(t);
                for d in self.set.iter() {
                    wire.push_as_bytes(d.val);
                    wire.push_as_bytes(d.sd);
                }
            }
        }
    }
}
