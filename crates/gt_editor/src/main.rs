fn main() -> eframe::Result {
    gt_editor::run(gt_editor::CliArgs::parse(std::env::args().skip(1)))
}
