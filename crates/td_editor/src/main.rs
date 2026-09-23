fn main() -> eframe::Result {
    let args = td_editor::CliArgs::parse(std::env::args().skip(1));
    if args.help {
        print!("{}", td_editor::USAGE);
        return Ok(());
    }

    if !args.errors.is_empty() {
        for e in &args.errors {
            eprintln!("trenchdot: {e}");
        }

        eprintln!("Run trenchdot --help to list the options.");
        std::process::exit(2);
    }

    if let Some(c) = &args.convert {
        if let Err(e) = td_editor::convert(c) {
            eprintln!("{e}");
            std::process::exit(1);
        }

        return Ok(());
    }

    td_editor::run(args)
}
