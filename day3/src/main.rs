use lyon_geom::{euclid::Point2D, LineSegment, euclid::UnknownUnit};

const INPUT: &'static str = include_str!("../input");

type Point = Point2D<f64, UnknownUnit>;

fn load_input() -> Vec<Wire> {
    INPUT.trim().lines().map(|it| Wire::from_str(it)).collect()
}

struct Wire(Vec<Point>);

impl Wire {
    fn from_str(ords: &str) -> Self {
        let mut points: Vec<Point> = vec![Point { x: 0.0, y: 0.0, ..Default::default() }];
        ords.trim().split(',').enumerate().for_each(|(n, entry)| {
            let (mut x, mut y) = (0.0, 0.0);
            match &entry[..1] {
                "R" => x = (&entry[1..]).parse().unwrap(),
                "L" => x = -(&entry[1..]).parse::<f64>().unwrap(),
                "U" => y = (&entry[1..]).parse().unwrap(),
                "D" => y = -(&entry[1..]).parse::<f64>().unwrap(),
                _ => panic!("Error parsing input"),
            }
            points.push(Point::new(x + points[n].x, y + points[n].y));
        });

        Self(points)
    }
}

fn crosspoints(first: &Wire, second: &Wire) -> Vec<Point> {
    let mut crosses = vec![];
    for i in 1..first.0.len() {
        for j in 1..second.0.len() {
            let seg1 = LineSegment {
                from: first.0[i-1],
                to: first.0[i],
            };
            let seg2 = LineSegment {
                from: second.0[j-1],
                to: second.0[j],
            };
            if let Some(intersection) = seg1.intersection(&seg2) {
                crosses.push(intersection);
            } else {
                for point1 in vec![seg1.from, seg1.to] {
                    for point2 in vec![seg2.from, seg2.to] {
                        if point1 == point2 {
                            crosses.push(point1);
                        }
                    }
                }
            }
        }
    }
    crosses.sort_by(|p1, p2| (p1.x.abs() + p1.y.abs()).partial_cmp(&(p2.x.abs() + p2.y.abs())).unwrap());
    crosses
}

#[allow(unused)]
fn distance(p: &Point) -> i32 {
    (p.x.round().abs() + p.y.round().abs()) as i32
}

fn route_len(wire: &Wire, target: &Point) -> i32 {
    let mut route = 0;
    for i in 1..wire.0.len() {
        let seg = LineSegment {
            from: wire.0[i-1],
            to: wire.0[i],
        };
        if seg.from.x == seg.to.x && seg.to.x == target.x {
            if seg.from.y <= target.y && seg.to.y >= target.y {
                route += (target.y - seg.from.y).round() as i32;
                break;
            } else if seg.from.y >= target.y && seg.to.y <= target.y {
                route += (seg.from.y - target.y).round() as i32;
                break;
            }
        } else if seg.from.y == seg.to.y && seg.to.y == target.y {
            if seg.from.x <= target.x && seg.to.x >= target.x {
                route += (target.x - seg.from.x).round() as i32;
                break;
            } else if seg.from.x >= target.x && seg.to.x <= target.x {
                route += (seg.from.x - target.x).round() as i32;
                break;
            }
        }
        route += seg.length().round() as i32;
    }
    route
}

fn shortest_cross(wire1: &Wire, wire2: &Wire, points: &[Point]) -> i32 {
    let mut min = std::i32::MAX;
    for cross in points {
        let route = route_len(wire1, cross) + route_len(wire2, cross);
        if route < min {
            min = route;
        }
        println!("{:?}, route: {}", cross, route);
    }
    min
}
    

fn main() {
    let wires = load_input();
    let crosses = &crosspoints(&wires[0], &wires[1])[1..];
    println!("{}", shortest_cross(&wires[0], &wires[1], crosses));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example1() {
        let first = Wire::from_str("R75,D30,R83,U83,L12,D49,R71,U7,L72");
        let second = Wire::from_str("U62,R66,U55,R34,D71,R55,D58,R83");
        let crosses = crosspoints(&first, &second);
        let distance = distance(&crosses[1]);
        assert_eq!(distance, 159);
        assert_eq!(shortest_cross(&first, &second, &crosses[1..]), 610);
    }

    #[test]
    fn example2() {
        let first = Wire::from_str("R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51");
        let second = Wire::from_str("U98,R91,D20,R16,D67,R40,U7,R15,U6,R7");
        let crosses = crosspoints(&first, &second);
        let distance = distance(&crosses[1]);
        assert_eq!(distance, 135);
        assert_eq!(shortest_cross(&first, &second, &crosses[1..]), 410);
    }
}
