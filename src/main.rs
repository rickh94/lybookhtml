use slugify::slugify;
use std::{fs, path::Path};

use askama::Template;

const LILY_CONTENT: &str = "\\version \"2.24.1\"\n\\language \"english\"\n\\paper {\n\t#'(set-paper-size \"letter\")\n}\n\\score {\n\t\\relative {\n\n\t}\n}\n";

#[derive(Template)]
#[template(path = "index.html")]
struct Index {
    title: String,
    chapters: Vec<Chapter>,
}

struct Chapter {
    header: Option<String>,
    exercises: Vec<Exercise>,
}

struct Exercise {
    title: String,
    subtitle: Option<String>,
    instructions: Option<String>,
    groups: Vec<Group>,
}

struct Group {
    heading: Option<String>,
    subheading: Option<String>,
    music: Vec<Music>,
    id: String,
}

struct Music {
    instructions: Option<String>,
    id: String,
}

// TODO: Validate that ids are unique per exercise-group

fn main() {
    let output = Index {
        title: "Sevcik, Op. 2, Book 1".into(),
        chapters: vec![
            Chapter {
                header: Some("Section I".into()),
                exercises: vec![Exercise {
                    title: "No. 2".into(),
                    subtitle: None,
                    instructions: Some("Play the 18 examples below, without raising the bow, in each of the following six styles.".into()),
                    groups: vec![
                        Group {
                            heading: Some("Variants".into()),
                            subheading: None,
                            music: vec![Music { instructions: None, id: "a".into()}],
                                id: 1.to_string(),
                        },
                        Group {
                            heading: Some("Examples".into()),
                            subheading: None,
                            music: vec![Music { instructions: None, id: "a".into()}],
                                id: 2.to_string()
                        },
                    ]
                },
                Exercise {
                    title: "No. 3".into(),
                    subtitle: Some("Rhythmic exercises and dividing of the bow length".into()),
                    instructions: Some("Examples in whole notes with 57 variants. Practice each variant of the whole example.".into()),
                    groups: vec![
                        Group {
                            heading: Some("Example".into()),
                            subheading: None,
                            music: vec![Music { instructions: None, id: "a".into()}],
                            id: 1.to_string()
                        },
                        Group {
                            heading: Some("Variants".into()),
                            subheading: None,
                            id: 2.to_string(),
                            music: vec![
                                Music {
                                    instructions: Some("Whole Bow".into()),
                                    id: "a".into()
                                },
                                Music {
                                    instructions: Some("Half Bow".into()),
                                    id: "b".into()
                                },
                                Music {
                                    instructions: Some("Half Bow then Whole Bow".into()),
                                    id: "c".into()
                                },
                                Music {
                                    instructions: Some("In the middle".into()),
                                    id: "d".into()
                                },
                                Music {
                                    instructions: Some("Picchettato (Detached)".into()),
                                    id: "e".into()
                                },
                                Music {
                                    instructions: Some("One Third of the bow".into()),
                                    id: "f".into()
                                }
                            ]
                        }
                    ]
                }
                ],
            },
            Chapter {
                header: None,
                exercises: vec![Exercise {
                    title: "No. 1".into(),
                    subtitle: None,
                    instructions: Some("Test".into()),
                    groups: vec![],
                }],
            },
        ],
    };
    let output_path = Path::new("./output");
    fs::create_dir(&output_path).expect("Could not create directory");

    for chapter in output.chapters.iter() {
        for exercise in chapter.exercises.iter() {
            let slug = slugify!(&exercise.title);
            let exercise_path = output_path.join(&slug);
            fs::create_dir(&exercise_path).expect("Could not create group directory");
            for group in exercise.groups.iter() {
                for music in group.music.iter() {
                    let path = exercise_path.join(format!("{}-{}.ly", &slug, music.id));
                    fs::write(path, LILY_CONTENT).expect("Could not write file");
                }
            }
        }
    }
    let html_path = output_path.join("index.html");
    fs::write(html_path, output.render().expect("Could not render html"))
        .expect("Could not write file");
}
