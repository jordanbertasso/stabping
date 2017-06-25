/*
 * Copyright 2017 icasdri
 *
 * This file is part of stabping. The original source code for stabping can be
 * found at <https://github.com/icasdri/stabping>. See COPYING for licensing
 * details.
 */

use std::mem;
use std::slice;
use std::cmp::Ordering;

/**
 * Trait for extracting the bytes (as a u8 slice) out of any Sized value.
 */
pub trait AsBytes {
    fn as_bytes(&self) -> &[u8];
}

impl<T> AsBytes for T where T: Sized {
    fn as_bytes(&self) -> &[u8] {
        let orig_ptr: *const T = self;
        let raw = orig_ptr as *const u8;
        let len = mem::size_of::<T>();
        unsafe {
            slice::from_raw_parts(raw, len)
        }
    }
}

impl AsBytes for () {
    fn as_bytes(&self) -> &[u8] {
        []
    }
}

pub trait PushAsBytes {
    fn push_as_bytes<T>(&mut self, val: T) where T: AsBytes;
}

impl PushBytes for Vec<u8> {
    fn push_as_bytes<T>(&mut self, val: T) where T: AsBytes {
        self.extend_from_slice(val.as_bytes());
    }
}

pub trait ToWire {
    fn to_wire(&self, wire: &mut Vec<u8>);
    fn wire_space_necessary(&self) -> usize;
}
