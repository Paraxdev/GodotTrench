fn main() -> eframe::Result {
    let args = gt_editor::CliArgs::parse(std::env::args().skip(1));
    if let Some(c) = &args.convert {
        if let Err(e) = gt_editor::convert(c) {
            eprintln!("{e}");
            std::process::exit(1);
        }

        return Ok(());
    }

    gt_editor::run(args)
}
