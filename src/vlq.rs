// Copyright 2025 Divy Srivastava <dj.srivastava23@gmail.com>
//
// https://github.com/littledivy/resym/blob/59db8fce68e8cf786b22dd37979afec70ddc363d/src/vlq.rs

use std::io::{Error, ErrorKind};

fn base64_lut(byte: u8) -> u8 {
  match byte {
    b'A'..=b'Z' => byte - b'A',
    b'a'..=b'z' => byte - b'a' + 26,
    b'0'..=b'9' => byte - b'0' + 52,
    b'-' => 62,
    b'_' => 63,
    _ => 0,
  }
}

pub fn vlq_decode<I>(iter: &mut I) -> Result<i32, Error>
where
  I: Iterator<Item = u8>,
{
  fn read_byte<I>(iter: &mut I) -> Result<u8, Error>
  where
    I: Iterator<Item = u8>,
  {
    iter
      .next()
      .ok_or_else(|| Error::new(ErrorKind::UnexpectedEof, "unexpected EOF"))
  }

  let mut result = 0;
  let mut shift = 0;
  loop {
    let byte = read_byte(iter)?;
    let value = base64_lut(byte);
    result |= ((value & 0b11111) as i32) << shift;
    shift += 5;
    if value & 0b100000 == 0 {
      break;
    }
  }

  Ok(if result & 1 == 1 {
    -(result >> 1)
  } else {
    result >> 1
  })
}

pub(crate) fn vlq_encode(value: i32, writer: &mut Vec<u8>) {
  const VLQ_MAX_IN_BYTES: usize = 7;

  const BASE64_URL: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

  let mut vlq: u32 = if value >= 0 {
    (value as u32) << 1
  } else {
    ((-value as u32) << 1) | 1
  };

  for _ in 0..VLQ_MAX_IN_BYTES {
    let mut digit = vlq & 31;
    vlq >>= 5;

    if vlq != 0 {
      digit |= 32;
    }

    writer.push(BASE64_URL[digit as usize]);

    if vlq == 0 {
      return;
    }
  }
}
