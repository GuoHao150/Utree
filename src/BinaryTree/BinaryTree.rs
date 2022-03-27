use super::super::MaxHeap::MaxHeap::MaxHeap as Maxheap;
use crate::MaxHeap::MaxHeap::MaxHeap;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::format;
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;
use std::sync::Arc;
use ABtree::BTree;

pub(crate) type ArcStr<'content> = Arc<&'content str>;
pub(crate) type NodeIndex = usize;

/// A simple struct to store indexes of paired node
/// in the heap
pub(crate) struct HeapPair {
    pub(crate) from_index: NodeIndex,
    pub(crate) to_index: NodeIndex,
}

impl HeapPair {
    pub(crate) fn new(f: NodeIndex, t: NodeIndex) -> Self {
        HeapPair {
            from_index: f,
            to_index: t,
        }
    }
}

#[derive(Eq)]
pub(crate) struct Node<'content> {
    pub(crate) data: VecDeque<ArcStr<'content>>,
    parent: Option<NodeIndex>,
    left: Option<NodeIndex>,
    right: Option<NodeIndex>,
}

impl<'content> PartialEq for Node<'content> {
    fn eq(&self, other: &Self) -> bool {
        self.data.iter().eq(other.data.iter())
    }
}

impl<'content> Hash for Node<'content> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // The order of elements will makes difference
        // when using iter to reimplement hash method
        self.data.iter().for_each(|x| x.hash(state))
    }
}

impl<'content> Node<'content> {
    #[inline]
    pub(crate) fn new_from_str(i: ArcStr<'content>) -> Self {
        let mut v = VecDeque::new();
        v.push_back(i);
        Node {
            data: v,
            parent: None,
            left: None,
            right: None,
        }
    }

    #[inline]
    pub(crate) fn new_empty() -> Self {
        let v = VecDeque::new();
        Node {
            data: v,
            parent: None,
            left: None,
            right: None,
        }
    }

    #[inline]
    fn inner_data_size(node: &Node) -> usize {
        node.data.len()
    }

    #[inline]
    fn get_inner_data<'n>(node: &'n Node<'content>) -> &'n VecDeque<ArcStr<'content>> {
        &node.data
    }

    #[inline]
    fn set_left(node: &mut Node, left_index: NodeIndex) {
        node.left = Some(left_index);
    }

    #[inline]
    fn set_right(node: &mut Node, right_index: NodeIndex) {
        node.right = Some(right_index);
    }

    #[inline]
    fn set_parent(node: &mut Node, parent_index: NodeIndex) {
        node.parent = Some(parent_index);
    }

    #[inline]
    fn get_parent_idx(node: &Node) -> Option<NodeIndex> {
        node.parent.clone()
    }

    /// Pop out the str data in the leaf node
    /// and it's not a in-place operation because of cloning
    /// Note: The caller must make sure the input node's inner data
    /// only has one `ArcStr`
    #[inline]
    fn get_left_str<'n>(node: &'n Node<'content>) -> ArcStr<'content> {
        let mut v = node.data.clone();
        v.pop_back().unwrap()
    }

    /// Given a node and an inner data which is a `VecDeque`
    /// copy the data in the inner into the data of node
    #[inline]
    fn add_inner<'n, 'v>(node: &'n mut Node<'content>, inputs: &'v VecDeque<ArcStr<'content>>) {
        for i in inputs.iter() {
            node.data.push_back(i.clone());
        }
    }

    /// Check if two nodes are the same
    #[inline]
    fn is_equal(n1: &Node, n2: &Node) -> bool {
        n1.data.iter().eq(n2.data.iter())
    }

    pub(crate) fn clone<'n>(node: &'n Node<'content>) -> Node<'content> {
        let inner = Node::get_inner_data(node);
        let mut new_n = Node::new_empty();
        Node::add_inner(&mut new_n, inner);
        new_n = new_n;
        new_n.right = node.right.clone();
        new_n.left = node.left.clone();
        new_n.parent = node.parent.clone();
        new_n
    }
}

pub(crate) struct BinaryT<'content> {
    pub(crate) root_index: Option<NodeIndex>,
    pub(crate) index_node_dict: HashMap<NodeIndex, Node<'content>>,
    node_index_dict: HashMap<Node<'content>, NodeIndex>,
    all_samples: HashSet<ArcStr<'content>>,
    clustered_leaf_nodes: HashSet<ArcStr<'content>>,
    sub_tree_roots_index: HashSet<NodeIndex>,
    len: usize,
}

impl<'content> BinaryT<'content> {
    pub(crate) fn new(
        index_node_d: HashMap<NodeIndex, Node<'content>>,
        node_index_d: HashMap<Node<'content>, NodeIndex>,
        all_samples: HashSet<ArcStr<'content>>,
    ) -> Self {
        let len = index_node_d.len();
        BinaryT {
            root_index: None,
            index_node_dict: index_node_d,
            node_index_dict: node_index_d,
            all_samples: all_samples,
            clustered_leaf_nodes: HashSet::new(),
            sub_tree_roots_index: HashSet::new(),
            len: len,
        }
    }

    /// Given two `Node` this method will merging them and generate a
    /// new parent `Node` and finally return the index of parent node
    fn merge(&mut self, mut left: Node<'content>, mut right: Node<'content>) -> NodeIndex {
        let right_len = Node::inner_data_size(&right);
        let left_len = Node::inner_data_size(&left);
        let left_index = self.node_index_dict.get(&left);
        let right_index = self.node_index_dict.get(&right);
        if left_index.is_none() | right_index.is_none() {
            panic!("Looks like not all the input nodes exists in the `node_index_dict`");
        }
        let left_index = *left_index.unwrap();
        let right_index = *right_index.unwrap();
        let parent_index: NodeIndex = self.node_index_dict.len() + 1;
        let mut parent = Node::new_empty();
        Node::set_parent(&mut left, parent_index);
        Node::set_parent(&mut right, parent_index);
        Node::set_left(&mut parent, left_index);
        Node::set_right(&mut parent, right_index);
        let left_inner = Node::get_inner_data(&left);
        let right_inner = Node::get_inner_data(&right);
        Node::add_inner(&mut parent, left_inner);
        Node::add_inner(&mut parent, right_inner);
        if left_len == 1 {
            self.clustered_leaf_nodes.insert(Node::get_left_str(&left));
        }
        if right_len == 1 {
            self.clustered_leaf_nodes.insert(Node::get_left_str(&right));
        }

        self.node_index_dict.insert(Node::clone(&left), left_index);
        self.node_index_dict
            .insert(Node::clone(&right), right_index);
        self.node_index_dict
            .insert(Node::clone(&parent), parent_index);

        self.index_node_dict.insert(left_index, left);
        self.index_node_dict.insert(right_index, right);
        self.index_node_dict.insert(parent_index, parent);
        self.len += 3;
        self.sub_tree_roots_index.insert(parent_index);
        parent_index
    }

    /// Get those unclustered leaf `ArcStr`
    #[inline]
    fn get_unclustered_leafs(&self) -> Vec<ArcStr<'content>> {
        self.all_samples
            .difference(&self.clustered_leaf_nodes)
            .map(|x| x.clone())
            .collect()
    }

    /// Given an index of node and return the node
    fn get_node(&self, index: NodeIndex) -> Option<&Node<'content>> {
        self.index_node_dict.get(&index)
    }

    pub(crate) fn updating(
        &mut self,
        left_index: NodeIndex,
        right_index: NodeIndex,
        max_heap: &mut MaxHeap<f64, HeapPair>,
        pair_value: &BTree<ArcStr<'content>, BTree<ArcStr<'content>, f64>>,
    ) {
        let left_node = self.index_node_dict.get(&left_index);
        let right_node = self.index_node_dict.get(&right_index);
        if left_node.is_none() | right_node.is_none() {
            panic!("The input index of Node are not exists");
        }
        let left_node = left_node.unwrap();
        let right_node = right_node.unwrap();
        let left_is_clustered = Node::get_parent_idx(left_node).is_some();
        let right_is_clustered = Node::get_parent_idx(right_node).is_some();
        if left_is_clustered | right_is_clustered {
            return;
        }
        let new_centroid_idx = self.merge(Node::clone(left_node), Node::clone(right_node));
        let new_centroid = self.get_node(new_centroid_idx);
        let remained_leaf_centroids = self.get_unclustered_leafs();
        if let Some(n) = new_centroid {
            if Node::inner_data_size(n) == self.all_samples.len() {
                // The new centroid node contains all the leafs str
                // so the clustering is done
                self.root_index = Some(new_centroid_idx);
                return;
            } else {
                let new_values = self.calculate_parallel(&remained_leaf_centroids, n, pair_value);
                new_values.into_iter().for_each(|o| {
                    if let Some(o) = o {
                        max_heap.insert(o.0, HeapPair::new(o.1, o.2));
                    }
                });
            }
        }
    }

    /// Get the clustered sub-tree root nodes
    #[inline]
    fn get_sub_tree_roots(&self) -> Vec<Option<&Node>> {
        self.sub_tree_roots_index
            .iter()
            .map(|idx| self.index_node_dict.get(idx))
            .collect::<Vec<_>>()
    }

    /// Give a Vec of `ArcStr` this method will generate leaf nodes
    #[inline]
    fn str_to_leaf_nodes(&self, inputs: &Vec<ArcStr<'content>>) -> Vec<Node<'content>> {
        inputs
            .iter()
            .map(|i| Node::new_from_str(i.clone()))
            .collect::<Vec<_>>()
    }

    fn calculate_parallel(
        &self,
        remained_leaf_centroids: &Vec<ArcStr<'content>>,
        new_centroid: &Node<'content>,
        pair_values: &BTree<ArcStr<'content>, BTree<ArcStr<'content>, f64>>,
    ) -> Vec<Option<(f64, NodeIndex, NodeIndex)>> {
        let remained_leaf_nodes = self.str_to_leaf_nodes(remained_leaf_centroids);
        let mut sub_tree_roots = self.get_sub_tree_roots();
        sub_tree_roots.retain(|x| x.is_some());
        let sub_tree_roots = sub_tree_roots
            .into_iter()
            .map(|x| x.unwrap())
            .filter(|x| Node::get_parent_idx(*x).is_none())
            .collect::<Vec<_>>();

        let mut o1 = remained_leaf_nodes
            .iter()
            .filter(|x| !Node::is_equal(*x, new_centroid))
            .map(|c| self.calculate_two(c, new_centroid, pair_values))
            .collect::<Vec<_>>();

        let o2 = sub_tree_roots
            .iter()
            .filter(|x| !Node::is_equal(**x, new_centroid))
            .map(|c| self.calculate_two(*c, new_centroid, pair_values))
            .collect::<Vec<_>>();
        o1.extend(o2);
        o1
    }

    fn calculate_two(
        &self,
        left_node: &Node,
        right_node: &Node,
        pair_value_dict: &BTree<ArcStr<'content>, BTree<ArcStr<'content>, f64>>,
    ) -> Option<(f64, NodeIndex, NodeIndex)> {
        let mut sum = 0.0;
        let mut size: usize = 0;
        for l in left_node.data.iter() {
            for r in right_node.data.iter() {
                let out_lr = pair_value_dict.get(l).and_then(|inner| inner.get(r));
                let out_rl = pair_value_dict.get(r).and_then(|inner| inner.get(l));
                match (out_lr, out_rl) {
                    (Some(o), None) => {
                        sum += *o;
                        size += 1;
                    }
                    (None, Some(o)) => {
                        sum += *o;
                        size += 1;
                    }
                    (_, _) => {}
                };
            }
        }
        if size == 0 {
            None
        } else {
            let out_value = sum / size as f64;
            let left_idx = self.get_node_index(left_node);
            let right_idx = self.get_node_index(right_node);
            match (left_idx, right_idx) {
                (Some(left_idx), Some(right_idx)) => Some((out_value, left_idx, right_idx)),
                (_, _) => None,
            }
        }
    }

    /// Give a node return it's index in the `node_index_dict`
    fn get_node_index(&self, node: &Node) -> Option<NodeIndex> {
        self.node_index_dict.get(node).map(|x| *x)
    }

    pub(crate) fn to_newick(&self) -> String {
        if self.root_index.is_none() {
            panic!("root node is none, which means the clustering is not done");
        }
        let root_n = self.get_node(self.root_index.unwrap());
        let mut nodes: VecDeque<Option<&Node>> = VecDeque::from_iter([root_n]);
        let mut out = "".to_string();
        loop {
            let node = nodes.pop_front();
            match node {
                None => {
                    if !nodes.is_empty() {
                        continue;
                    } else {
                        break out;
                    }
                }
                Some(cur_node) => {
                    let cur_node = cur_node.unwrap();
                    let cur_size = Node::inner_data_size(cur_node);
                    if cur_size < 2 {
                        continue;
                    }
                    let left_idx = cur_node.left.unwrap();
                    let right_idx = cur_node.right.unwrap();
                    let left_n = self.get_node(left_idx);
                    let right_n = self.get_node(right_idx);
                    nodes.push_back(left_n);
                    nodes.push_back(right_n);
                    let inner_data = Node::get_inner_data(cur_node);
                    let cur_str = inner_data.iter().map(|x| **x).collect::<Vec<_>>().join(",");
                    if out.len() == 0 {
                        out = format!("({});", cur_str);
                    } else {
                        let replace_str = format!("({})", cur_str);
                        out = out.replace(cur_str.as_str(), replace_str.as_str());
                    }
                }
            }
        }
    }
}
