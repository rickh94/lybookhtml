use askama::Template;
use clap::Parser;
use serde::Deserialize;
use slugify::slugify;
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};

const LILY_CONTENT: &str = "\\version \"2.24.1\"
\\language \"english\"
\\paper {
    #'(set-paper-size \"letter\")
}

\\score {
    \\relative {
        a4 b c d
    }
}
";

const BUILDSCRIPT: &str = "
#!/usr/bin/env bash
BOOK=`which lilypond-book`
LILYPOND=`which lilypond`
$BOOK --process=\"$LILYPOND -dresolution=300\" index.html -o output
";

const WATCHSCRIPT: &str = "
#!/usr/bin/env bash
ENTR=`which entr`
FSWATCH=`which fswatch`
BOOK=`which lilypond-book`
LILYPOND=`which lilypond`

if [ ! -z $ENTR ]; then
	ls **/*.{html,ly} | $ENTR -s \"$BOOK --process=\\\"$LILYPOND -dresolution=300\\\" index.html -o output\" 
elif [ ! -z $FSWATCH ]; then
	$FSWATCH **/*.{html,ly} | xargs -n1 -I{} $BOOK --process=\"$LILYPOND -dresolution=300\" index.html -o output
fi
";

#[derive(Template, Deserialize)]
#[template(path = "index.html")]
struct Index {
    title: String,
    composer: Option<String>,
    chapters: Vec<Chapter>,
}

#[derive(Deserialize)]
struct Chapter {
    header: Option<String>,
    exercises: Vec<Exercise>,
}

#[derive(Deserialize)]
struct Exercise {
    title: String,
    subtitle: Option<String>,
    instructions: Option<String>,
    groups: Vec<Group>,
}

#[derive(Deserialize)]
struct Group {
    heading: Option<String>,
    subheading: Option<String>,
    music: Vec<Music>,
    id: String,
}

#[derive(Deserialize)]
struct Music {
    instructions: Option<String>,
    id: String,
}

mod filters {
    use slugify::slugify;

    // REMINDER: NO space before |filter
    pub fn slug(input: &str) -> ::askama::Result<String> {
        Ok(slugify!(input))
    }
}

/// Lilypond Book HTML
/// Create a skeleton for a music exercise book that will use
/// lilypond-book and html from a yaml file.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to your configuration yaml file
    config: PathBuf,
    /// Path to save the generated files
    #[arg(short, long)]
    output: Option<PathBuf>,
    /// Do not delete any existing files (default is to replace the html)
    #[arg(short, long)]
    no_clobber: bool,
    /// Overwrite all existing files (default is not to overwrite existing lilypond files)
    #[arg(short, long)]
    clobber_all: bool,
}

// TODO: Validate that ids are unique per exercise-group

fn main() {
    let args = Args::parse();

    let mut config_file = fs::File::open(&args.config).expect("Could not open config file");
    let mut config = String::with_capacity(1024);
    config_file
        .read_to_string(&mut config)
        .expect("Could not read config file");

    let output: Index = serde_yaml::from_str(&config).expect("Invalid config file");
    let title_slug = slugify!(&output.title);

    let output_path = match &args.output {
        Some(p) => Path::new(p),
        None => Path::new(&title_slug),
    };
    if !output_path.is_dir() {
        fs::create_dir(&output_path).expect("Could not create directory");
    }

    for chapter in output.chapters.iter() {
        for exercise in chapter.exercises.iter() {
            let slug = slugify!(&exercise.title);
            let exercise_path = output_path.join(&slug);
            if !exercise_path.is_dir() {
                fs::create_dir(&exercise_path).expect("Could not create exercise directory");
            }
            for group in exercise.groups.iter() {
                for music in group.music.iter() {
                    let path =
                        exercise_path.join(format!("{}-{}-{}.ly", &slug, &group.id, music.id));
                    if !path.is_file() || args.clobber_all {
                        fs::write(path, LILY_CONTENT).expect("Could not write file");
                    }
                }
            }
        }
    }

    let html_path = output_path.join("index.html");
    if !args.no_clobber {
        fs::write(html_path, output.render().expect("Could not render html"))
            .expect("Could not write file");
    }

    let build_script_path = output_path.join("build.sh");
    if !build_script_path.is_file() || args.clobber_all {
        fs::write(build_script_path, BUILDSCRIPT).expect("Could not write build script");
    }
    let watch_script_path = output_path.join("watch.sh");
    if !watch_script_path.is_file() || args.clobber_all {
        fs::write(watch_script_path, WATCHSCRIPT).expect("Could not write watch script");
    }
}
