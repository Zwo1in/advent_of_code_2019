#![allow(dead_code, unused_variables)]

use regex::Regex;
use itertools::Itertools;
use std::fmt;

pub const INPUT: &'static str = include_str!("../input");

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
struct Vec3 {
    x: i128,
    y: i128,
    z: i128,
}

impl Vec3 {
    const fn zero() -> Self {
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

pub struct Tracker { moons: [Moon; 4] }

impl Tracker {
    pub fn step(&mut self) {
        let mut i = 0;
        let mut next_state = self.moons.clone();
        self.moons.iter()
            .cartesian_product(self.moons.iter())
            .filter(|(lhs, rhs)| lhs != rhs)
            .for_each(|(lhs, rhs)| {
                use std::cmp::Ordering::*;
                let pos_lhs = [lhs.pos.x, lhs.pos.y, lhs.pos.z];
                let pos_rhs = [rhs.pos.x, rhs.pos.y, rhs.pos.z];
                next_state[i/3].vel += Vec3::from_vec(
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
                i += 1;
            });
        next_state.iter_mut().for_each(|moon| moon.pos += moon.vel);
        self.moons = next_state;
    }

    pub fn total_energy(&self) -> i128 {
        self.moons.iter()
            .map(|moon| moon.potential_energy() * moon.kinetic_energy())
            .fold1(|acc, val| acc + val)
            .unwrap()
    }
}

impl fmt::Display for Tracker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let moons: String = self.moons.iter()
            .map(|moon| format!("<Pos {}>  <Vel {}>", moon.pos, moon.vel))
            .intersperse("\n".to_owned())
            .collect();
        write!(f, "{}\n", moons)
    }
}

pub fn prepare_tracker(data: &'static str) -> Tracker {
    let positions = parse_input(data);
    let mut moons: Vec<_> = positions.iter()
        .map(|&pos| Moon { pos, ..Default::default() })
        .rev()
        .collect();
    let moons = [moons.pop().unwrap(), moons.pop().unwrap(), moons.pop().unwrap(), moons.pop().unwrap()];
    Tracker { moons }
}

fn main() {
    let mut tracker = prepare_tracker(INPUT);
    println!("{}", tracker);
    (0..1000).for_each(|_| tracker.step());
    println!("{}", tracker);
    println!("Total energy: {}", tracker.total_energy());

    let mut tracker = prepare_tracker(INPUT);
    let mut n = 0u128;
    loop {
        n += 1;
        tracker.step();
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
            tracker.moons[1],
            Moon {
                pos: Vec3 { x: 4, y: 10, z: 9 },
                vel: Vec3 { x: -3, y: 7, z: -2 },
            }
        );
        assert_eq!(
            tracker.moons[3],
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
