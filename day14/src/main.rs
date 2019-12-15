#![allow(unused)]
use log::{info, debug};
use std::collections::{HashSet, HashMap};
use std::cell::RefCell;
use trees::{tr, Tree, Node};

use itertools::Itertools;


const INPUT: &'static str = include_str!("../input");

fn parse_input(input: &'static str) -> Formula {
    Formula(input.lines().flat_map(|line| {
        line.trim().split("=>")
            .tuples()
            .map(|(components, product)| {
                let parse = |ingredient: &str| {
                    ingredient
                        .trim().split(" ").tuples().map(|(quantity, component)| {
                            Ingredient::new(
                                component,
                                quantity.parse::<u32>().unwrap()
                            )
                        })
                        .next().unwrap()
                };
                (parse(product), components.trim().split(',').map(parse).collect::<HashSet<_>>())

            })
        })
        .collect())
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct Ingredient {
    name: String,
    quantity: u32,
}

impl Ingredient {
    fn new(name: &str, quantity: u32) -> Self {
        let name = name.to_owned();
        Self { name, quantity }
    }

    fn node(&self, qty_produced: u32) -> IngredientNode {
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
    needed: u32,
    qty_produced: u32,
}

impl IngredientNode {
    fn reactions(&self, needed: u32, surplus: &mut u32) -> u32 {
        let surplus_usage = *surplus / self.qty_produced;
        let reactions = needed / self.qty_produced
                + if needed.rem_euclid(self.qty_produced) != 0 { 1 } else { 0 };
        let left = reactions.checked_sub(surplus_usage).unwrap_or(0);
        *surplus -= (reactions - left) * self.qty_produced;
        left
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct Amounts {
    produced: u32,
    surplus: u32,
}

impl Amounts {
    fn new(produced: u32, surplus: u32) -> Self {
        Self { produced, surplus }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Formula(HashMap<Ingredient, HashSet<Ingredient>>);

impl Formula {
    fn create_tree(&self) -> Tree<RefCell<IngredientNode>> {
        let root = self.get_key(&"FUEL".to_owned()).unwrap().node(1);
        let mut tree = tr(RefCell::new(root));
        self.add_children(&mut tree);
        tree
    }

    fn add_children(&self, node: &mut Tree<RefCell<IngredientNode>>) {
        let try_children = self.get(&node.data.borrow().name);
        if let Some(children) = try_children {
            for child in children {
                let mut child_node = tr(
                    RefCell::new(child.node(
                        self.get_amount_produced(&child.name)
                            .unwrap_or(1)
                    ))
                );
                self.add_children(&mut child_node);
                *node = node.clone() / child_node;
            }
        } else {
            return
        }
    }

    fn create_need_table(&self, alchemy_tree: &Tree<RefCell<IngredientNode>>) -> HashMap<String, Amounts> {
        let mut map = HashMap::new();
        info!("Started creating need table");
        self.calc_child_needs(alchemy_tree.root(), &mut map);
        map
    }

    fn calc_child_needs(
        &self,
        node: &Node<RefCell<IngredientNode>>,
        need_table: &mut HashMap<String, Amounts>
    ) {
        info!("Calculating needs for {}", node.data.borrow().name);
        if node.is_leaf() {
            info!("Node is leaf");
            return
        }
        for child in node.iter() {
            info!("Processing child: {}", child.data.borrow().name);
            let multiplier = self.evaluate(&node.data.borrow(), &child.data.borrow(), need_table);
            info!("Evaluation finished, current child amount: produced = {}, surplus = {}",
                need_table.get(&child.data.borrow().name).unwrap().produced,
                need_table.get(&child.data.borrow().name).unwrap().surplus,
            );
            child.data.borrow_mut().needed *= multiplier;
            self.calc_child_needs(child, need_table);
        }
    }

    fn evaluate(
        &self,
        target: &IngredientNode,
        component: &IngredientNode,
        amounts_table: &mut HashMap<String, Amounts>,
    ) -> u32 {
        info!("{}->{} Target: needed = {}, qty_produced = {}", target.name, component.name, target.needed, target.qty_produced);
        info!("{}->{} Component: needed = {}, qty_produced = {}", target.name, component.name, component.needed, component.qty_produced);
        let mut target_reactions = 0;
        {
            let current_target_amount = amounts_table.entry(target.name.clone())
                                                     .or_insert(Amounts::new(0, 0));
            info!("{}->{} Target amounts: produced = {}, surplus = {}", target.name, component.name,
                current_target_amount.produced,
                current_target_amount.surplus,
            );
            target_reactions = target.reactions(target.needed, &mut current_target_amount.surplus);
            info!("{}->{} Target reactions needed: {}", target.name, component.name, target_reactions);
        }
        let current_comp_amount = amounts_table.entry(component.name.clone())
                                               .or_insert(Amounts::new(0, 0));
        info!("{}->{} Component amounts: produced = {}, surplus = {}", target.name, component.name,
            current_comp_amount.produced,
            current_comp_amount.surplus,
        );
        let component_need = component.needed * target_reactions;
        info!("{}->{} Components needed: {}", target.name, component.name, component_need);
        let component_reactions = component.reactions(component_need, &mut current_comp_amount.surplus);
        info!("{}->{} Components reactions needed: {}", target.name, component.name, component_reactions);
        let component_produced = component.qty_produced * component_reactions;
        info!("{}->{} Components produced: {}", target.name, component.name, component_produced);
        current_comp_amount.produced += component_produced;
        current_comp_amount.surplus += component_produced.checked_sub(component_need)
                                                         .unwrap_or(0);
        target_reactions
    }

    fn get(&self, key: &String) -> Option<&HashSet<Ingredient>> {
        self.0.iter().filter_map(|(k, v)| if k.name == *key { Some(v) } else { None })
            .next()
    }

    fn get_key(&self, key: &String) -> Option<Ingredient> {
        self.0.keys().filter(|k| k.name == *key).next().cloned()
    }

    fn get_amount_produced(&self, key: &String) -> Option<u32> {
        if let Some(item) = self.get_key(key) {
            Some(item.quantity)
        } else {
            None
        }
    }
}

fn main() {
    env_logger::init();
    let input = include_str!("../test/ore_necessity_2");
    let formula = parse_input(INPUT);
    let tree = formula.create_tree();
    assert_eq!(formula.create_need_table(&tree)["ORE"].produced, 180697);
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use maplit::{hashmap, hashset};
    use super::*;
    const TINY_RECIEPE: &'static str = include_str!("../test/input_small");

    fn prepare_logger() {
        env_logger::init();
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
    fn ore_necessity_0() {
        //prepare_logger();
        let input = include_str!("../test/ore_necessity_0");
        let formula = parse_input(input);
        let tree = formula.create_tree();
        assert_eq!(formula.create_need_table(&tree)["ORE"].produced, 165);
    }

    #[test]
    fn ore_necessity_1() {
        //prepare_logger();
        let input = include_str!("../test/ore_necessity_1");
        let formula = parse_input(input);
        let tree = formula.create_tree();
        assert_eq!(formula.create_need_table(&tree)["ORE"].produced, 13312);
    }

    #[test]
    fn ore_necessity_2() {
        //prepare_logger();
        let input = include_str!("../test/ore_necessity_2");
        let formula = parse_input(input);
        let tree = formula.create_tree();
        assert_eq!(formula.create_need_table(&tree)["ORE"].produced, 180697);
    }

    #[test]
    fn ore_necessity_3() {
        //prepare_logger();
        let input = include_str!("../test/ore_necessity_3");
        let formula = parse_input(input);
        let tree = formula.create_tree();
        assert_eq!(formula.create_need_table(&tree)["ORE"].produced, 2210736);
    }
}
