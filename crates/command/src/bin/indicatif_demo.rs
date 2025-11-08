use indicatif::MultiProgress;

fn main() {
    download_speed_progressbar();
}

fn buffer() {
    use console::Term;
    use dialoguer::{Confirm, Input, MultiSelect, Select, Sort, theme::ColorfulTheme};

    fn main() {
        let items = &[
            "Ice Cream",
            "Vanilla Cupcake",
            "Chocolate Muffin",
            "A Pile of sweet, sweet mustard",
        ];
        let term = Term::buffered_stderr();
        let theme = ColorfulTheme::default();

        println!("All the following controls are run in a buffered terminal");
        Confirm::with_theme(&theme)
            .with_prompt("Do you want to continue?")
            .interact_on(&term)
            .unwrap();

        let _: String = Input::with_theme(&theme)
            .with_prompt("Your name")
            .interact_on(&term)
            .unwrap();

        Select::with_theme(&theme)
            .with_prompt("Pick an item")
            .items(items)
            .interact_on(&term)
            .unwrap();

        MultiSelect::with_theme(&theme)
            .with_prompt("Pick some items")
            .items(items)
            .interact_on(&term)
            .unwrap();

        Sort::with_theme(&theme)
            .with_prompt("Order these items")
            .items(items)
            .interact_on(&term)
            .unwrap();
    }
    main();
}
fn multi_tree_ext() {
    use clap::Parser;
    use std::fmt::Debug;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;
    use std::time::Duration;

    use console::style;
    use indicatif::{MultiProgress, MultiProgressAlignment, ProgressBar, ProgressStyle};
    use once_cell::sync::Lazy;
    use rand::rngs::ThreadRng;
    use rand::{Rng, RngCore};
    println!("multi_tree_ext");
    #[derive(Debug, Clone)]
    enum Action {
        ModifyTree(usize),
        IncProgressBar(usize),
        Stop,
    }

    #[derive(Clone, Debug)]
    enum Elem {
        AddItem(Item),
        RemoveItem(Index),
    }

    #[derive(Clone, Debug)]
    struct Item {
        key: String,
        index: usize,
        indent: usize,
        progress_bar: ProgressBar,
    }

    #[derive(Clone, Debug)]
    struct Index(usize);

    const PB_LEN: u64 = 32;
    static ELEM_IDX: AtomicUsize = AtomicUsize::new(0);

    static ELEMENTS: Lazy<[Elem; 27]> = Lazy::new(|| {
        [
            Elem::AddItem(Item {
                indent: 9,
                index: 0,
                progress_bar: ProgressBar::new(PB_LEN),
                key: "dog".to_string(),
            }),
            Elem::AddItem(Item {
                indent: 0,
                index: 0,
                progress_bar: ProgressBar::new(PB_LEN),
                key: "temp_1".to_string(),
            }),
            Elem::AddItem(Item {
                indent: 8,
                index: 1,
                progress_bar: ProgressBar::new(PB_LEN),
                key: "lazy".to_string(),
            }),
            Elem::AddItem(Item {
                indent: 0,
                index: 1,
                progress_bar: ProgressBar::new(PB_LEN),
                key: "temp_2".to_string(),
            }),
            Elem::AddItem(Item {
                indent: 1,
                index: 0,
                progress_bar: ProgressBar::new(PB_LEN),
                key: "the".to_string(),
            }),
            Elem::AddItem(Item {
                indent: 0,
                index: 0,
                progress_bar: ProgressBar::new(PB_LEN),
                key: "temp_3".to_string(),
            }),
            Elem::AddItem(Item {
                indent: 7,
                index: 3,
                progress_bar: ProgressBar::new(PB_LEN),
                key: "a".to_string(),
            }),
            Elem::AddItem(Item {
                indent: 0,
                index: 3,
                progress_bar: ProgressBar::new(PB_LEN),
                key: "temp_4".to_string(),
            }),
            Elem::AddItem(Item {
                indent: 6,
                index: 2,
                progress_bar: ProgressBar::new(PB_LEN),
                key: "over".to_string(),
            }),
            Elem::RemoveItem(Index(6)),
            Elem::RemoveItem(Index(4)),
            Elem::RemoveItem(Index(3)),
            Elem::RemoveItem(Index(0)),
            Elem::AddItem(Item {
                indent: 0,
                index: 2,
                progress_bar: ProgressBar::new(PB_LEN),
                key: "temp_5".to_string(),
            }),
            Elem::AddItem(Item {
                indent: 4,
                index: 1,
                progress_bar: ProgressBar::new(PB_LEN),
                key: "fox".to_string(),
            }),
            Elem::AddItem(Item {
                indent: 0,
                index: 1,
                progress_bar: ProgressBar::new(PB_LEN),
                key: "temp_6".to_string(),
            }),
            Elem::AddItem(Item {
                indent: 2,
                index: 1,
                progress_bar: ProgressBar::new(PB_LEN),
                key: "quick".to_string(),
            }),
            Elem::AddItem(Item {
                indent: 0,
                index: 1,
                progress_bar: ProgressBar::new(PB_LEN),
                key: "temp_7".to_string(),
            }),
            Elem::AddItem(Item {
                indent: 5,
                index: 5,
                progress_bar: ProgressBar::new(PB_LEN),
                key: "jumps".to_string(),
            }),
            Elem::AddItem(Item {
                indent: 0,
                index: 5,
                progress_bar: ProgressBar::new(PB_LEN),
                key: "temp_8".to_string(),
            }),
            Elem::AddItem(Item {
                indent: 3,
                index: 4,
                progress_bar: ProgressBar::new(PB_LEN),
                key: "brown".to_string(),
            }),
            Elem::AddItem(Item {
                indent: 0,
                index: 3,
                progress_bar: ProgressBar::new(PB_LEN),
                key: "temp_9".to_string(),
            }),
            Elem::RemoveItem(Index(10)),
            Elem::RemoveItem(Index(7)),
            Elem::RemoveItem(Index(4)),
            Elem::RemoveItem(Index(3)),
            Elem::RemoveItem(Index(1)),
        ]
    });

    #[derive(Debug, Parser)]
    pub struct Config {
        #[clap(long)]
        bottom_alignment: bool,
    }

    /// The examples demonstrates the usage of `MultiProgress` and further extends `multi-tree` examples.
    /// Now the examples has 3 different actions implemented, and the item tree can be modified
    /// by inserting or removing progress bars. The progress bars to be removed eventually
    /// have messages with pattern `"temp_*"`.
    ///
    /// Also the command option `--bottom-alignment` is used to control the vertical alignment of the
    /// `MultiProgress`. To enable this run it with
    /// ```ignore
    /// cargo run --examples multi-tree-ext -- --bottom-alignment
    /// ```
    pub fn main() {
        let conf: Config = Config::parse();
        let mp = Arc::new(MultiProgress::new());
        let alignment = if conf.bottom_alignment {
            MultiProgressAlignment::Bottom
        } else {
            MultiProgressAlignment::Top
        };
        mp.set_alignment(alignment);
        let sty_main =
            ProgressStyle::with_template("{bar:40.green/yellow} {pos:>4}/{len:4}").unwrap();
        let sty_aux =
            ProgressStyle::with_template("[{pos:>2}/{len:2}] {prefix}{spinner:.green} {msg}")
                .unwrap();
        let sty_fin = ProgressStyle::with_template("[{pos:>2}/{len:2}] {prefix}{msg}").unwrap();

        let pb_main = mp.add(ProgressBar::new(
            ELEMENTS
                .iter()
                .map(|e| match e {
                    Elem::AddItem(item) => item.progress_bar.length().unwrap(),
                    Elem::RemoveItem(_) => 1,
                })
                .sum(),
        ));

        pb_main.set_style(sty_main);
        for e in ELEMENTS.iter() {
            match e {
                Elem::AddItem(item) => item.progress_bar.set_style(sty_aux.clone()),
                Elem::RemoveItem(_) => {}
            }
        }

        let mut items: Vec<&Item> = Vec::with_capacity(ELEMENTS.len());

        let mp2 = Arc::clone(&mp);
        let mut rng = ThreadRng::default();
        pb_main.tick();
        loop {
            match get_action(&mut rng, &items) {
                Action::Stop => {
                    // all elements were exhausted
                    pb_main.finish();
                    return;
                }
                Action::ModifyTree(elem_idx) => match &ELEMENTS[elem_idx] {
                    Elem::AddItem(item) => {
                        let pb = mp2.insert(item.index, item.progress_bar.clone());
                        pb.set_prefix("  ".repeat(item.indent));
                        pb.set_message(&item.key);
                        items.insert(item.index, item);
                    }
                    Elem::RemoveItem(Index(index)) => {
                        let item = items.remove(*index);
                        let pb = &item.progress_bar;
                        mp2.remove(pb);
                        pb_main.inc(pb.length().unwrap() - pb.position());
                    }
                },
                Action::IncProgressBar(item_idx) => {
                    let item = &items[item_idx];
                    item.progress_bar.inc(1);
                    let pos = item.progress_bar.position();
                    if pos >= item.progress_bar.length().unwrap() {
                        item.progress_bar.set_style(sty_fin.clone());
                        item.progress_bar.finish_with_message(format!(
                            "{} {}",
                            style("✔").green(),
                            item.key
                        ));
                    }
                    pb_main.inc(1);
                }
            }
            thread::sleep(Duration::from_millis(20));
        }
    }

    /// The function guarantees to return the action, that is valid for the current tree.
    fn get_action(rng: &mut dyn RngCore, items: &[&Item]) -> Action {
        let elem_idx = ELEM_IDX.load(Ordering::SeqCst);
        // the indices of those items, that not completed yet
        let uncompleted = items
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                let pos = item.progress_bar.position();
                pos < item.progress_bar.length().unwrap()
            })
            .map(|(idx, _)| idx)
            .collect::<Vec<usize>>();
        let k = rng.gen_range(0..16);
        if (k > 0 || k == 0 && elem_idx == ELEMENTS.len()) && !uncompleted.is_empty() {
            let idx = rng.gen_range(0..uncompleted.len() as u64) as usize;
            Action::IncProgressBar(uncompleted[idx])
        } else if elem_idx < ELEMENTS.len() {
            ELEM_IDX.fetch_add(1, Ordering::SeqCst);
            Action::ModifyTree(elem_idx)
        } else {
            // nothing to do more
            Action::Stop
        }
    }
    main();
}
fn iterator() {
    use std::thread;
    use std::time::Duration;

    use indicatif::{ProgressBar, ProgressIterator, ProgressStyle};
    println!("iterator");
    fn main() {
        // Default styling, attempt to use Iterator::size_hint to count input size
        for _ in (0..1000).progress() {
            // ...
            thread::sleep(Duration::from_millis(5));
        }

        // Provide explicit number of elements in iterator
        for _ in (0..1000).progress_count(1000) {
            // ...
            thread::sleep(Duration::from_millis(5));
        }

        // Provide a custom bar style
        let pb = ProgressBar::new(1000);
        pb.set_style(
      ProgressStyle::with_template(
        "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] ({pos}/{len}, ETA {eta})",
      )
        .unwrap(),
    );
        for _ in (0..1000).progress_with(pb) {
            // ...
            thread::sleep(Duration::from_millis(5));
        }
    }
    main();
}
fn finebars() {
    use std::thread;
    use std::time::Duration;

    use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
    use rand::{Rng, thread_rng};
    let styles = [
        ("Rough bar:", "█  ", "red"),
        ("Fine bar: ", "█▉▊▋▌▍▎▏  ", "yellow"),
        ("Vertical: ", "█▇▆▅▄▃▂▁  ", "green"),
        ("Fade in:  ", "█▓▒░  ", "blue"),
        ("Blocky:   ", "█▛▌▖  ", "magenta"),
    ];

    let m = MultiProgress::new();

    let handles: Vec<_> = styles
        .iter()
        .map(|s| {
            let pb = m.add(ProgressBar::new(512));
            pb.set_style(
                ProgressStyle::with_template(&format!("{{prefix:.bold}}▕{{bar:.{}}}▏{{msg}}", s.2))
                    .unwrap()
                    .progress_chars(s.1),
            );
            pb.set_prefix(s.0);
            let wait = Duration::from_millis(thread_rng().gen_range(10..30));
            thread::spawn(move || {
                for i in 0..512 {
                    thread::sleep(wait);
                    pb.inc(1);
                    pb.set_message(format!("{:3}%", 100 * i / 512));
                }
                pb.finish_with_message("100%");
            })
        })
        .collect();

    for h in handles {
        let _ = h.join();
    }
}
fn yarnish() {
    use std::thread;
    use std::time::{Duration, Instant};

    use console::{Emoji, style};
    use indicatif::{HumanDuration, MultiProgress, ProgressBar, ProgressStyle};
    use rand::Rng;
    use rand::seq::SliceRandom;
    static PACKAGES: &[&str] = &[
        "fs-events",
        "my-awesome-module",
        "emoji-speaker",
        "wrap-ansi",
        "stream-browserify",
        "acorn-dynamic-import",
    ];

    static COMMANDS: &[&str] = &[
        "cmake .",
        "make",
        "make clean",
        "gcc foo.c -o foo",
        "gcc bar.c -o bar",
        "./helper.sh rebuild-cache",
        "make all-clean",
        "make test",
    ];

    static LOOKING_GLASS: Emoji<'_, '_> = Emoji("🔍  ", "");
    static TRUCK: Emoji<'_, '_> = Emoji("🚚  ", "");
    static CLIP: Emoji<'_, '_> = Emoji("🔗  ", "");
    static PAPER: Emoji<'_, '_> = Emoji("📃  ", "");
    static SPARKLE: Emoji<'_, '_> = Emoji("✨ ", ":-)");
    let mut rng = rand::thread_rng();
    let started = Instant::now();
    let spinner_style = ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {wide_msg}")
        .unwrap()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");

    println!(
        "{} {}Resolving packages...",
        style("[1/4]").bold().dim(),
        LOOKING_GLASS
    );
    println!(
        "{} {}Fetching packages...",
        style("[2/4]").bold().dim(),
        TRUCK
    );

    println!(
        "{} {}Linking dependencies...",
        style("[3/4]").bold().dim(),
        CLIP
    );
    let deps = 1232;
    let pb = ProgressBar::new(deps);
    for _ in 0..deps {
        thread::sleep(Duration::from_millis(3));
        pb.inc(1);
    }
    pb.finish_and_clear();

    println!(
        "{} {}Building fresh packages...",
        style("[4/4]").bold().dim(),
        PAPER
    );
    let m = MultiProgress::new();
    let handles: Vec<_> = (0..4u32)
        .map(|i| {
            let count = rng.gen_range(30..80);
            let pb = m.add(ProgressBar::new(count));
            pb.set_style(spinner_style.clone());
            pb.set_prefix(format!("[{}/?]", i + 1));
            thread::spawn(move || {
                let mut rng = rand::thread_rng();
                let pkg = PACKAGES.choose(&mut rng).unwrap();
                for _ in 0..count {
                    let cmd = COMMANDS.choose(&mut rng).unwrap();
                    thread::sleep(Duration::from_millis(rng.gen_range(25..200)));
                    pb.set_message(format!("{pkg}: {cmd}"));
                    pb.inc(1);
                }
                pb.finish_with_message("waiting...");
            })
        })
        .collect();
    for h in handles {
        let _ = h.join();
    }
    m.clear().unwrap();

    println!("{} Done in {}", SPARKLE, HumanDuration(started.elapsed()));
}

fn download_speed_progressbar() {
    use indicatif::{ProgressBar, ProgressStyle};
    use std::cmp::min;
    use std::thread;
    use std::time::Duration;
    let mut downloaded = 0;
    let total_size = 2312310;
    let m = MultiProgress::new();
    let pb = ProgressBar::new(total_size as u64);
    let sty = ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
        .unwrap()
        .progress_chars("#>-");

    pb.set_style(ProgressStyle
  ::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
    .unwrap()
    .progress_chars("#>-"));
    let pb1 = m.add(ProgressBar::new(100));
    pb1.set_style(sty.clone());
    pb1.set_message("Task 1");

    let pb2 = m.add(ProgressBar::new(200));
    pb2.set_style(sty.clone());
    pb2.set_message("Task 2");

    let pb3 = m.add(ProgressBar::new(50));
    pb3.set_style(sty);
    pb3.set_message("Task 3");
    println!("Downloading...");
    while downloaded < total_size as u64 {
        let new = min(downloaded + 223211, total_size as u64);
        downloaded = new;
        pb.set_position(new);
        pb1.inc(1);
        pb2.inc(1);
        pb3.inc(1);
        thread::sleep(Duration::from_millis(12));
    }
    println!("Downloaded");
}
