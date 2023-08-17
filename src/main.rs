use std::env;
use std::fs::{File, OpenOptions};
use std::io::{self, BufReader, Lines};
use std::io::{prelude::*, BufWriter};
use std::sync::{Arc, RwLock};

mod BinaryTree;
mod MaxHeap;

use std::collections::{HashMap, HashSet};
use ABtree::BTree;

use BinaryTree::BinaryTree::{ArcStr, BinaryT, HeapPair, Node, NodeIndex};
use MaxHeap::MaxHeap::MaxHeap as Maxheap;

/// Loading content of input file into memory
fn read_file(path: &str) -> io::Result<String> {
    let file = File::open(path)?;
    let mut buf_reader = BufReader::new(file);
    let mut content = String::new();
    buf_reader.read_to_string(&mut content)?;
    Ok(content)
}

fn wirte_file(path: &str, s: &str) -> io::Result<()> {
    let file = File::create(path)?;
    let mut fw = BufWriter::new(file);
    fw.write_all(s.as_bytes())?;
    Ok(())
}

/// Spltting the str
/// The life time of each line should share the
/// same lift time with the input file content
fn strtok<'content, 'b>(s: &'b mut &'content str, delimiter: &'static str) -> &'content str {
    if let Some(i) = s.find(delimiter) {
        let prefix = &s[..i];
        let suffix = &s[(i + delimiter.len())..];
        *s = suffix;
        prefix
    } else {
        let prefix = *s;
        *s = "";
        prefix
    }
}

fn clustering(tsv_file: &String) -> io::Result<()> {
    //let mut content = "s1 s2 -2\ns1 s3 -5\ns1 s4 -7\ns1 s5 -9\ns2 s3 -4\ns2 s4 -6\ns2 s5 -7\ns3 s4 -4\ns3 s5 -6\ns4 s5 -3\n".to_string();
    let content = read_file(&tsv_file)?;
    let row_sep = "\n";
    let data_sep = "\t";
    let mut lines = &content[..]; // life time is 'content
    let mut paired_values_dict: BTree<ArcStr<'_>, BTree<ArcStr<'_>, f64>> = BTree::new(4);
    let mut paired_values_heap: Maxheap<f64, HeapPair> = Maxheap::new();
    let mut all_samples: HashSet<ArcStr<'_>> = HashSet::new();
    let mut index_node_dict: HashMap<NodeIndex, Node<'_>> = HashMap::new();
    let mut node_index_dict: HashMap<Node<'_>, NodeIndex> = HashMap::new();
    let mut node_index: NodeIndex = 0;
    while lines.len() > 0 {
        // So the life time of each row is 'content
        let mut row = strtok(&mut lines, row_sep);
        let mut row_vec = Vec::new();
        while row.len() > 0 {
            row_vec.push(strtok(&mut row, data_sep))
        }
        if row_vec.len() != 3 {
            panic!("Wrong format for the line of content: {}", row);
        }
        let value = row_vec.pop().unwrap().trim().parse::<f64>().unwrap();
        let to_str = Arc::new(row_vec.pop().unwrap());
        let from_str = Arc::new(row_vec.pop().unwrap());
        let from_node = Node::new_from_str(from_str.clone());
        let to_node = Node::new_from_str(to_str.clone());

        if all_samples.insert(from_str.clone()) {
            index_node_dict.insert(node_index, Node::clone(&from_node));
            node_index_dict.insert(Node::clone(&from_node), node_index);
            node_index += 1;
        }
        if all_samples.insert(to_str.clone()) {
            index_node_dict.insert(node_index, Node::clone(&to_node));
            node_index_dict.insert(Node::clone(&to_node), node_index);
            node_index += 1;
        }

        if paired_values_dict.contains(&from_str) {
            let inner_map = paired_values_dict.get_mut(&from_str).unwrap();
            inner_map.insert(to_str.clone(), value);
        } else {
            let mut inner_map = BTree::new(4);
            inner_map.insert(to_str.clone(), value);
            paired_values_dict.insert(from_str.clone(), inner_map);
        }

        let from_index = *node_index_dict.get(&from_node).unwrap();
        let to_index = *node_index_dict.get(&to_node).unwrap();
        paired_values_heap.insert(value, HeapPair::new(from_index, to_index));
    }

    let n = all_samples.len();
    let expected_combination_nums = ((n * n) - n) / 2;
    let accepted_pair_nums = paired_values_dict
        .iter()
        .map(|(_k, v)| v.len())
        .into_iter()
        .reduce(|x, y| x + y)
        .unwrap_or(0);

    let msg = format!("The number of input samples is {} and expected combination number is {}, but accepted combination number is {}",
                      n, expected_combination_nums, accepted_pair_nums);
    assert!(expected_combination_nums == accepted_pair_nums, "{}", msg);

    let mut tree = BinaryT::new(index_node_dict, node_index_dict, all_samples);

    while tree.root_index.is_none() {
        let pair_info = paired_values_heap.pop_max().unwrap();
        tree.updating(
            pair_info.1.from_index,
            pair_info.1.to_index,
            &mut paired_values_heap,
            &paired_values_dict,
        )
    }

    let out_str = tree.to_newick();
    println!("{}", out_str);
    Ok(())
}

fn main() -> io::Result<()> {
    let tsv = std::env::args().nth(1).expect("Must give a tsv file.");
    clustering(&tsv)?;

    Ok(())
}
