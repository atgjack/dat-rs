//! Automatically generated rust module for 'schema.proto' file

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(unused_imports)]
#![allow(unknown_lints)]
#![allow(clippy)]
#![cfg_attr(rustfmt, rustfmt_skip)]


use std::io::Write;
use std::borrow::Cow;
use quick_protobuf::{MessageRead, MessageWrite, BytesReader, Writer, Result};
use quick_protobuf::sizeofs::*;
use super::*;

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Feed<'a> {
    pub discoveryKey: Cow<'a, [u8]>,
    pub nonce: Option<Cow<'a, [u8]>>,
}

impl<'a> MessageRead<'a> for Feed<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.discoveryKey = r.read_bytes(bytes).map(Cow::Borrowed)?,
                Ok(18) => msg.nonce = Some(r.read_bytes(bytes).map(Cow::Borrowed)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for Feed<'a> {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_len((&self.discoveryKey).len())
        + self.nonce.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(10, |w| w.write_bytes(&**&self.discoveryKey))?;
        if let Some(ref s) =self.nonce { w.write_with_tag(18, |w| w.write_bytes(&**s))?; }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Handshake<'a> {
    pub id: Option<Cow<'a, [u8]>>,
    pub live: Option<bool>,
    pub userData: Option<Cow<'a, [u8]>>,
    pub extensions: Vec<Cow<'a, str>>,
    pub ack: Option<bool>,
}

impl<'a> MessageRead<'a> for Handshake<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.id = Some(r.read_bytes(bytes).map(Cow::Borrowed)?),
                Ok(16) => msg.live = Some(r.read_bool(bytes)?),
                Ok(26) => msg.userData = Some(r.read_bytes(bytes).map(Cow::Borrowed)?),
                Ok(34) => msg.extensions.push(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(40) => msg.ack = Some(r.read_bool(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for Handshake<'a> {
    fn get_size(&self) -> usize {
        0
        + self.id.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.live.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.userData.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.extensions.iter().map(|s| 1 + sizeof_len((s).len())).sum::<usize>()
        + self.ack.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) =self.id { w.write_with_tag(10, |w| w.write_bytes(&**s))?; }
        if let Some(ref s) =self.live { w.write_with_tag(16, |w| w.write_bool(*s))?; }
        if let Some(ref s) =self.userData { w.write_with_tag(26, |w| w.write_bytes(&**s))?; }
        for s in &self.extensions { w.write_with_tag(34, |w| w.write_string(&**s))?; }
        if let Some(ref s) =self.ack { w.write_with_tag(40, |w| w.write_bool(*s))?; }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Info {
    pub uploading: Option<bool>,
    pub downloading: Option<bool>,
}

impl<'a> MessageRead<'a> for Info {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.uploading = Some(r.read_bool(bytes)?),
                Ok(16) => msg.downloading = Some(r.read_bool(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for Info {
    fn get_size(&self) -> usize {
        0
        + self.uploading.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.downloading.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) =self.uploading { w.write_with_tag(8, |w| w.write_bool(*s))?; }
        if let Some(ref s) =self.downloading { w.write_with_tag(16, |w| w.write_bool(*s))?; }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Have<'a> {
    pub start: u64,
    pub length: u64,
    pub bitfield: Option<Cow<'a, [u8]>>,
}

impl<'a> MessageRead<'a> for Have<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Have {
            length: 1u64,
            ..Self::default()
        };
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.start = r.read_uint64(bytes)?,
                Ok(16) => msg.length = r.read_uint64(bytes)?,
                Ok(26) => msg.bitfield = Some(r.read_bytes(bytes).map(Cow::Borrowed)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for Have<'a> {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_varint(*(&self.start) as u64)
        + if self.length == 1u64 { 0 } else { 1 + sizeof_varint(*(&self.length) as u64) }
        + self.bitfield.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(8, |w| w.write_uint64(*&self.start))?;
        if self.length != 1u64 { w.write_with_tag(16, |w| w.write_uint64(*&self.length))?; }
        if let Some(ref s) =self.bitfield { w.write_with_tag(26, |w| w.write_bytes(&**s))?; }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Unhave {
    pub start: u64,
    pub length: u64,
}

impl<'a> MessageRead<'a> for Unhave {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Unhave {
            length: 1u64,
            ..Self::default()
        };
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.start = r.read_uint64(bytes)?,
                Ok(16) => msg.length = r.read_uint64(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for Unhave {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_varint(*(&self.start) as u64)
        + if self.length == 1u64 { 0 } else { 1 + sizeof_varint(*(&self.length) as u64) }
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(8, |w| w.write_uint64(*&self.start))?;
        if self.length != 1u64 { w.write_with_tag(16, |w| w.write_uint64(*&self.length))?; }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Want {
    pub start: u64,
    pub length: Option<u64>,
}

impl<'a> MessageRead<'a> for Want {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.start = r.read_uint64(bytes)?,
                Ok(16) => msg.length = Some(r.read_uint64(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for Want {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_varint(*(&self.start) as u64)
        + self.length.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(8, |w| w.write_uint64(*&self.start))?;
        if let Some(ref s) =self.length { w.write_with_tag(16, |w| w.write_uint64(*s))?; }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Unwant {
    pub start: u64,
    pub length: Option<u64>,
}

impl<'a> MessageRead<'a> for Unwant {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.start = r.read_uint64(bytes)?,
                Ok(16) => msg.length = Some(r.read_uint64(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for Unwant {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_varint(*(&self.start) as u64)
        + self.length.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(8, |w| w.write_uint64(*&self.start))?;
        if let Some(ref s) =self.length { w.write_with_tag(16, |w| w.write_uint64(*s))?; }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Request {
    pub index: u64,
    pub bytes: Option<u64>,
    pub hash: Option<bool>,
    pub nodes: Option<u64>,
}

impl<'a> MessageRead<'a> for Request {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.index = r.read_uint64(bytes)?,
                Ok(16) => msg.bytes = Some(r.read_uint64(bytes)?),
                Ok(24) => msg.hash = Some(r.read_bool(bytes)?),
                Ok(32) => msg.nodes = Some(r.read_uint64(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for Request {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_varint(*(&self.index) as u64)
        + self.bytes.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.hash.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.nodes.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(8, |w| w.write_uint64(*&self.index))?;
        if let Some(ref s) =self.bytes { w.write_with_tag(16, |w| w.write_uint64(*s))?; }
        if let Some(ref s) =self.hash { w.write_with_tag(24, |w| w.write_bool(*s))?; }
        if let Some(ref s) =self.nodes { w.write_with_tag(32, |w| w.write_uint64(*s))?; }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Cancel {
    pub index: u64,
    pub bytes: Option<u64>,
    pub hash: Option<bool>,
}

impl<'a> MessageRead<'a> for Cancel {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.index = r.read_uint64(bytes)?,
                Ok(16) => msg.bytes = Some(r.read_uint64(bytes)?),
                Ok(24) => msg.hash = Some(r.read_bool(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for Cancel {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_varint(*(&self.index) as u64)
        + self.bytes.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
        + self.hash.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(8, |w| w.write_uint64(*&self.index))?;
        if let Some(ref s) =self.bytes { w.write_with_tag(16, |w| w.write_uint64(*s))?; }
        if let Some(ref s) =self.hash { w.write_with_tag(24, |w| w.write_bool(*s))?; }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Data<'a> {
    pub index: u64,
    pub value: Option<Cow<'a, [u8]>>,
    pub nodes: Vec<mod_Data::Node<'a>>,
    pub signature: Option<Cow<'a, [u8]>>,
}

impl<'a> MessageRead<'a> for Data<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.index = r.read_uint64(bytes)?,
                Ok(18) => msg.value = Some(r.read_bytes(bytes).map(Cow::Borrowed)?),
                Ok(26) => msg.nodes.push(r.read_message::<mod_Data::Node>(bytes)?),
                Ok(34) => msg.signature = Some(r.read_bytes(bytes).map(Cow::Borrowed)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for Data<'a> {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_varint(*(&self.index) as u64)
        + self.value.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
        + self.nodes.iter().map(|s| 1 + sizeof_len((s).get_size())).sum::<usize>()
        + self.signature.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(8, |w| w.write_uint64(*&self.index))?;
        if let Some(ref s) =self.value { w.write_with_tag(18, |w| w.write_bytes(&**s))?; }
        for s in &self.nodes { w.write_with_tag(26, |w| w.write_message(s))?; }
        if let Some(ref s) =self.signature { w.write_with_tag(34, |w| w.write_bytes(&**s))?; }
        Ok(())
    }
}

pub mod mod_Data {

use std::borrow::Cow;
use super::*;

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Node<'a> {
    pub index: u64,
    pub hash: Cow<'a, [u8]>,
    pub size: u64,
}

impl<'a> MessageRead<'a> for Node<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.index = r.read_uint64(bytes)?,
                Ok(18) => msg.hash = r.read_bytes(bytes).map(Cow::Borrowed)?,
                Ok(24) => msg.size = r.read_uint64(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for Node<'a> {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_varint(*(&self.index) as u64)
        + 1 + sizeof_len((&self.hash).len())
        + 1 + sizeof_varint(*(&self.size) as u64)
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(8, |w| w.write_uint64(*&self.index))?;
        w.write_with_tag(18, |w| w.write_bytes(&**&self.hash))?;
        w.write_with_tag(24, |w| w.write_uint64(*&self.size))?;
        Ok(())
    }
}

}

