use quick_xml::{Reader, events::Event};
use petgraph::{Graph, dot::{Dot, Config}, Direction};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use walkdir::WalkDir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut graph = Graph::<String, ()>::new();
    let mut id_map = HashMap::new();

    for entry in WalkDir::new("./xml_folder").into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() && entry.path().extension().map_or(false, |ext| ext == "xml") {
            let file = File::open(entry.path())?;
            let file_reader = BufReader::new(file);
            let mut reader = Reader::from_reader(file_reader);
            let mut buf = Vec::new();
            let mut node_stack = Vec::new();

            loop {
                match reader.read_event(&mut buf) {
                    Ok(Event::Start(ref e)) => {
                        let tag_name = reader.decode(e.name())?.to_string();
                        if !id_map.contains_key(&tag_name) {
                            let id = graph.add_node(tag_name.clone());
                            id_map.insert(tag_name.clone(), id);
                        }

                        if let Some(&parent_id) = node_stack.last() {
                            let child_id = *id_map.get(&tag_name).unwrap();
                            graph.add_edge(parent_id, child_id, ());
                        }

                        node_stack.push(*id_map.get(&tag_name).unwrap());
                    },
                    Ok(Event::End(_)) => {
                        node_stack.pop();
                    },
                    Ok(Event::Eof) => break,
                    Err(e) => return Err(Box::new(e)),
                    _ => {}
                }
            }

            buf.clear();
        }
    }

    let dot = Dot::with_config(&graph, &[Config::EdgeNoLabel]);
    let output_file = File::create("combined_output.dot")?;
    let mut writer = BufWriter::new(output_file);
    write!(writer, "{:?}", dot)?;

    Ok(())
}
