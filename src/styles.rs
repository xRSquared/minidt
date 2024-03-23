use anstyle::{Color, Style};

pub fn get_styles() -> clap::builder::Styles{
    clap::builder::Styles::styled()
        .header(Style::new().fg_color(Some(Color::Ansi(anstyle::AnsiColor::Yellow))))
        .literal(Style::new().fg_color(Some(Color::Ansi(anstyle::AnsiColor::Green))))
        .usage(Style::new().fg_color(Some(Color::Ansi(anstyle::AnsiColor::Yellow))))
        .error(Style::new().bold().fg_color(Some(Color::Ansi(anstyle::AnsiColor::BrightRed))))
        .placeholder(Style::new().fg_color(Some(Color::Ansi(anstyle::AnsiColor::BrightWhite))))
        .valid(Style::new().bold().underline().fg_color(Some( Color::Ansi(anstyle::AnsiColor::Green))))
        .invalid(Style::new().bold().underline().fg_color(Some( Color::Ansi(anstyle::AnsiColor::Red))))
}
