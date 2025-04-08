use std::path::Path;
use crate::install::ArchiveFormat;

pub struct  SevenZipStruct<'a> {
  pub archive_format: ArchiveFormat,
  pub  archive_dir  : &'a Path 
  
}

impl  <'a> SevenZipStruct<'a >  {
  pub fn new(archive_format: ArchiveFormat, archive_dir: &'a Path) -> Self {
    Self {
      archive_format,
      archive_dir 
    }
  }
}