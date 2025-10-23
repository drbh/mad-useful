use termcolor::{Color, ColorSpec, StandardStream, WriteColor};

pub fn print_colored_count(
    stdout: &mut StandardStream,
    count: usize,
    file_count: usize,
    max_lines_per_file: usize,
    no_color: bool,
) {
    if no_color {
        print!("{count:>8}");
    } else {
        let max_lines = if file_count == 1 {
            max_lines_per_file
        } else {
            file_count * max_lines_per_file
        };
        let ratio = (count as f64 / max_lines as f64).min(1.0);
        let red = (255.0 * ratio) as u8;
        let green = (255.0 * (1.0 - ratio)) as u8;

        let mut color_spec = ColorSpec::new();
        color_spec.set_fg(Some(Color::Rgb(red, green, 0)));

        stdout.set_color(&color_spec).unwrap();
        print!("{count:>8}");
        stdout.reset().unwrap();
    }
}
