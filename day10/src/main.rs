use std::collections::HashSet;
use num::integer::gcd;
use itertools::{Itertools, MinMaxResult};

const INPUT: &'static str = include_str!("../input");

fn parse_map(input: &'static str) -> Vec<(i64, i64)> {
    input.lines().enumerate()
        .flat_map(|(y, l)| {
            l.trim().chars()
                .enumerate()
                .filter(|&(_, ch)| ch == '#')
                .map(move |(x, _)| (x as i64, y as i64))
        })
        .collect()
}

struct Bounds {
    min_x: i64,
    max_x: i64,
    min_y: i64,
    max_y: i64,
}

impl Bounds {
    fn new(vals: &Vec<&(i64, i64)>) -> Self
    {
        use MinMaxResult::MinMax;
        let (min_x, max_x) = if let MinMax(min, max) = vals.iter().minmax_by_key(|(x, _)| *x) {
            (min.0, max.0)
        } else {
            panic!("Collection should have at least one element");
        };
        let (min_y, max_y) = if let MinMax(min, max) = vals.iter().minmax_by_key(|(_, y)| *y) {
            (min.1, max.1)
        } else {
            panic!("Collection should have at least one element");
        };
        Self { min_x, max_x, min_y, max_y }
    }

    fn limits(&self, x: i64, y:i64) -> bool {
        x >= self.min_x &&
        x <= self.max_x &&
        y >= self.min_y &&
        y <= self.max_y
    }
}

fn direction(origin: (i64, i64), target: (i64, i64)) -> Option<(i64, i64)> {
    let (x, y) = (target.0-origin.0, target.1-origin.1);
    if x != 0 && y != 0 {
        let divisor = gcd(x, y);
        Some((x/divisor, y/divisor))
    } else if x != 0 {
        if x > 0 { Some((1, 0)) } else { Some((-1, 0)) }
    } else if y != 0 {
        if y > 0 { Some((0, 1)) } else { Some((0, -1)) }
    } else {
        None
    }
}

fn laser_angle_rad(offset: (i64, i64)) -> f64 {
    use std::f64::consts::{PI, FRAC_PI_2};
    use std::cmp::Ordering::*;
    let (x, y) = offset;
    if y != 0 {
        let angle = (x as f64 / y as f64).atan();
        if y > 0 {
            PI - angle
        } else {
            match x.cmp(&0) {
                Less    => 2.0*PI - angle,
                Equal   => 0.0,
                Greater => -angle,
            }
        }
    } else if x > 0 {
        FRAC_PI_2
    } else if x < 0 {
        PI + FRAC_PI_2
    } else {
        0.0
    }
}

fn sort_by_desintegration<'a, I>(origin: (i64, i64), iter: I) -> impl Iterator<Item=(i64, i64)>
where
    I: Iterator<Item=&'a (i64, i64)>,
{
    let mut to_be_sorted: Vec<_> = iter
        .filter(|&&item| item != origin)
        .map(|&item| {
            let offset = direction(origin, item).unwrap();
            let angle = laser_angle_rad(offset);
            (angle, item)
        })
        .collect();
    to_be_sorted.sort_by(|ref lhs, ref rhs| lhs.0.partial_cmp(&rhs.0).unwrap());
    to_be_sorted
        .into_iter()
        .map(|(_, offset)| offset)
}

fn get_visible<'a, I>(origin: (i64, i64), asteroids: I) -> HashSet<&'a (i64, i64)>
where
    I: Iterator<Item=&'a (i64, i64)> + Clone
{
    let mut test_set: HashSet<_> = asteroids.clone().collect();
    let bounds = Bounds::new(&asteroids.clone().collect());
    test_set.remove(&origin);
    asteroids
        .filter(|&&pos| pos != origin)
        .for_each(|&target| {
            let (step_x, step_y) = direction(origin, target).unwrap();
            let (mut x, mut y) = target;
            while bounds.limits(x, y) {
                x += step_x;
                y += step_y;
                if test_set.contains(&(x, y)) {
                    test_set.remove(&(x, y));
                }
            }
        });
    test_set
}

fn peek_nth_destroyed(origin: (i64, i64), n: usize, asteroids: &Vec<(i64, i64)>) -> (i64, i64) {
    let mut n = n-1;
    let mut future_set: HashSet<_> = asteroids.iter().collect();
    let mut visible = get_visible(origin, asteroids.iter());
    while visible.len() < n {
        n -= visible.len();
        future_set = future_set
            .into_iter()
            .collect::<HashSet<_>>()
            .difference(&visible)
            .cloned()
            .collect();
        visible = get_visible(origin, future_set.iter().cloned());
        println!("Now there are {} asteroids visible and {} more to utilize", visible.len(), n);
    }
    sort_by_desintegration(origin, visible.into_iter())
        .nth(n)
        .unwrap()
}

fn best_station(asteroids: &Vec<(i64, i64)>) -> (u64, (i64, i64)) {
    asteroids.iter()
        .map(|&current| (get_visible(current, asteroids.iter()).len() as u64, current))
        .max_by_key(|&(count, _)| count)
        .expect("Failed to find best station, are there any asteroids?")
}

fn main() {
    let asteroids = parse_map(INPUT);
    let best = best_station(&asteroids);
    println!("Best station is: {:?}, it sees {} asteroids", best.1, best.0);
    let nth_destroyed = peek_nth_destroyed(best.1, 200, &asteroids);
    println!("Asteroid destroyed as 200th is: {:?}", nth_destroyed);
}

#[cfg(test)]
mod tests {
    use super::*;
    const TINY_SPACE: &'static str = ".#..#
                                      .....
                                      #####
                                      ....#
                                      ...##";

    const HUGE_SPACE: &'static str = ".#..##.###...#######
                                      ##.############..##.
                                      .#.######.########.#
                                      .###.#######.####.#.
                                      #####.##.#.##.###.##
                                      ..#####..#.#########
                                      ####################
                                      #.####....###.#.#.##
                                      ##.#################
                                      #####.##.###..####..
                                      ..######..##.#######
                                      ####.##.####...##..#
                                      .#####..#.######.###
                                      ##...#.##########...
                                      #.##########.#######
                                      .####.#.###.###.#.##
                                      ....##.##.###..#####
                                      .#.#.###########.###
                                      #.#.#.#####.####.###
                                      ###.##.####.##.#..##";

    #[test]
    fn get_direction() {
        assert_eq!(direction((0,0), (6,3)),    Some((2, 1)));
        assert_eq!(direction((1,4), (13, 12)), Some((3, 2)));
        assert_eq!(direction((7,0), (0,0)),    Some((-1, 0)));
        assert_eq!(direction((0,8), (0,3)),    Some((0, -1)));
        assert_eq!(direction((0,0), (0,3)),    Some((0, 1)));
    }

    #[test]
    fn map_parsing() {
        assert_eq!(
            parse_map(TINY_SPACE),
            [(1, 0), (4, 0), (0, 2), (1, 2), (2, 2),
             (3, 2), (4, 2), (4, 3), (3, 4), (4, 4)]
        );
    }

    #[test]
    fn find_best_station() {
        let map = parse_map(HUGE_SPACE);
        let best = best_station(&map);
        assert_eq!(best.0, 210);
        assert_eq!(best.1, (11, 13));
    }

    #[test]
    fn peeking_nth_destroyed() {
        let map = parse_map(HUGE_SPACE);
        let target = peek_nth_destroyed((11, 13), 200, &map);
        assert_eq!(target, (8, 2));
    }

    #[test]
    fn getting_visible() {
        let map = parse_map(TINY_SPACE);
        let origin = (1, 0);
        let visible = get_visible(origin, map.iter());
        assert_eq!(
            visible,
            [(4, 0), (4, 2), (3, 2), (4, 4), (2, 2), (1, 2), (0, 2)]
                .iter().collect()
        );
    }

    #[test]
    fn sorting_by_desintegration() {
        let map = parse_map(TINY_SPACE);
        let origin = (1, 0);
        let visible = get_visible(origin, map.iter());
        assert_eq!(
            sort_by_desintegration(origin, visible.into_iter()).collect::<Vec<_>>(),
            [(4, 0), (4, 2), (3, 2), (4, 4), (2, 2), (1, 2), (0, 2)]
        );
    }
}
