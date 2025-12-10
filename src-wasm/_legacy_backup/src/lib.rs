// Petit export pour WASM (fonction addition) + tests natifs

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn ga_add(a: i32, b: i32) -> i32 {
    a + b
}

// Permet de tester en natif (x86_64) sans target wasm
#[cfg(not(target_arch = "wasm32"))]
pub fn ga_add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn adds() {
        assert_eq!(ga_add(2, 2), 4);
    }
}
