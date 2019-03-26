use crate::file::File;

pub struct FileList {
  inner: web_sys::FileList,
  length: usize,
}

impl FileList {
  pub fn new(input: &web_sys::HtmlInputElement) -> Option<Self> {
    input.files().map(Self::from_raw)
  }

  pub fn from_raw(inner: web_sys::FileList) -> Self {
    let length = inner.length() as usize;
    FileList { inner, length }
  }

  pub fn get(&self, index: usize) -> Option<File> {
    self.inner.get(index as u32).map(File::from_raw)
  }

  pub fn len(&self) -> usize {
    self.length
  }

  pub fn iter(&self) -> FileListIter {
    FileListIter {
      file_list: self,
      current: 0,
    }
  }

  pub fn into_vec(self) -> Vec<File> {
    self.iter().collect()
  }
}

pub struct FileListIter<'a> {
  file_list: &'a FileList,
  current: usize,
}

impl<'a> Iterator for FileListIter<'a> {
  type Item = File;

  fn next(&mut self) -> Option<Self::Item> {
    if self.current >= self.file_list.len() {
      return None;
    }
    let file = self.file_list.get(self.current);
    self.current += 1;

    assert!(file.is_some());

    file
  }
}
