macro_rules! eprintln (
    ($($arg:tt)*) => { {
        use std::io::Write;

        writeln!(&mut ::std::io::stderr(), $($arg)*).unwrap();
    } }
);
