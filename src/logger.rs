#[macro_export]
macro_rules! log {
    ($level: expr, $text: expr) => {
        assert!($level == "debug" || $level == "info" || $level == "warning" || $level == "error");
        let now: DateTime<Utc> = Utc::now();
        let formatted = String::from(format!(
            "[{}] {} [{}:{}]: {}",
            $level,
            now.format("%Y-%m-%d %H:%M:%S"),
            file!(),
            line!(),
            $text
        ));

        if *LOG_LEVEL.to_lowercase() == String::from("debug") {
            let val = formatted.to_owned();
            match $level {
                "debug" => println!("{}", Colour::Green.paint(val)),
                "info" => println!("{}", val),
                "warning" => println!("{}", Colour::Yellow.paint(val)),
                "error" => println!("{}", Colour::Red.paint(val)),
                _ => {}
            }
        };

        if *LOG_LEVEL.to_lowercase() == String::from("info") {
            let val = formatted.to_owned();
            match $level {
                "info" => println!("{}", val),
                "warning" => println!("{}", Colour::Yellow.paint(val)),
                "error" => println!("{}", Colour::Red.paint(val)),
                _ => {}
            }
        };

        if *LOG_LEVEL.to_lowercase() == String::from("warning") {
            let val = formatted.to_owned();
            match $level {
                "warning" => println!("{}", Colour::Yellow.paint(val)),
                "error" => println!("{}", Colour::Red.paint(val)),
                _ => {}
            }
        };

        if *LOG_LEVEL.to_lowercase() == String::from("error") {
            let val = formatted.to_owned();
            match $level {
                "error" => println!("{}", Colour::Red.paint(val)),
                _ => {}
            }
        };
    };
}
