#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Otimizações extremas de consumo de memória para Linux WebKitGTK
    #[cfg(target_os = "linux")]
    {
        std::env::set_var("WEBKIT_USE_SINGLE_WEB_PROCESS", "1");
        std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
    }

    relay_lib::run();
}
