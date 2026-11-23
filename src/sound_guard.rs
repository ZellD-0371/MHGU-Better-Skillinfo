//! Accept the original entry or a standard Forge jump with an intact body.
pub fn valid(words:[u32;4],site:u32)->bool{
    const ORIGINAL:[u32;4]=[0xe1a02000,0xe59200c0,0xe3500000,0x012fff1e];
    crate::hook_guard::classify(&words,&ORIGINAL,site)!=crate::hook_guard::Entry::Incompatible
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test] fn entries(){
        let tail=[0xe3500000,0x012fff1e];
        assert!(valid([0xe1a02000,0xe59200c0,tail[0],tail[1]],0x54eeb78));
        assert!(valid([0xe51ff004,0x3401000,tail[0],tail[1]],0x54eeb78));
        for target in [0,0x3401001,0x54eeb78,0xfffffff0]{assert!(!valid([0xe51ff004,target,tail[0],tail[1]],0x54eeb78));}
        assert!(!valid([0xe51ff004,0x3401000,0,tail[1]],0x54eeb78));
    }
}
