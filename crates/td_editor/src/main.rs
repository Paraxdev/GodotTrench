fn main() -> eframe::Result {
    td_editor::run(td_editor::CliArgs::parse(std::env::args().skip(1)))
}
