// Run: rustc --edition=2024 --test tools/check_hook_guards.rs -o target/check_hook_guards.exe
extern crate alloc;
#[path="../src/hook_guard.rs"] mod hook_guard;
#[path="../src/menu_input.rs"] mod menu_input;
#[path="../src/sound_guard.rs"] mod sound_guard;
#[path="../src/katsmithy_model.rs"] mod katsmithy_model;
