
extern crate termion;


use termion::{
    color,
    style,
    cursor,
    cursor::DetectCursorPos,
    clear,
    event::{
        Key, Event, MouseEvent
    },
    input::{
        TermRead, MouseTerminal
    },
    raw::IntoRawMode,
};

use scraper::{
    Selector,
    Html
};

use std::io::{
    Write,
    stdout,
    stdin
};
use std::fs;
use symlink;

trait Tui<'a> {
    fn choice(&mut self, message: &'a str, options: Vec<&'a str>) -> Result<&'a str, Box<dyn std::error::Error>>;
    fn input_number(&mut self, message: &'a str, min: i128, max: i128) -> Result<i128, Box<dyn std::error::Error>>;
}

impl<'a, W: Write> Tui<'a> for W {
    fn choice(&mut self, message: &'a str, options: Vec<&'a str>) -> Result<&'a str, Box<dyn std::error::Error>> {
        let stdin = stdin();
        let mut select = 0;
        let len = options.len();
        // let maxlen: u16 = {
        //     let mut a: u16 = 0;
        //     for e in &options {
        //         a = std::cmp::max(a, e.len() as u16);
        //     }
        //     a + 3
        // };

        // let mut draw = |select| {
        //     for i in 0..len  {
        //         if select == i {
        //             print!(
        //                 "\r{}>{} {}{}{} {}<{}\n",
        //                 color::Fg(color::Cyan), color::Fg(color::Reset),
        //                 style::Bold, options[i], style::Reset,
        //                 color::Fg(color::Cyan), color::Fg(color::Reset),
        //             );
        //         } else {
        //             print!("\r  {}{}{}  \n", color::Fg(color::LightBlack), options[i], color::Fg(color::Reset));
        //         }
        //     }
        //     self.flush()?;
        // };
        
        print!("\n{}\n", message);
        print!("\rPress the Arrow(↑→↓←) or Tab(Shift-Tab) key, or click a item to move.\n");

        // draw(select);
        for i in 0..len  {
            if select == i {
                print!(
                    "\r{}>{} {}{}{} {}<{}\n",
                    color::Fg(color::Cyan), color::Fg(color::Reset),
                    style::Bold, options[i], style::Reset,
                    color::Fg(color::Cyan), color::Fg(color::Reset),
                );
            } else {
                print!("\r  {}{}{}  \n", color::Fg(color::LightBlack), options[i], color::Fg(color::Reset));
            }
        }
        self.flush()?;
    
        for c in stdin.events() {
            let evt = c?;
            match evt {
                Event::Key(Key::BackTab) |
                Event::Key(Key::Up) => {
                    select = (select + len - 1) % len;
                }
                Event::Key(Key::Char('\t')) |
                Event::Key(Key::Down) => {
                    select = (select + 1) % len;
                }
                Event::Key(Key::Char('\n')) => break,
                Event::Mouse(MouseEvent::Press(_, _, pressed_y)) => {
                    let (s_y, e_y) = {
                        let (_, y) = self.cursor_pos()?;
                        (y, y - len as u16)
                    };
                    if s_y >= pressed_y && pressed_y >= e_y {
                        select = (pressed_y - e_y).into();
                    }
                }
                _ => {}
            }
            print!("{}", cursor::Up(len as u16));
            // draw(select);
            for i in 0..len  {
                if select == i {
                    print!(
                        "\r{}>{} {}{}{} {}<{}\n",
                        color::Fg(color::Cyan), color::Fg(color::Reset),
                        style::Bold, options[i], style::Reset,
                        color::Fg(color::Cyan), color::Fg(color::Reset),
                    );
                } else {
                    print!("\r  {}{}{}  \n", color::Fg(color::LightBlack), options[i], color::Fg(color::Reset));
                }
            }
            self.flush()?;
        }
        print!("{}\r{}", cursor::Up(2 + len as u16), clear::AfterCursor);
        self.flush()?;
        Ok(options[select])
    }
    fn input_number(&mut self, message: &'a str, min: i128, max: i128) -> Result<i128, Box<dyn std::error::Error>>
    {
        let stdin = stdin();
        print!(
            "\r{} : {}{}{}{}{}",
            message,
            cursor::Save,
            color::Fg(color::LightBlack), min, color::Fg(color::Reset),
            cursor::Restore,
        );
        self.flush()?;

        let mut buf = String::from("");

        for c in stdin.events() {
            let evt = c?;
            match evt {
                Event::Key(Key::Backspace) => {
                    if buf.len() > 0 {
                        buf.pop();
                    }
                    print!("{} {}", cursor::Left(1), cursor::Left(1));
                    self.flush()?;
                }
                Event::Key(Key::Char(c)) if c.is_ascii_digit() => {
                    buf.push(c);
                    if buf.parse::<i128>()? > max {
                        buf = format!("{}", max);
                    }
                }
                Event::Key(Key::Char('\n')) => 
                {
                    if buf.is_empty() {
                        print!("\r{}", clear::UntilNewline);
                        return Ok(min)
                    }
                    if buf.parse::<i128>()? < min {

                    } else {
                        break;
                    }
                },
                _ => {}
            }
            if buf.is_empty() {
                println!(
                    "{}{}{}{}{}{}",
                    cursor::Restore,
                    color::Fg(color::LightBlack), min, color::Fg(color::Reset),
                    clear::UntilNewline,
                    cursor::Restore,
                )
            } else {
                let f = buf.parse::<i128>()? < min;
                println!(
                    "{}{}{}{}{}",
                    cursor::Restore,
                    buf,
                    if f {" It's too small."} else {""},
                    clear::UntilNewline,
                    cursor::Restore,
                )
            }
        }
        print!("{}\r{}", cursor::Up(1), clear::UntilNewline);
        self.flush()?;
        if buf.is_empty() {
            Ok(min)
        } else {
            Ok(buf.parse()?)
        }
    }
}

fn gen_cargo(difficulty: char) -> String {
    format!(
        "
[package]
name = \"{project_name}\"
version = \"0.1.0\"
authors = [\"{authors}\"]
edition = \"2018\"

[[bin]]
name = \"main\"
path = \"src/main.rs\"

[dependencies]
# github.com/rust-lang-ja/atcoder-rust-resources/wiki/2020-Update

#num = \"=0.2.1\"
#num-bigint = \"=0.2.6\"
#num-complex = \"=0.2.4\"
#num-integer = \"=0.1.42\"
#num-iter = \"=0.1.40\"
#num-rational = \"=0.2.4\"
#num-traits = \"=0.2.11\"

#num-derive = \"=0.3.0\"

#ndarray = \"=0.13.0\"

#nalgebra = \"=0.20.0\"

#alga = \"=0.9.3\"

#libm = \"=0.2.1\"

#rand = {{ version = \"=0.7.3\", features = [\"small_rng\"] }}
#getrandom = \"=0.1.14\"
#rand_chacha = \"=0.2.2\"
#rand_core = \"=0.5.1\"
#rand_hc = \"=0.2.0\"
#rand_pcg = \"=0.2.1\"

#rand_distr = \"=0.2.2\"

#petgraph = \"=0.5.0\"

#indexmap = \"=1.3.2\"

#regex = \"=1.3.6\"

#lazy_static = \"=1.4.0\"

#ordered-float = \"=1.0.2\"

#ascii = \"=1.0.0\"

#permutohedron = \"=0.2.4\"

#superslice = \"=1.0.0\"

#itertools = \"=0.9.0\"

#itertools-num = \"=0.1.3\"

#maplit = \"=1.0.2\"

#either = \"=1.5.3\"

#im-rc = \"=14.3.0\"

#fixedbitset = \"=0.2.0\"

#bitset-fixed = \"=0.1.0\"

proconio = {{ version = \"=0.3.6\", features = [\"derive\"] }}

#text_io = \"=0.1.8\"

#whiteread = \"=0.5.0\"

#rustc-hash = \"=1.1.0\"

#smallvec = \"=1.2.0\"


[dev-dependencies]
cli_test_dir = \"0.1\"
",
        project_name=difficulty,
        authors=std::env::var("CARGO_NAME").unwrap_or_default(),
    )
}

fn gen_src(url: &String) -> String {
    format!(
        "
// {url}

use proconio::input;

fn main() {{
    input! {{
    }}
}}
",
        url=url
    )
}

fn gen_test(samples: &mut scraper::html::Select) -> String {
    let mut ret = String::from("use cli_test_dir::*;\n\n");
    let mut i = 1;
    loop {
        let input = match samples.next() {
            Some(x) => x.text().next().unwrap(),
            None => break,
        };
        let output = match samples.next() {
            Some(x) => x.text().next().unwrap(),
            None => break,
        };
        let test = format!(
            "
#[test]
fn sample{}() {{
    let testdir = TestDir::new(\"./main\", \"\");
    let output = testdir
        .cmd()
        .output_with_stdin(r#\"{}\"#)
        .tee_output()
        .expect_success();
    assert_eq!(output.stdout_str(), \"{}\");
    assert!(output.stderr_str().is_empty());
}}
",
            i,
            input,
            output
        );
        ret.push_str(test.as_str());
        i += 1;
    };
    ret
}

fn gen(contest: &str, number: i128) -> Result<(), Box<dyn std::error::Error>> {
    let samples = Selector::parse("div#task-statement > span.lang > span.lang-ja > hr ~ div.part > section > h3 ~ pre").unwrap();
    let difficulties = Selector::parse("table > tbody > tr > td:first-of-type > a").unwrap();
    let tasks = Selector::parse("table > tbody > tr > td:nth-of-type(2) > a").unwrap();

    let url = format!("https://atcoder.jp/contests/{c}{n:03}/tasks", c=contest, n=number);
    let body = reqwest::blocking::get(url.as_str())?.text()?;

    let document = Html::parse_document(&body);

    let difficulties = document.select(&difficulties).map(
        |e| {
            format!("{}", e.text().next().unwrap()).parse::<char>().unwrap()
        }
    );
    let tasks = document.select(&tasks).map(
        |e| {
            (
                format!("{}", e.text().next().unwrap()),
                format!("https://atcoder.jp{}", e.value().attr("href").unwrap())
            )
        }
    );

    fs::create_dir(format!("{c}{n:03}", c=contest.to_ascii_uppercase(), n=number))?;
    let mut path_link: String = Default::default();
    for (i, (difficulty, (name, url))) in difficulties.zip(tasks).enumerate() {
        let path = format!("{c}{n:03}/{}-{}", difficulty, name, c=contest.to_ascii_uppercase(), n=number);
        fs::create_dir_all(format!("{}/src", path))?;
        
        let src = gen_src(&url);
        let mut f = fs::File::create(format!("{}/src/main.rs", path))?;
        f.write_all(src.as_bytes())?;

        let cargo = gen_cargo(difficulty);
        let mut f = fs::File::create(format!("{}/Cargo.toml", path))?;
        f.write_all(cargo.as_bytes())?;

        let body = reqwest::blocking::get(url.as_str())?.text()?;
        let document = Html::parse_document(&body);

        let mut samples = document.select(&samples);

        fs::create_dir(format!("{}/tests", path))?;
        let tests = gen_test(&mut samples);
        let mut f = fs::File::create(format!("{}/tests/sampls.rs", path))?;
        f.write_all(tests.as_bytes())?;

        if i == 0 {
            path_link = format!("{}/target", path);
            fs::create_dir_all(format!("{}/debug", path_link))?;
        } else {
            symlink::symlink_dir(format!("../../{}", path_link), format!("{}/target", path))?;
        }

        let mut f = fs::File::create(format!("{}/rust-toolchain", path))?;
        f.write("1.42.0".as_bytes())?;
    }
    
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (contest, number) = {
        let mut out = MouseTerminal::from(stdout().into_raw_mode()?);
        let contest = out.choice("Which contest?", vec!["abc", "arc", "agc"])?;
        let number = out.input_number("What number?", 42, 200)?;
        (contest, number)
    };

    gen(contest, number).unwrap();
    
    Ok(())
}