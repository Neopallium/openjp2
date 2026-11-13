// The copyright in this software is being made available under the 2-clauses
// BSD License, included below. This software may be subject to other third
// party and contributor rights, including patent rights, and no such rights
// are granted under this license.
//
// Copyright (c) 2002-2014, Universite catholique de Louvain (UCL), Belgium
// Copyright (c) 2002-2014, Professor Benoit Macq
// Copyright (c) 2001-2003, David Janssens
// Copyright (c) 2002-2003, Yannick Verschueren
// Copyright (c) 2003-2007, Francois-Olivier Devaux
// Copyright (c) 2003-2014, Antonin Descampe
// Copyright (c) 2005, Herve Drolon, FreeImage Team
// Copyright (c) 2008, 2011-2012, Centre National d'Etudes Spatiales (CNES), FR
// Copyright (c) 2012, CS Systemes d'Information, France
// All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions
// are met:
// 1. Redistributions of source code must retain the above copyright
// notice, this list of conditions and the following disclaimer.
// 2. Redistributions in binary form must reproduce the above copyright
// notice, this list of conditions and the following disclaimer in the
// documentation and/or other materials provided with the distribution.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS `AS IS'
// AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
// IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
// ARE DISCLAIMED.  IN NO EVENT SHALL THE COPYRIGHT OWNER OR CONTRIBUTORS BE
// LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
// CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
// SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
// INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
// CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
// ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
// POSSIBILITY OF SUCH DAMAGE.
//! Byte streams for the codecs: file/reader/callback inputs and
//! outputs, the in-memory cursor variant, and the C-ABI callbacks.

use std::io::{self as io, BufReader, BufWriter, Error as IoError, Read, Seek, SeekFrom, Write};

use std::{fs::File, path::Path};

use super::event::*;
use super::openjpeg::*;

pub(crate) trait ReadSeek: Read + Seek {}

impl<R: Read + Seek> ReadSeek for R {}

pub(crate) trait WriteSeek: Write + Seek {}

impl<R: Write + Seek> WriteSeek for R {}

/// Byte stream the codecs read from / write to (see the `stream` module
/// for the constructors).
pub struct Stream {
  pub(crate) m_inner: super::stream::StreamInner,
  pub(crate) m_stream_length: u64,
  pub(crate) m_byte_offset: i64,
  /// Sticky flag: a hard (non-EOF) I/O error occurred on a read (the C's
  /// `OPJ_STREAM_STATUS_ERROR`).  EOF short reads leave it clear.
  pub(crate) m_error: bool,
}
pub(crate) type stream_private = Stream;

pub(crate) enum StreamInner {
  Reader(BufReader<Box<dyn ReadSeek>>),
  Writer(BufWriter<Box<dyn WriteSeek>>),
  /// Owned in-memory bytes with a direct cursor (no BufReader indirection
  /// and no extra copy for the common decode-from-buffer case).
  Bytes(std::io::Cursor<std::sync::Arc<[u8]>>),
}

impl StreamInner {
  pub fn new_reader<R: Read + Seek + 'static>(capacity: usize, reader: R) -> Self {
    Self::Reader(BufReader::with_capacity(capacity, Box::new(reader)))
  }

  pub fn new_writer<R: Write + Seek + 'static>(capacity: usize, writer: R) -> Self {
    Self::Writer(BufWriter::with_capacity(capacity, Box::new(writer)))
  }

  pub fn from_bytes(bytes: Vec<u8>) -> Self {
    Self::Bytes(std::io::Cursor::new(std::sync::Arc::<[u8]>::from(bytes)))
  }

  pub fn seek_relative(&mut self, offset: i64) -> io::Result<()> {
    match self {
      StreamInner::Reader(reader) => reader.seek_relative(offset),
      StreamInner::Bytes(cursor) => {
        cursor.seek(SeekFrom::Current(offset))?;
        Ok(())
      }
      StreamInner::Writer(writer) => {
        writer.seek(SeekFrom::Current(offset))?;
        Ok(())
      }
    }
  }

  /// Whether the stream supports seeking.
  pub fn has_seek(&self) -> bool {
    match self {
      StreamInner::Reader(_) => true,
      StreamInner::Bytes(_) => true,
      StreamInner::Writer(_) => true,
    }
  }
}

impl Read for StreamInner {
  fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
    match self {
      StreamInner::Reader(reader) => reader.read(buf),
      StreamInner::Bytes(cursor) => cursor.read(buf),
      _ => Err(IoError::other("Can't read from output stream.")),
    }
  }
}

impl Write for StreamInner {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    match self {
      StreamInner::Writer(writer) => writer.write(buf),
      _ => Err(IoError::other("Can't write to input stream.")),
    }
  }

  fn flush(&mut self) -> io::Result<()> {
    match self {
      StreamInner::Writer(writer) => writer.flush(),
      _ => Err(IoError::other("Can't flush input stream.")),
    }
  }
}

impl Seek for StreamInner {
  fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
    match self {
      StreamInner::Reader(reader) => reader.seek(pos),
      StreamInner::Writer(writer) => writer.seek(pos),
      StreamInner::Bytes(cursor) => cursor.seek(pos),
    }
  }
}

impl Stream {
  /// Opens a file as an input or output stream.
  pub fn new_file<P: AsRef<Path>>(path: P, buffer_size: usize, is_input: bool) -> io::Result<Self> {
    if is_input {
      let file = File::open(&path)?;
      let m_stream_length = file.metadata().map(|m| m.len())?;
      Ok(Self::new_input(buffer_size, file, m_stream_length))
    } else {
      let file = File::create(&path)?;
      Ok(Self::new_output(buffer_size, file))
    }
  }

  /// Wrap a `Read + Seek` source as an input stream. The capi crate uses this
  /// to plug its C-callback adapter in without the core knowing about C.
  pub fn new_input<R: Read + Seek + 'static>(
    buffer_size: usize,
    reader: R,
    stream_length: u64,
  ) -> Self {
    Self {
      m_inner: super::stream::StreamInner::new_reader(buffer_size, reader),
      m_stream_length: stream_length,
      m_byte_offset: 0,
      m_error: false,
    }
  }

  /// Wrap an owned in-memory buffer as an input stream: the bytes move
  /// into a direct cursor (no `BufReader` indirection, no copy) — the
  /// common decode-from-buffer shape.
  pub fn from_bytes(bytes: Vec<u8>) -> Self {
    let len = bytes.len() as u64;
    Self {
      m_inner: super::stream::StreamInner::from_bytes(bytes),
      m_stream_length: len,
      m_byte_offset: 0,
      m_error: false,
    }
  }

  /// For in-memory streams: share the underlying buffer without copying
  /// (the parallel decode's snapshot becomes zero-copy; B.1.6).  Returns
  /// `None` for reader/file streams.
  pub(crate) fn shared_bytes(&self) -> Option<std::sync::Arc<[u8]>> {
    match &self.m_inner {
      super::stream::StreamInner::Bytes(cursor) => Some(cursor.get_ref().clone()),
      _ => None,
    }
  }

  /// Wrap a `Write + Seek` source as an output stream.
  pub fn new_output<R: Write + Seek + 'static>(buffer_size: usize, writer: R) -> Self {
    Self {
      m_inner: super::stream::StreamInner::new_writer(buffer_size, writer),
      m_stream_length: 0,
      m_byte_offset: 0,
      m_error: false,
    }
  }

  /// Whether the stream is readable (an input stream).
  pub fn is_input(&self) -> bool {
    matches!(self.m_inner, StreamInner::Reader(_) | StreamInner::Bytes(_))
  }

  /// Overrides the logical stream length (input streams).
  pub fn set_stream_length(&mut self, len: u64) {
    self.m_stream_length = len;
  }

  /// Reads as many bytes as `buf` holds (short only on error/EOF).
  pub fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
    let len = buf.len();
    match self.m_inner.read_exact(buf) {
      Ok(_) => {
        self.m_byte_offset += len as i64;
        Ok(len)
      }
      Err(err) => {
        /* read_exact may have delivered a partial prefix before hitting
        EOF; those bytes are already in `buf`.  Count them through the
        reader's position delta so they are not silently lost (a plain
        follow-up read would start after them and return 0). */
        let pos = match self.m_inner.stream_position() {
          Ok(pos) => pos,
          Err(pos_err) => {
            self.m_error = true;
            log::trace!("Failed to read from stream: {pos_err}");
            return Err(pos_err);
          }
        };
        let consumed = (pos as i64).saturating_sub(self.m_byte_offset).max(0) as usize;
        let nb = consumed.min(len);
        self.m_byte_offset += nb as i64;
        /* Hard I/O errors are distinguished from EOF for embedders with
        custom readers (the C's ERROR status; cio.c:327-334). */
        if err.kind() != io::ErrorKind::UnexpectedEof {
          self.m_error = true;
          log::trace!("Stream read error: {err}");
        }
        Ok(nb)
      }
    }
  }

  /// Writes all of `buf` (short only on error).
  pub fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    let len = buf.len();
    log::trace!("-- write({len}), offset={}", self.m_byte_offset);
    match self.m_inner.write_all(buf) {
      Ok(_) => {
        self.m_byte_offset += len as i64;
        Ok(len)
      }
      Err(err) => {
        log::trace!("Failed to write to stream: {err}");
        Err(err)
      }
    }
  }

  /// Flushes buffered output.
  pub fn flush(&mut self) -> io::Result<()> {
    log::trace!("-- flush(), offset={}", self.m_byte_offset);
    self.m_inner.flush()
  }

  /// Whether a hard I/O error occurred on a read (sticky; EOF does not
  /// set it).  Mirror of the C stream's ERROR status.
  pub fn is_error(&self) -> bool {
    self.m_error
  }

  /// The current byte position.
  pub fn tell(&self) -> i64 {
    log::trace!("-- tell() = {}", self.m_byte_offset);
    self.m_byte_offset
  }

  /// Bytes remaining before the declared stream end.
  pub fn get_bytes_left(&self) -> i64 {
    log::trace!("-- byte_left(), offset={}", self.m_byte_offset);
    /* The C's stream_get_number_byte_left asserts
    m_user_data_length >= m_byte_offset only in debug builds (NDEBUG strips
    it) and unknown-length streams (length 0) return 0.  Mirror the
    behaviour WITHOUT the asserts: the TLM-driven decode can seek to the
    end of the last tile part and probe 2 bytes past EOF, which must read
    as a clean short-read, not a panic. */
    if self.m_stream_length != 0 {
      log::trace!(
        "-- get_number_byte_left() = {}",
        (self.m_stream_length as i64) - self.m_byte_offset
      );
      (self.m_stream_length as i64) - self.m_byte_offset
    } else {
      log::trace!("-- get_number_byte_left() = 0");
      0i64
    }
  }

  /// Moves the position forward by `count` bytes.
  pub fn skip(&mut self, count: i64) -> io::Result<i64> {
    self.skip_inner(count, None)
  }

  /// [`Self::skip`] with the C's `EVT_INFO "Stream reached its end !"`
  /// routed through the caller's event manager (the codec call sites).
  pub(crate) fn skip_events(&mut self, count: i64, p_manager: &EventMgr) -> io::Result<i64> {
    self.skip_inner(count, Some(p_manager))
  }

  fn skip_inner(&mut self, count: i64, p_manager: Option<&EventMgr>) -> io::Result<i64> {
    if count < 0 {
      log::trace!("Can't skip with count < 0: {count}");
      return Err(io::Error::other(format!(
        "Can't skip with count < 0: {count}"
      )));
    }
    /* Input streams mirror the C's opj_stream_read_skip: clamp at the
    declared stream length, seek to EOF, and return the partial count (or
    -1 when nothing could be skipped).  Output streams use the C's write
    skip, which has no length clamp (the JP2 header writes placeholders
    beyond the current end and seek back later). */
    if !matches!(self.m_inner, StreamInner::Writer(_)) {
      let remaining = (self.m_stream_length.min(i64::MAX as u64) as i64)
        .saturating_sub(self.m_byte_offset)
        .max(0);
      if count > remaining {
        match p_manager {
          Some(manager) => {
            event_msg!(manager, EVT_INFO, "Stream reached its end !\n");
          }
          None => log::info!("Stream reached its end !"),
        }
        if remaining > 0 {
          self.m_inner.seek_relative(remaining)?;
          self.m_byte_offset = self.m_byte_offset.saturating_add(remaining);
          return Ok(remaining);
        }
        return Err(io::Error::other("Skip pass the end of the stream"));
      }
    }
    let res = self
      .m_inner
      .seek_relative(count)
      .map(|_| (self.m_byte_offset + count) as u64);
    match res {
      Ok(offset) => {
        self.m_byte_offset = offset as i64;
        log::trace!("-- skip({count}) = {offset}");
        Ok(count)
      }
      Err(err) => {
        log::trace!("Failed to skip stream: {err}");
        Err(err)
      }
    }
  }

  /// Moves the position to an absolute offset.
  pub fn seek(&mut self, offset: i64) -> io::Result<()> {
    let res = self.m_inner.seek_relative(offset - self.m_byte_offset);
    match res {
      Ok(_) => {
        self.m_byte_offset = offset;
        log::trace!("-- seek({offset}) = Ok");
        Ok(())
      }
      Err(err) => {
        log::trace!("Failed to seek stream: {err}");
        Err(err)
      }
    }
  }

  /// Whether the stream supports seeking.
  pub fn has_seek(&self) -> bool {
    self.m_inner.has_seek()
  }
}

impl Read for Stream {
  fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
    self.read(buf)
  }
}

impl Write for Stream {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    self.write(buf)
  }

  fn flush(&mut self) -> io::Result<()> {
    self.flush()
  }
}

pub(crate) fn stream_read_data(
  p_stream: &mut Stream,
  buf: &mut [u8],
  mut _p_event_mgr: &mut EventMgr,
) -> usize {
  match p_stream.read(buf) {
    Ok(nb) => nb,
    Err(err) => {
      log::trace!("Failed to read from stream: {err}");
      -1i32 as usize
    }
  }
}

pub(crate) fn stream_write_data(
  p_stream: &mut Stream,
  buf: &[u8],
  mut _p_event_mgr: &mut EventMgr,
) -> usize {
  log::trace!("-- write({}), offset={}", buf.len(), p_stream.m_byte_offset);
  match p_stream.write(buf) {
    Ok(nb) => nb,
    Err(err) => {
      log::trace!("Failed to write to stream: {err}");
      -1i32 as usize
    }
  }
}

pub(crate) fn stream_flush(p_stream: &mut Stream, mut _p_event_mgr: &mut EventMgr) -> OpResult {
  match p_stream.flush() {
    Ok(_) => Ok(()),
    Err(err) => {
      log::trace!("Failed to flush stream: {err}");
      Err(())
    }
  }
}

pub(crate) fn stream_tell(p_stream: &Stream) -> i64 {
  p_stream.tell()
}

pub(crate) fn stream_get_number_byte_left(p_stream: &Stream) -> i64 {
  p_stream.get_bytes_left()
}

pub(crate) fn stream_skip(
  p_stream: &mut Stream,
  mut p_size: i64,
  p_event_mgr: &mut EventMgr,
) -> i64 {
  match p_stream.skip_events(p_size, p_event_mgr) {
    Ok(offset) => {
      log::trace!("-- skip({p_size}) = {}", offset);
      offset
    }
    Err(err) => {
      log::trace!("Failed to skip stream: {err}");
      -1i32 as _
    }
  }
}

pub(crate) fn stream_seek(
  p_stream: &mut Stream,
  mut p_size: i64,
  mut _p_event_mgr: &mut EventMgr,
) -> OpResult {
  match p_stream.seek(p_size) {
    Ok(_) => {
      log::trace!("-- seek({p_size}) = {}", 1);
      Ok(())
    }
    Err(err) => {
      log::trace!("Failed to seek stream: {err}");
      Err(())
    }
  }
}

pub(crate) fn stream_has_seek(p_stream: &Stream) -> bool {
  p_stream.has_seek() as _
}

/// `Write + Seek` adapter collecting into a shared `Vec<u8>`; the encoder
/// seeks back to patch marker lengths, so the position must be honoured.
pub(crate) struct SharedVecWriter {
  pub(crate) buf: std::rc::Rc<std::cell::RefCell<Vec<u8>>>,
  pub(crate) pos: u64,
}

impl std::io::Write for SharedVecWriter {
  fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
    let mut buf = self.buf.borrow_mut();
    let start = self.pos as usize;
    if start + data.len() > buf.len() {
      buf.resize(start + data.len(), 0);
    }
    buf[start..start + data.len()].copy_from_slice(data);
    self.pos += data.len() as u64;
    Ok(data.len())
  }

  fn flush(&mut self) -> std::io::Result<()> {
    Ok(())
  }
}

impl std::io::Seek for SharedVecWriter {
  fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
    /* Saturate the position: a negative relative seek must clamp to the
    buffer start, never wrap to a huge u64 (a later write would then try to
    resize the buffer past the address space - open-findings OF-16). */
    let new_pos = match pos {
      std::io::SeekFrom::Start(p) => p,
      std::io::SeekFrom::Current(d) => self.pos.saturating_add_signed(d),
      std::io::SeekFrom::End(d) => (self.buf.borrow().len() as u64).saturating_add_signed(d),
    };
    self.pos = new_pos;
    Ok(new_pos)
  }
}
