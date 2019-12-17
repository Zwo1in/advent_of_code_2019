use log::{info, debug};
use std::collections::{HashMap, HashSet};
use trees::{tr, Node, Tree};

use itertools::Itertools;
use maplit::{hashmap, hashset};

const INPUT: &'static str = include_str!("../input");

fn parse_input(input: &'static str) -> Formula {
    Formula(
        input
            .lines()
            .flat_map(|line| {
                line.trim()
                    .split("=>")
                    .tuples()
                    .map(|(components, product)| {
                        let parse = |ingredient: &str| {
                            ingredient
                                .trim()
                                .split(" ")
                                .tuples()
                                .map(|(quantity, component)| {
                                    Ingredient::new(component, quantity.parse::<u64>().unwrap())
                                })
                                .next()
                                .unwrap()
                        };
                        (
                            parse(product),
                            components
                                .trim()
                                .split(',')
                                .map(parse)
                                .collect::<HashSet<_>>(),
                        )
                    })
            })
            .collect(),
    )
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct Ingredient {
    name: String,
    quantity: u64,
}

impl Ingredient {
    fn new(name: &str, quantity: u64) -> Self {
        let name = name.to_owned();
        Self { name, quantity }
    }

    fn node(&self, qty_produced: u64) -> IngredientNode {
        IngredientNode {
            name: self.name.clone(),
            needed: self.quantity,
            qty_produced,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct IngredientNode {
    name: String,
    needed: u64,
    qty_produced: u64,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct Amounts {
    produced: u64,
    surplus: u64,
    reactions: u64,
}

impl Amounts {
    fn new(produced: u64, surplus: u64, reactions: u64) -> Self {
        Self {
            produced,
            surplus,
            reactions,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Formula(HashMap<Ingredient, HashSet<Ingredient>>);
type AlchemyTree = Tree<IngredientNode>;
type AlchemyNode = Node<IngredientNode>;

impl Formula {
    fn create_tree(&self) -> AlchemyTree {
        let root = self.get_key(&"FUEL".to_owned()).unwrap().node(1);
        let mut tree = tr(root);
        self.add_children(&mut tree);
        tree
    }

    fn add_children(&self, node: &mut AlchemyTree) {
        let try_children = self.get(&node.data.name);
        if let Some(children) = try_children {
            for child in children {
                let mut child_node =
                    tr(child.node(self.get_amount_produced(&child.name).unwrap_or(1)));
                self.add_children(&mut child_node);
                *node = node.clone() / child_node;
            }
        } else {
            return;
        }
    }

    fn get(&self, key: &String) -> Option<&HashSet<Ingredient>> {
        self.0
            .iter()
            .filter_map(|(k, v)| if k.name == *key { Some(v) } else { None })
            .next()
    }

    fn get_key(&self, key: &String) -> Option<Ingredient> {
        self.0.keys().filter(|k| k.name == *key).next().cloned()
    }

    fn get_amount_produced(&self, key: &String) -> Option<u64> {
        if let Some(item) = self.get_key(key) {
            Some(item.quantity)
        } else {
            None
        }
    }
}

fn reduce_tree(tree: &AlchemyTree, unique_components: &HashSet<String>) -> Amounts {
    let root = tree.root().data.clone();
    let mut mapper = hashmap! {"FUEL".to_owned() => Amounts::new(root.needed, 0, root.needed)};
    let mut diff = difference(&mapper, unique_components).clone();
    while diff != hashset! {} {
        let ready = find_ready_to_map(&tree, &mapper, unique_components);
        for key in ready {
            let amounts = evaluate(key.clone(), &tree, &mapper);
            debug!("ReduceTree -> Adding {}, <{}, {}, {}>", key, amounts.produced, amounts.surplus, amounts.reactions);
            mapper.insert(key.clone(), amounts);
        }
        diff = difference(&mapper, unique_components).clone();
    }
    mapper["ORE"]
}

fn find_ready_to_map<'a>(
    tree: &'a AlchemyNode,
    mapper: &'a HashMap<String, Amounts>,
    unique_components: &'a HashSet<String>,
) -> HashSet<String> {
    difference(mapper, unique_components)
        .iter()
        .filter_map(|name| {
            if is_ready_to_map(find_nodes(tree, name.clone()), mapper) {
                Some(name.clone())
            } else {
                None
            }
        })
        .inspect(|node| debug!("Ready to map: {}", node))
        .collect()
}

/// Find nodes which name is equal to 'name'
fn find_nodes<'a>(tree: &'a AlchemyNode, name: String) -> Vec<&'a AlchemyNode> {
    if tree.is_leaf() {
        return vec![];
    }
    let mut ret = vec![];
    for child in tree.iter() {
        if child.data.name == name.clone() {
            ret.push(child);
        }
        ret.append(&mut find_nodes(child, name.clone()));
    }
    return ret;
}

fn is_ready_to_map(nodes: Vec<&AlchemyNode>, mapper: &HashMap<String, Amounts>) -> bool {
    nodes.iter().all(|&node| {
        let mut current = node;
        while let Some(parent) = current.parent() {
            if !mapper.contains_key(&parent.data.name.clone()) {
                return false;
            } else {
                current = parent;
            }
        }
        true
    })
}

fn evaluate(key: String, tree: &AlchemyTree, mapper: &HashMap<String, Amounts>) -> Amounts {
    let needed: u64 = find_nodes(tree, key.clone())
        .iter()
        .sorted_by_key(|node| parent_name(node))
        .into_iter()
        .dedup_by(|lhs, rhs| parent_name(lhs) == parent_name(rhs))
        .map(|node| {
            let reactions = mapper[parent_name(node)].reactions;
            node.data.needed * reactions
        })
        .sum();
    let produced = find_nodes(tree, key.clone())
        .iter()
        .next()
        .unwrap()
        .data
        .qty_produced;
    let operations = needed / produced
        + if needed.rem_euclid(produced) != 0 {
            1
        } else {
            0
        };
    let surplus = operations * produced - needed;
    Amounts::new(operations*produced, surplus, operations)
}

fn parent_name(node: &AlchemyNode) -> &str {
    node.parent().expect("Node is root").data.name.as_str()
}

fn difference<'a>(
    hashmap: &'a HashMap<String, Amounts>,
    hashset: &'a HashSet<String>,
) -> HashSet<String> {
    hashset
        .difference(&hashmap.keys().cloned().collect::<HashSet<_>>())
        .cloned()
        .collect()
}

fn get_unique_components(formula: &Formula) -> HashSet<String> {
    let mut unique_components: HashSet<_> = formula.0.keys().map(|k| k.name.clone()).collect();
    unique_components.insert("ORE".to_owned());
    unique_components
}

fn fuel_amount_for_ores(ores: u64, alchemy_formula: &Formula) -> u64 {
    let mut tree = alchemy_formula.create_tree();
    let unique_components = get_unique_components(&alchemy_formula);
    let mut current = ores / reduce_tree(&tree, &unique_components).produced;
    let step = 20000;
    let mut previous;
    let mut ores_needed;
    info!("Finding range for bs");
    info!("\tCurrent: {}, Step: 20000", current);
    'find_range: loop {
        previous = current;
        current += step;
        tree.root_mut().data.needed = current;
        ores_needed = reduce_tree(&tree, &unique_components).produced;
        if ores_needed >= ores {
            info!("Ores exceeded");
            info!("\tPrevious: {}, Current: {}", previous, current);
            break;
        }
    }
    let (mut lb, mut ub) = (previous, current);
    info!("Binary search in {} - {}", lb, ub);
    let half = |l: u64, u: u64| l + (u - l)/2;
    'bin_search: loop {
        current = half(lb, ub);
        tree.root_mut().data.needed = current;
        info!("Current: {}", current);
        ores_needed = reduce_tree(&tree, &unique_components).produced;
        if ores_needed <= ores {
            info!("Ores too low, {}", ores_needed);
            lb = current;
        } else {
            info!("Ores exceeded, {}", ores_needed);
            ub = current;
        }
        info!("Lower bound: {}, Upper bound: {}", lb, ub);
        if lb == ub-1 {
            info!("Returning {}", lb);
            break lb
        }
    }
}

fn main() {
    env_logger::init();
    let formula = parse_input(INPUT);
    let unique_components = get_unique_components(&formula);
    let tree = formula.create_tree();
    println!("Ores for fuel: {}", reduce_tree(&tree, &unique_components).produced);

    println!("Max fuel created: {}", fuel_amount_for_ores(1000000000000, &formula))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    const TINY_RECIEPE: &'static str = include_str!("../test/input_small");

    fn prepare_logger() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn parsing_input() {
        //prepare_logger();
        assert_eq!(
            parse_input(TINY_RECIEPE).0,
            hashmap! {
                Ingredient::new("A", 10) =>
                 hashset!{Ingredient::new("ORE", 10)},
                Ingredient::new("B", 1) =>
                 hashset!{Ingredient::new("ORE", 1)},
                Ingredient::new("C", 1) =>
                 hashset!{Ingredient::new("A", 7), Ingredient::new("B", 1)},
                Ingredient::new("D", 1) =>
                 hashset!{Ingredient::new("A", 7), Ingredient::new("C", 1)},
                Ingredient::new("E", 1) =>
                 hashset!{Ingredient::new("A", 7), Ingredient::new("D", 1)},
                Ingredient::new("FUEL", 1)  =>
                 hashset!{Ingredient::new("A", 7), Ingredient::new("E", 1)},
            }
        )
    }

    #[test]
    fn finding_nodes() {
        let formula = parse_input(TINY_RECIEPE);
        let tree = formula.create_tree();
        let target = tr(IngredientNode { name: "E".to_owned(), needed: 1, qty_produced: 1 });
        assert_eq!(
            find_nodes(&tree, "E".to_owned()).iter().next().unwrap().data,
            hashset! {target.root()}.iter().next().unwrap().data,
        );
    }

    #[test]
    fn diff() {
        assert_eq!(
            difference(
                &hashmap! {
                    "A".to_owned() => Amounts::new(0,0,0),
                    "B".to_owned() => Amounts::new(0,0,0),
                    "C".to_owned() => Amounts::new(0,0,0),
                    "D".to_owned() => Amounts::new(0,0,0),
                },
                &hashset! {
                    "A".to_owned(),
                    "B".to_owned(),
                    "C".to_owned(),
                    "D".to_owned(),
                    "E".to_owned(),
                    "F".to_owned(),
                    "G".to_owned(),
                }
            ),
            hashset! {
                "E".to_owned(),
                "F".to_owned(),
                "G".to_owned(),
            }
        );
        assert_eq!(
            difference(
                &hashmap! {
                    "A".to_owned() => Amounts::new(0,0,0),
                    "B".to_owned() => Amounts::new(0,0,0),
                    "C".to_owned() => Amounts::new(0,0,0),
                    "D".to_owned() => Amounts::new(0,0,0),
                },
                &hashset! {
                    "A".to_owned(),
                    "B".to_owned(),
                    "C".to_owned(),
                    "D".to_owned(),
                }
            ),
            hashset! {}
        );
    }

    #[test]
    fn finding_ready_to_map() {
        let formula = parse_input(TINY_RECIEPE);
        let unique_components = get_unique_components(&formula);
        let mapper = hashmap! {
            "FUEL".to_owned() => Amounts::new(0, 0, 0),
            "E".to_owned() => Amounts::new(0, 0, 0),
        };
        let tree = formula.create_tree();
        assert_eq!(
            find_ready_to_map(&tree, &mapper, &unique_components),
            hashset! {"D".to_owned()},
        );
    }

    #[test]
    fn evaluating() {
        let formula = parse_input(TINY_RECIEPE);
        let mapper = hashmap! {
            "FUEL".to_owned() => Amounts::new(1, 0, 1),
            "E".to_owned() => Amounts::new(1, 0, 1),
        };
        let tree = formula.create_tree();
        assert_eq!(
            evaluate("D".to_owned(), &tree, &mapper),
            Amounts::new(1, 0, 1),
        );
    }

    #[test]
    fn ore_necessity_0() {
        prepare_logger();
        let input = include_str!("../test/ore_necessity_0");
        let formula = parse_input(input);
        let unique_components = get_unique_components(&formula);
        let tree = formula.create_tree();
        assert_eq!(reduce_tree(&tree, &unique_components).produced, 165);
    }

    #[test]
    fn ore_necessity_1() {
        prepare_logger();
        let input = include_str!("../test/ore_necessity_1");
        let formula = parse_input(input);
        let unique_components = get_unique_components(&formula);
        let tree = formula.create_tree();
        assert_eq!(reduce_tree(&tree, &unique_components).produced, 13312);
    }

    #[test]
    fn ore_necessity_2() {
        prepare_logger();
        let input = include_str!("../test/ore_necessity_2");
        let formula = parse_input(input);
        let unique_components = get_unique_components(&formula);
        let tree = formula.create_tree();
        assert_eq!(reduce_tree(&tree, &unique_components).produced, 180697);
    }

    #[test]
    fn ore_necessity_3() {
        prepare_logger();
        let input = include_str!("../test/ore_necessity_3");
        let formula = parse_input(input);
        let unique_components = get_unique_components(&formula);
        let tree = formula.create_tree();
        assert_eq!(reduce_tree(&tree, &unique_components).produced, 2210736);
    }

    #[test]
    fn max_fuel_produced1() {
        prepare_logger();
        let input = include_str!("../test/ore_necessity_1");
        let formula = parse_input(input);
        assert_eq!(fuel_amount_for_ores(1000000000000, &formula), 82892753);
    }

    #[test]
    fn max_fuel_produced2() {
        prepare_logger();
        let input = include_str!("../test/ore_necessity_2");
        let formula = parse_input(input);
        assert_eq!(fuel_amount_for_ores(1000000000000, &formula), 5586022);
    }

    #[test]
    fn max_fuel_produced3() {
        prepare_logger();
        let input = include_str!("../test/ore_necessity_3");
        let formula = parse_input(input);
        assert_eq!(fuel_amount_for_ores(1000000000000, &formula), 460664);
    }
}
