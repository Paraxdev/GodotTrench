fn main() -> eframe::Result {
    let args = gt_editor::CliArgs::parse(std::env::args().skip(1));
    if args.help {
        print!("{}", gt_editor::USAGE);
        return Ok(());
    }

    if !args.errors.is_empty() {
        for e in &args.errors {
            eprintln!("godottrench: {e}");
        }

        eprintln!("Run godottrench --help to list the options.");
        std::process::exit(2);
    }

    if let Some(c) = &args.convert {
        if let Err(e) = gt_editor::convert(c) {
            eprintln!("{e}");
            std::process::exit(1);
        }

        return Ok(());
    }

    gt_editor::run(args)
}
