use vec_tree::{VecTree, Index};
use std::collections::HashMap;
const INPUT: &'static str = include_str!("../input");

pub fn parse_orbits(input: &'static str) -> HashMap<&'static str, &'static str> {
    let mut map: HashMap<&'static str, &'static str> = HashMap::new();
    input.lines().for_each(|line| {
        let mut splitted = line.trim().split(")").map(|i| i.trim());
        let (orbiter, orbited) = (splitted.next().unwrap(),
                                  splitted.next().unwrap());
        map.insert(orbited, orbiter);
    });
    map
}

pub fn form_tree(
    parent: &'static str,
    map: &HashMap<&'static str, &'static str>,
    indexer: &mut HashMap<&'static str, Index>,
    tree: &mut VecTree<&'static str>
) {
    map.iter()
        .filter(|(_, v)| *v == &parent)
        .for_each(|(k, _)| {
            let index = tree.insert(k, indexer[parent]);
            indexer.insert(k, index);
            form_tree(k, map, indexer, tree);
        });
}

pub fn ancestors_count(desired: Index, tree: &VecTree<&'static str>, idx: Index) -> i32 {
    let mut count = 0;
    let mut idx = idx;
    while let Some(p) = tree.parent(idx) {
        count += 1;
        idx = p;
        if p == desired {
            break;
        }
    }
    count
}

pub fn count_orbits(root: &'static str, tree: &VecTree<&'static str>, indexer: &HashMap<&'static str, Index>) -> i32 {
    let mut count = 0;
    indexer.iter().filter(|(k, _)| *k != &root)
        .for_each(|(_, v)| {
            count += ancestors_count(indexer[root], tree, *v);
        });
    count
}

pub fn shortest_path_len(a: Index, b: Index, tree: &VecTree<&'static str>) -> i32 {
    let mut closest_common_parent = None;
    let mut a_idx = a;
    'outer: while let Some(p) = tree.parent(a_idx) {
        let mut b_idx = b;
        'inner: while let Some(q) = tree.parent(b_idx) {
            if q == p {
                closest_common_parent = Some(q);
                break 'outer;
            }
            b_idx = q;
        }
        a_idx = p;
    }
    let closest_common_parent = closest_common_parent.unwrap();
    let a_count = ancestors_count(closest_common_parent, &tree, a);
    let b_count = ancestors_count(closest_common_parent, &tree, b);
    a_count + b_count - 2 // 1 because parent is common and 1 because we are counting connections not nodes
}

fn main() {
    let map = parse_orbits(INPUT);
    let mut tree = VecTree::new();
    let mut indexer = HashMap::new();
    let idx = tree.insert_root("COM");
    indexer.insert("COM", idx);
    form_tree("COM", &map, &mut indexer, &mut tree);

    println!("{}", shortest_path_len(indexer["YOU"], indexer["SAN"], &tree));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn prepare_tree(root: &'static str, input: &'static str) -> (HashMap<&'static str, Index>, VecTree<&'static str>) {
        let map = parse_orbits(input);
        let mut tree = VecTree::new();
        let mut indexer = HashMap::new();
        let idx = tree.insert_root(root);
        indexer.insert(root, idx);
        form_tree(root, &map, &mut indexer, &mut tree);
        (indexer, tree)
    }

    #[test]
    fn parsing_test() {
        let input = "A)B\nB)C\nB)D";
        assert_eq!(parse_orbits(input), [("B", "A"),
                                         ("C", "B"),
                                         ("D", "B")].iter().cloned().collect());
    }

    #[test]
    fn form_tree_test() {
        let input = "A)B\nB)C\nB)D";
        let (indexer, tree) = prepare_tree("A", input);

        let b_childs = tree.children(indexer["B"]).map(|node| tree[node]).collect::<HashSet<&str>>();
        assert_eq!(b_childs, ["D", "C"].iter().cloned().collect());
    }

    #[test]
    fn ancestors_count_test() {
        let input = "A)B\nB)C\nB)D";
        let (indexer, tree) = prepare_tree("A", input);

        assert_eq!(ancestors_count(indexer["A"], &tree, indexer["D"]), 2);
    }

    #[test]
    fn orbits_count_test() {
        let input = "A)B\nB)C\nB)D";
        let (indexer, tree) = prepare_tree("A", input);

        assert_eq!(count_orbits("A", &tree, &indexer), 5);
    }

    #[test]
    fn shortest_path_len_test() {
        let input = "A)B\nB)C\nB)D\nD)E\nC)F";
        let (indexer, tree) = prepare_tree("A", input);

        assert_eq!(shortest_path_len(indexer["F"], indexer["E"], &tree), 2);
    }

    #[test]
    fn shortest_path_len_test_2() {
        let input = "COM)B\nB)C\nC)D\nD)E\nE)F\nB)G\nG)H\nD)I\nE)J\nJ)K\nK)L\nK)YOU\nI)SAN";
        let (indexer, tree) = prepare_tree("COM", input);

        assert_eq!(shortest_path_len(indexer["YOU"], indexer["SAN"], &tree), 4);
    }
}
