# rs-init

This crate generates rust code that calls functions with the `#[init]` attribute in the correct order.
`#[init]` macro is provided by the `rs-init-macro` crate.
Example usage:
```rust
use rs_init_macro::init;

#[init(stage = 0)]
fn init0() {
    println!("init0");
}

#[init(stage = 1)]
fn init1() {
    println!("init1");
}

fn main() {
    generated_init();
}
```

Build.rs:

```rust
fn main() {
    rs_init::default_setup();
}
```

`#[init]` macro can be used on function in any module, but the `rs-init` crate must be able to find the module.
This can be done by adding `pub(crate)` to the module declaration.

You probably would not use this crate by itself, but rather to create some sort of framework and other macros that use it.

License: MIT
