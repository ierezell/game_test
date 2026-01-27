mod tests;


fn main() {
    #[cfg(target_family = "wasm")]
    launcher::wasm::run();

    #[cfg(not(target_family = "wasm"))]
    launcher::native::run();
}
