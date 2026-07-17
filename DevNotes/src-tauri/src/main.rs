// Точка входа десктоп-приложения DEVNOTES.
// Прячем консольное окно на Windows в release-сборке.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    devnotes_lib::run()
}
