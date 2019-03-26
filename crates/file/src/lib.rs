use std::rc::Rc;

use futures::Future;
use wasm_bindgen::JsCast;

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

pub struct File {
  inner: web_sys::File,
}

impl File {
  fn from_raw(inner: web_sys::File) -> File {
    File { inner }
  }
}

pub enum MimeType {
  Unknown,
  ApplicationJson,
}
pub trait Blob {
  fn size(&self) -> usize;
  fn mime_type(&self) -> MimeType;
}

pub trait RawBlob {
  fn raw(&self) -> &web_sys::Blob;
}

pub struct DataBlob {
  inner: web_sys::Blob,
}

impl DataBlob {
  pub fn new(content: &str) -> DataBlob {
    let parts = js_sys::Array::of1(&wasm_bindgen::JsValue::from_str(content));
    let inner = web_sys::Blob::new_with_str_sequence(&parts).unwrap();
    DataBlob { inner }
  }
}

impl Blob for DataBlob {
  fn size(&self) -> usize {
    self.inner.size() as usize
  }

  fn mime_type(&self) -> MimeType {
    match self.inner.type_().as_ref() {
      "application/json" => MimeType::ApplicationJson,
      _ => MimeType::Unknown,
    }
  }
}

impl RawBlob for DataBlob {
  fn raw(&self) -> &web_sys::Blob {
    &self.inner
  }
}

impl Blob for File {
  fn size(&self) -> usize {
    self.inner.size() as usize
  }

  fn mime_type(&self) -> MimeType {
    match self.inner.type_().as_ref() {
      "application/json" => MimeType::ApplicationJson,
      _ => MimeType::Unknown,
    }
  }
}

pub struct FileReader {
  inner: web_sys::FileReader,
}

impl FileReader {
  pub fn new() -> FileReader {
    FileReader {
      inner: web_sys::FileReader::new().unwrap(),
    }
  }

  pub fn read_as_string(
    self,
    blob: &(impl Blob + RawBlob),
  ) -> impl futures::Future<Item = String, Error = ()> {
    let (tx, rx) = futures::sync::oneshot::channel();
    let reader = Rc::new(self.inner);
    let cloned_reader = reader.clone();
    let cb = wasm_bindgen::closure::Closure::once(move || {
      let _ = cloned_reader.result().map(|r| {
        let _ = tx.send(r.as_string().unwrap());
      });
    });
    let reader = reader.clone();
    let function = cb.as_ref().dyn_ref().unwrap();
    reader.set_onload(Some(function));
    reader.read_as_text(&blob.raw()).unwrap();
    rx.map_err(|_| ())
  }
}
