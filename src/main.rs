use quick_xml::{Reader, events::Event};
use petgraph::{Graph, dot::{Dot, Config}};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Write, Read};
use zip::ZipArchive;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("example.adpro")?;
    let mut archive = ZipArchive::new(file)?;

    let mut graph = Graph::<String, ()>::new();
    let mut id_map = HashMap::new();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        println!("Processing file: {}", file.name()); // Debug statement

        if file.name().starts_with("task\\") && file.name().ends_with(".rll") {
            println!("Found XML file in tasks/: {}", file.name()); // Debug statement

            let mut contents = String::new();
            file.read_to_string(&mut contents)?;

            let mut reader = Reader::from_str(&contents);
            let mut buf = Vec::new();
            let mut node_stack = Vec::new();

            loop {
                match reader.read_event(&mut buf) {
                    Ok(Event::Start(ref e)) => {
                        let tag_name = reader.decode(e.name())?.to_string();
                        println!("Start Tag: {}", tag_name); // Debug statement

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
                    Err(e) => {
                        println!("Error parsing XML: {}", e); // Error output
                        return Err(Box::new(e));
                    },
                    _ => {}
                }
            }

            buf.clear();
        }
    }

    if graph.node_count() == 0 {
        println!("No nodes in graph."); // Check if graph is empty
    } else {
        println!("Graph has nodes."); // Confirm nodes are present
    }

    let output_file = "combined_output.dot";
    let mut writer = File::create(output_file)?;
    write!(writer, "{:?}", Dot::with_config(&graph, &[Config::EdgeNoLabel]))?;

    Ok(())
}
