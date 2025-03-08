use std::io::Cursor;

use symbolic::{
  common::ByteView,
  debuginfo::Archive,
  symcache::{SymCache, SymCacheConverter},
};
use symbolic_demangle::{Demangle, DemangleOptions};

use crate::vlq;

pub fn create_symcache(debug_file: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
  let byteview = ByteView::from_slice(debug_file);
  let fat_obj = Archive::parse(&byteview)?;
  let objects: Result<Vec<_>, _> = fat_obj.objects().collect();
  let objects = objects?;
  if objects.len() != 1 {
    anyhow::bail!("Fat archives are not supported currently");
  }
  let object = &objects[0];
  let mut converter = SymCacheConverter::new();
  converter.process_object(object)?;

  let mut result = Vec::new();
  converter.serialize(&mut Cursor::new(&mut result))?;
  Ok(result)
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameLocation {
  pub demangled_name: String,
  pub name: String,
  pub language: String,
  pub full_path: String,
  pub line: u32,
}

pub fn symbolicate_addrs<B: Iterator<Item = u8>>(
  addrs: &mut B,
  symcache: &[u8],
) -> Result<Vec<Vec<FrameLocation>>, anyhow::Error> {
  let symcache = SymCache::parse(symcache)?;
  let mut out = Vec::new();
  loop {
    let addr = match vlq::vlq_decode(addrs) {
      Ok(addr) => addr,
      Err(_) => break,
    };

    let syms = symcache.lookup(addr as u64).collect::<Vec<_>>();
    out.push(
      syms
        .into_iter()
        .map(|sym| FrameLocation {
          demangled_name: sym
            .function()
            .name_for_demangling()
            .try_demangle(DemangleOptions::name_only())
            .into_owned(),
          name: sym.function().name().into(),
          language: sym.function().language().to_string(),
          full_path: sym
            .file()
            .map(|file| file.full_path())
            .unwrap_or_else(|| "<unknown file>".into()),
          line: sym.line(),
        })
        .collect(),
    );
  }

  Ok(out)
}
