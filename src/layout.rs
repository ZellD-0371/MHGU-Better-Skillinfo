//! Shared logical viewport coordinates; no emulator-specific DPI multiplier.
pub fn viewport_size(viewport:(f32,f32),fallback:(f32,f32))->Option<(f32,f32)>{
    [viewport,fallback].into_iter().find(|&(x,y)|x.is_finite()&&y.is_finite()&&x>0.&&y>0.)
}
