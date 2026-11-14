//! One selected language in owned memory; no borrowed text escapes a UI frame.
use alloc::{vec::Vec,ffi::CString};
use core::{cell::UnsafeCell,ffi::{c_char,c_void}};
use forge::log::prelude::warn;
#[repr(C)]#[derive(Clone,Copy)] struct File(u32);
unsafe extern "C"{
 #[link_name="_ZN2nn2fs8OpenFileEPNS0_10FileHandleEPKci"] fn open(f:*mut File,p:*const c_char,m:i32)->u32;
 #[link_name="_ZN2nn2fs11GetFileSizeEPxNS0_10FileHandleE"] fn size(n:*mut i64,f:File)->u32;
 #[link_name="_ZN2nn2fs8ReadFileENS0_10FileHandleExPvj"] fn read(f:File,offset:i64,p:*mut c_void,n:usize)->u32;
 #[link_name="_ZN2nn2fs9CloseFileENS0_10FileHandleE"] fn close(f:File);
}
struct OpenFile(File);
impl Drop for OpenFile{fn drop(&mut self){unsafe{close(self.0)}}}
pub struct Data{bytes:Vec<u8>,count:usize,start:usize}
fn word(b:&[u8],at:usize)->Option<usize>{Some(u32::from_le_bytes(b.get(at..at.checked_add(4)?)?.try_into().ok()?) as usize)}
impl Data{
 pub fn parse(bytes:Vec<u8>)->Option<Self>{
  if bytes.get(..4)?!=b"BSI1"{return None}let count=word(&bytes,4)?;
  if count!=crate::generated::TEXT_COUNT{return None} // generated schema, prevents mismatched text IDs
  let start=8usize.checked_add(count.checked_add(1)?.checked_mul(4)?)?;
  let text=core::str::from_utf8(bytes.get(start..)?).ok()?;let mut previous=0;
  for i in 0..=count{let off=word(&bytes,8+i*4)?;if off<previous||off>text.len()||!text.is_char_boundary(off){return None}if i==0&&off!=0{return None}previous=off;}
  if previous!=text.len(){return None}Some(Self{bytes,count,start})
 }
 pub fn text(&self,id:u32)->&str{
  let id=id as usize;if id>=self.count{return ""}
  let a=word(&self.bytes,8+id*4).unwrap();let b=word(&self.bytes,12+id*4).unwrap();
  core::str::from_utf8(&self.bytes[self.start+a..self.start+b]).unwrap_or("")
 }
}
struct Cache{language:usize,data:Option<Data>}
struct UiCache(UnsafeCell<Cache>);unsafe impl Sync for UiCache{}
// Accessed only by ImGui callbacks on the UI thread. No language data in hooks.
static CACHE:UiCache=UiCache(UnsafeCell::new(Cache{language:usize::MAX,data:None}));
fn load(language:usize)->Option<Data>{
 let name=["ko","en","ja","zh-Hans","zh-Hant","de","fr","it","es"].get(language)?;
 let path=CString::new(alloc::format!("app:/nativeNX/plugins/better-skillinfo/languages/{}.bin",name)).ok()?;
 let mut handle=File(0);if unsafe{open(&mut handle,path.as_ptr(),1)}!=0{return None}
 let _file=OpenFile(handle);
 let mut len=0;if unsafe{size(&mut len,handle)}!=0||!(8..=512*1024).contains(&len){return None}
 let mut bytes=alloc::vec![0;len as usize];if unsafe{read(handle,0,bytes.as_mut_ptr().cast(),bytes.len())}!=0{return None}
 Data::parse(bytes)
}
pub fn select(language:usize){
 let cache=unsafe{&mut *CACHE.0.get()};if cache.language==language{return}
 crate::panel::clear_cache();
 cache.data=None; // drop the previous allocation before opening the next language
 cache.language=language;
 cache.data=load(language);
 if let Some(data)=&cache.data{crate::log_info!("[better-skillinfo] language {} loaded: {} bytes; previous language released",language,data.bytes.len());}
 else{warn!("[better-skillinfo] language {} missing or invalid; check mod language files",language);}
}
pub fn with<R>(f:impl FnOnce(Option<&Data>)->R)->R{f(unsafe{&*CACHE.0.get()}.data.as_ref())}
