use async_std::task::block_on;
use bve::filesystem::read_convert_utf8;
use itertools::Itertools;
use obj::{Obj, SimplePolygon};
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, Cursor, Write},
    panic::catch_unwind,
    path::PathBuf,
};

const HEADER: &str = concat!(
    r#"BVE-Reborn .obj to .csv Converter
https://github.com/bve-reborn/bve-reborn
Version "#,
    env!("CARGO_PKG_VERSION"),
    "\n\n"
);

fn print(data: impl AsRef<[u8]>) {
    let stdout = std::io::stdout();
    let mut lock = stdout.lock();
    lock.write_all(data.as_ref()).expect("Could not write to stdout");
    lock.flush().expect("Could not flush stdout");
}

fn read_data() -> String {
    let stdin = std::io::stdin();
    let mut lock = stdin.lock();
    let mut string = String::new();
    lock.read_line(&mut string).expect("Could not read input");
    string.pop(); // remove newline
    if let Some(idx) = string.find('"') {
        string = string.chars().skip(idx + 1).take_while(|c| *c != '"').collect();
    };
    string
}

enum State {
    Input,
    Wrong,
}

fn get_input<T>(prompt: impl AsRef<[u8]>, mut verification: impl FnMut(Option<PathBuf>) -> Result<T, String>) -> T {
    let mut state = State::Input;
    loop {
        match state {
            State::Input => {
                print(prompt.as_ref());
                let input = read_data();
                let trimmed = input.trim();
                let verifiable = if trimmed.is_empty() {
                    None
                } else {
                    Some(PathBuf::from(trimmed))
                };
                match verification(verifiable) {
                    Ok(value) => break value,
                    Err(msg) => {
                        state = State::Wrong;
                        print(msg + "\n");
                    }
                }
            }
            State::Wrong => {
                print("\nTry again:\n");
                state = State::Input;
            }
        }
    }
}

fn obj_to_csv(input_size: usize, obj: Obj<SimplePolygon>) -> String {
    let mut output = String::from(concat!(
        "; Generated by BVE-Reborn .obj to .csv Converter\n; bve-obj-cov v",
        env!("CARGO_PKG_VERSION"),
        "\n; Do not edit manually\n"
    ));
    output.reserve(input_size * 4);
    for object in &obj.objects {
        for group in &object.groups {
            let mut translation: HashMap<(usize, Option<usize>), usize> = HashMap::new();
            let mut vert_count = 0_usize;
            output.push_str("\nCreateMeshBuilder\n");

            if let Some(material) = &group.material {
                if let Some(texture) = &material.map_kd {
                    output.push_str(&format!("LoadTexture, {}\n", texture));
                } else {
                    if let Some(color) = &material.kd {
                        let filter_color = |color: f32| (color * 255.0).round().min(255.0) as u8;
                        output.push_str(&format!(
                            "SetColor, {}, {}, {}, 255\n",
                            filter_color(color[0]),
                            filter_color(color[1]),
                            filter_color(color[2])
                        ));
                    }
                }
            }

            for polygon in &group.polys {
                let mut indices = Vec::new();
                for index_pair in polygon {
                    let position_idx = index_pair.0;
                    let tex_coord_idx = index_pair.1.map(|v| v);
                    let normal_idx = index_pair.2.map(|v| v);
                    let vertex_idx = if let Some(&existing_vert) = translation.get(&(position_idx, normal_idx)) {
                        existing_vert
                    } else {
                        let this_vert = vert_count;
                        vert_count += 1;

                        let position = obj.position[position_idx];
                        let normal_opt = normal_idx.map(|idx| obj.normal[idx]);
                        let texture_coords_opt = tex_coord_idx.map(|idx| obj.texture[idx]);

                        if let Some(normal) = normal_opt {
                            output.push_str(&format!(
                                "AddVertex, {}, {}, {}, {}, {}, {}\n",
                                position[0], position[1], position[2], normal[0], normal[1], normal[2]
                            ));
                        } else {
                            output.push_str(&format!(
                                "AddVertex, {}, {}, {}\n",
                                position[0], position[1], position[2]
                            ));
                        }

                        if let Some(texture_coords) = texture_coords_opt {
                            output.push_str(&format!(
                                "SetTextureCoordinates, {}, {}, {}\n",
                                this_vert, texture_coords[0], texture_coords[1]
                            ));
                        }

                        translation.insert((position_idx, normal_idx), this_vert);

                        this_vert
                    };
                    indices.push(vertex_idx);
                }
                output.push_str("AddFace");
                for index in indices {
                    output.push_str(&format!(", {}", index));
                }
                output.push('\n');
            }
        }
    }
    output
}

fn input_main() -> Option<()> {
    let (file, contents) = get_input(
        "Enter or drag and drop path to .obj file then hit enter:\n > ",
        |file_opt| {
            let file = file_opt.ok_or_else(|| String::from("No path given"))?;
            let contents = block_on(read_convert_utf8(&file)).map_err(|err| err.to_string())?;
            Ok((file, contents))
        },
    );

    print("Parsing .obj... ");

    let length = contents.len();
    let mut cursor = Cursor::new(contents);
    let parsed_object_result = Obj::<SimplePolygon>::load_buf(&mut cursor)
        .map_err(|err| format!("{:?}", err))
        .and_then(|mut obj| {
            obj.path = file.parent().unwrap().to_path_buf();
            obj.load_mtls_fn(|base, mat| {
                let file = block_on(read_convert_utf8(base.join(mat)))?;
                Ok(Cursor::new(file))
            })
            .map_err(|vec| {
                format!(
                    "Could not load mtl files:\n{}",
                    vec.into_iter()
                        .map(|(file, error)| format!("\t{}, {:?}", file, error))
                        .join("\n")
                )
            })?;
            Ok(obj)
        });

    let parsed_object = match parsed_object_result {
        Ok(obj) => obj,
        Err(err) => {
            eprintln!("\nError reading obj file: {}\n", err);
            return None;
        }
    };

    print("done\n\n");

    let csv_file_suggestion = file.with_extension("csv");

    let mut csv_file = get_input(
        format!(
            "Output .csv file: [defaults to \"{}\"]\n > ",
            csv_file_suggestion.display()
        ),
        |file| File::create(file.unwrap_or(csv_file_suggestion.clone())).map_err(|err| err.to_string()),
    );

    let output = obj_to_csv(length, parsed_object);

    csv_file
        .write_all(output.as_bytes())
        .expect("Can't write to output file");

    drop(csv_file);

    get_input("\nFinished Conversion! Press enter to close\n", |_| Ok(()));

    Some(())
}

fn main() {
    print(HEADER);
    loop {
        let error = catch_unwind(|| input_main());
        match error {
            Err(_error) => {
                get_input(
                    "\nFatal Error. This is a bug. Copy the above text and send in a bug report.\n",
                    |_| Ok(()),
                );
            }
            Ok(Some(())) => {
                break;
            }
            _ => {}
        }
    }
}
