#![allow(dead_code, unused_variables)]

use regex::Regex;
use itertools::Itertools;
use std::fmt;

pub const INPUT: &'static str = include_str!("../input");

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
struct Vec3 {
    x: i64,
    y: i64,
    z: i64,
}

impl Vec3 {
    const fn zero() -> Self {
        Self { x: 0, y: 0, z: 0 }
    }

    fn from_vec(input: Vec<i64>) -> Self {
        assert_eq!(input.len(), 3);
        Vec3 {
            x: input[0],
            y: input[1],
            z: input[2],
        }
    }

    fn sum_abs(&self) -> i64 {
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
                num.as_str().parse::<i64>().unwrap()
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
    fn potential_energy(&self) -> i64 {
        self.pos.sum_abs()
    }
    
    fn kinetic_energy(&self) -> i64 {
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
    pub fn step_x(&mut self) {
        use std::cmp::Ordering::*;
        let mut next_x = [
            unsafe { self.moons.get_unchecked(0).vel.x },
            unsafe { self.moons.get_unchecked(1).vel.x },
            unsafe { self.moons.get_unchecked(2).vel.x },
            unsafe { self.moons.get_unchecked(3).vel.x },
        ];
        for i in 0..4 {
            for j in 0..4 {
                if i == j { continue; }
                unsafe {
                    *next_x.get_unchecked_mut(i) += match 
                        self.moons.get_unchecked(i).pos.x.cmp(&self.moons.get_unchecked(j).pos.x)
                    {
                        Less => 1,
                        Equal => 0,
                        Greater => -1,
                    };
                }
            }
        }

        (0..4).for_each(|n| {
            unsafe {
                let mut current = self.moons.get_unchecked_mut(n);
                let x = next_x.get_unchecked(n);
                current.vel.x = *x;
                current.pos.x += *x;
            }
        });
    }

    pub fn step_y(&mut self) {
        use std::cmp::Ordering::*;
        let mut next_y = [
            unsafe { self.moons.get_unchecked(0).vel.y },
            unsafe { self.moons.get_unchecked(1).vel.y },
            unsafe { self.moons.get_unchecked(2).vel.y },
            unsafe { self.moons.get_unchecked(3).vel.y },
        ];
        for i in 0..4 {
            for j in 0..4 {
                if i == j { continue; }
                unsafe {
                    *next_y.get_unchecked_mut(i) += match 
                        self.moons.get_unchecked(i).pos.y.cmp(&self.moons.get_unchecked(j).pos.y)
                    {
                        Less => 1,
                        Equal => 0,
                        Greater => -1,
                    };
                }
            }
        }

        (0..4).for_each(|n| {
            unsafe {
                let mut current = self.moons.get_unchecked_mut(n);
                let y = next_y.get_unchecked(n);
                current.vel.y = *y;
                current.pos.y += *y;
            }
        });
    }

    pub fn step_z(&mut self) {
        use std::cmp::Ordering::*;
        let mut next_z = [
            unsafe { self.moons.get_unchecked(0).vel.z },
            unsafe { self.moons.get_unchecked(1).vel.z },
            unsafe { self.moons.get_unchecked(2).vel.z },
            unsafe { self.moons.get_unchecked(3).vel.z },
        ];
        for i in 0..4 {
            for j in 0..4 {
                if i == j { continue; }
                unsafe {
                    *next_z.get_unchecked_mut(i) += match 
                        self.moons.get_unchecked(i).pos.z.cmp(&self.moons.get_unchecked(j).pos.z)
                    {
                        Less => 1,
                        Equal => 0,
                        Greater => -1,
                    };
                }
            }
        }

        (0..4).for_each(|n| {
            unsafe {
                let mut current = self.moons.get_unchecked_mut(n);
                let z = next_z.get_unchecked(n);
                current.vel.z = *z;
                current.pos.z += *z;
            }
        });
    }

    pub fn step(&mut self) {
        use std::cmp::Ordering::*;
        let mut next_vels = [
            unsafe { self.moons.get_unchecked(0).vel },
            unsafe { self.moons.get_unchecked(1).vel },
            unsafe { self.moons.get_unchecked(2).vel },
            unsafe { self.moons.get_unchecked(3).vel },
        ];
        for i in 0..4 {
            for j in 0..4 {
                if i == j { continue; }
                unsafe {
                    next_vels.get_unchecked_mut(i).x += match 
                        self.moons.get_unchecked(i).pos.x.cmp(&self.moons.get_unchecked(j).pos.x)
                    {
                        Less => 1,
                        Equal => 0,
                        Greater => -1,
                    };
                    next_vels.get_unchecked_mut(i).y += match 
                        self.moons.get_unchecked(i).pos.y.cmp(&self.moons.get_unchecked(j).pos.y)
                    {
                        Less => 1,
                        Equal => 0,
                        Greater => -1,
                    };
                    next_vels.get_unchecked_mut(i).z += match 
                        self.moons.get_unchecked(i).pos.z.cmp(&self.moons.get_unchecked(j).pos.z)
                    {
                        Less => 1,
                        Equal => 0,
                        Greater => -1,
                    };
                }
            }
        }

        (0..4).for_each(|n| {
            unsafe {
                let mut current = self.moons.get_unchecked_mut(n);
                let curr_vel = next_vels.get_unchecked(n);
                current.vel = *curr_vel;
                current.pos += *curr_vel;
            }
        });
    }

    pub fn total_energy(&self) -> i64 {
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

pub fn is_initial_state(tracker: &Tracker) -> bool {
    const INITIAL_STATE: [Moon; 4] = [
        Moon {
            pos: Vec3 { x: -9, y: -1, z: -1 },
            vel: Vec3::zero(),
        },
        Moon {
            pos: Vec3 { x: 2, y: 9, z: 5 },
            vel: Vec3::zero(),
        },
        Moon {
            pos: Vec3 { x: 10, y: 18, z: -12 },
            vel: Vec3::zero(),
        },
        Moon {
            pos: Vec3 { x: -6, y: 15, z: -7 },
            vel: Vec3::zero(),
        },
    ];
    tracker.moons == INITIAL_STATE
}

pub fn is_initial_state_x(tracker: &Tracker) -> bool {
    const INITIAL_STATE_X: [(i64, i64); 4] = [(-9, 0), (2,  0), (10, 0), (-6, 0)];
    tracker.moons.iter().map(|moon| (moon.pos.x, moon.vel.x)).collect::<Vec<_>>() == INITIAL_STATE_X
}

pub fn is_initial_state_y(tracker: &Tracker) -> bool {
    const INITIAL_STATE_Y: [(i64, i64); 4] = [(-1, 0), (9,  0), (18, 0), (15, 0)];
    tracker.moons.iter().map(|moon| (moon.pos.y, moon.vel.y)).collect::<Vec<_>>() == INITIAL_STATE_Y
}

pub fn is_initial_state_z(tracker: &Tracker) -> bool {
    const INITIAL_STATE_Z: [(i64, i64); 4] = [(-1, 0), (5,  0), (-12, 0), (-7, 0)];
    tracker.moons.iter().map(|moon| (moon.pos.z, moon.vel.z)).collect::<Vec<_>>() == INITIAL_STATE_Z
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
