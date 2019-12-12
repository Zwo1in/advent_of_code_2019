use regex::Regex;
use itertools::Itertools;
use std::fmt;
use std::collections::HashMap;

pub const INPUT: &'static str = include_str!("../input");

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
struct Vec3 {
    x: i128,
    y: i128,
    z: i128,
}

impl Vec3 {
    fn zero() -> Self {
        Self { x: 0, y: 0, z: 0 }
    }

    fn from_vec(input: Vec<i128>) -> Self {
        assert_eq!(input.len(), 3);
        Vec3 {
            x: input[0],
            y: input[1],
            z: input[2],
        }
    }

    fn sum_abs(&self) -> i128 {
        self.x.abs() + self.y.abs() + self.z.abs()
    }
}

impl std::ops::AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl fmt::Display for Vec3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "x: {0: >4}, y: {1: >4}, z: {2: >4}", self.x, self.y, self.z)
    }
}

fn parse_input(input: &'static str) -> Vec<Vec3> {
    let num = Regex::new(r"-?\d+").unwrap();
    input.lines().map(|line| {
        num.captures_iter(line)
            .map(|cap| {
                let num = cap.get(0).unwrap();
                num.as_str().parse::<i128>().unwrap()
            })
            .collect::<Vec<_>>()
        })
        .map(Vec3::from_vec)
        .collect()
}

#[derive(PartialEq, Eq, Debug, Clone)]
struct Moon {
    pos: Vec3,
    vel: Vec3,
}

impl Moon {
    fn potential_energy(&self) -> i128 {
        self.pos.sum_abs()
    }
    
    fn kinetic_energy(&self) -> i128 {
        self.vel.sum_abs()
    }
}

impl Default for Moon {
    fn default() -> Self {
        Self {
            pos: Vec3::zero(),
            vel: Vec3::zero(),
        }
    }
}

#[derive(Debug)]
pub struct Tracker { moons: HashMap<String, Moon> }

impl Tracker {
    pub fn step(&mut self) {
        let mut next_state = self.moons.clone();
        self.moons.iter()
            .cartesian_product(self.moons.iter())
            .filter(|((name_lhs, _), (name_rhs, _))| name_lhs != name_rhs)
            .for_each(|((name, lhs), (_, rhs))| {
                use std::cmp::Ordering::*;
                let pos_lhs = [lhs.pos.x, lhs.pos.y, lhs.pos.z];
                let pos_rhs = [rhs.pos.x, rhs.pos.y, rhs.pos.z];
                next_state.get_mut(name).unwrap().vel += Vec3::from_vec(
                    pos_lhs.iter()
                        .zip(pos_rhs.iter())
                        .map(|(lhs, rhs)| {
                            match lhs.cmp(&rhs) {
                                Less    => 1,
                                Equal   => 0,
                                Greater => -1,
                            }
                        })
                        .collect() 
                    );
            });
        next_state.iter_mut().for_each(|(_, moon)| moon.pos += moon.vel);
        self.moons = next_state;
    }

    pub fn total_energy(&self) -> i128 {
        self.moons.values()
            .map(|moon| moon.potential_energy() * moon.kinetic_energy())
            .fold1(|acc, val| acc + val)
            .unwrap()
    }
}

impl fmt::Display for Tracker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let moons: String = self.moons.iter()
            .map(|(name, moon)| format!("{: <8}  <Pos {}>  <Vel {}>", name, moon.pos, moon.vel))
            .intersperse("\n".to_owned())
            .collect();
        write!(f, "{}\n", moons)
    }
}

pub fn prepare_tracker(data: &'static str) -> Tracker {
    let positions = parse_input(data);
    let moons = positions.iter().zip(["Io", "Europa", "Ganymede", "Callisto"].into_iter())
        .map(|(&pos, &name)| {
            (name.to_owned(), Moon { pos, ..Default::default() })
        })
        .collect();
    Tracker { moons }
}

fn main() {
    let positions = parse_input(INPUT);
    let moons: HashMap<_, _> = positions.iter().zip(["Io", "Europa", "Ganymede", "Callisto"].into_iter())
        .map(|(&pos, &name)| {
            (name.to_owned(), Moon { pos, ..Default::default() })
        })
        .collect();

    let mut tracker = Tracker { moons: moons.clone() };
    println!("{}", tracker);
    (0..1000).for_each(|_| tracker.step());
    println!("{}", tracker);
    println!("Total energy: {}", tracker.total_energy());

    let mut n = 0u128;
    let mut tracker = Tracker { moons };
    loop {
        n += 1;
        tracker.step();
        if positions.iter().zip(tracker.moons.values()).all(|(&lhs, rhs)| lhs == rhs.pos && rhs.vel == Vec3::zero()) {
            println!("Tracker in beggining state after {} steps\n{}", n, tracker);
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE1: &'static str = "<x=-8, y=-10, z=0>
                                   <x=5,  y=5,   z=10>
                                   <x=2,  y=-7,  z=3>
                                   <x=9,  y=-8,  z=-3>";

    #[test]
    fn parsing_input() {
        assert_eq!(
            parse_input(INPUT),
            [Vec3 { x: -9, y: -1, z: -1 },
             Vec3 { x:  2, y:  9, z:  5 },
             Vec3 { x: 10, y: 18, z:-12 },
             Vec3 { x: -6, y: 15, z: -7 }],
        );
    }

    #[test]
    fn step_prediction() {
        let mut tracker = prepare_tracker(SAMPLE1);
        (0..10).for_each(|_| tracker.step());
        assert_eq!(
            tracker.moons["Europa"],
            Moon {
                pos: Vec3 { x: 4, y: 10, z: 9 },
                vel: Vec3 { x: -3, y: 7, z: -2 },
            }
        );
        assert_eq!(
            tracker.moons["Callisto"],
            Moon {
                pos: Vec3 { x: 5, y: -10, z: 3 },
                vel: Vec3 { x: 0, y: -4, z: 5 },
            }
        );
    }

    #[test]
    fn total_energy_calculation() {
        let mut tracker = prepare_tracker(SAMPLE1);
        (0..20).for_each(|_| tracker.step());
        assert_eq!(tracker.total_energy(), 119+216+135+32);
    }
}

