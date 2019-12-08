#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;
use itertools::Itertools;

const INPUT: &'static str = include_str!("../input");

#[cfg(not(test))]
lazy_static! {
    static ref DIMS: HashMap<&'static str, i32> = {
        let mut map = HashMap::new();
        map.insert("width", 25);
        map.insert("height", 6);
        map.insert("layers", INPUT.len() as i32 / (map["width"]*map["height"]));
        map
    };
}

#[cfg(test)]
lazy_static! {
    static ref DIMS: HashMap<&'static str, i32> = {
        let mut map = HashMap::new();
        map.insert("width", 4);
        map.insert("height", 2);
        map.insert("layers", 2);
        map
    };
}

/// x is a column number
/// y is a row number
/// z is a layer number
fn index(x: i32, y: i32, z: i32) -> usize {
    assert!(x < DIMS["width"] && y < DIMS["height"]);
    (z * DIMS["width"] * DIMS["height"] + y * DIMS["width"] + x) as usize
}

fn count_digits_on_layer(image: &Vec<i32>, layer: i32, target: i32) -> i32 {
    image.chunks((DIMS["width"]*DIMS["height"]) as usize)
        .nth(layer as usize)
        .map(|lay| lay.into_iter().filter(|d| *d == &target).count())
        .unwrap() as i32
}

fn find_layer_with_fewest(image: &Vec<i32>, target: i32) -> i32 {
    (0..DIMS["layers"]).map(|i| (i, count_digits_on_layer(image, i, target)))
        .min_by_key(|(_, ref count)| *count)
        .unwrap().0
}
    
fn load_image(content: &str) -> Vec<i32> {
    content.chars()
        .map(|c| c.to_digit(10))
        .filter_map(|d| d.map(|inner_d| inner_d as i32))
        .collect()
}

fn decode(image: Vec<i32>) -> Vec<i32> {
    (0..DIMS["height"]).cartesian_product(0..DIMS["width"])
        .map(|(y, x)| {
            (0..DIMS["layers"]).filter(|layer| image[index(x, y, *layer)] != 2)
                .map(|layer| image[index(x, y, layer)])
                .next().unwrap_or(2)
        })
        .collect()
}

fn main() {
    let image = load_image(INPUT);
    let layer_with_fewest_0 = find_layer_with_fewest(&image, 0);
    let ones = count_digits_on_layer(&image, layer_with_fewest_0, 1);
    let twos = count_digits_on_layer(&image, layer_with_fewest_0, 2);
    dbg!(ones*twos);
    let decoded = decode(image);
    for x in 0..DIMS["width"] {
        for y in 0..DIMS["height"] {
            print!("{}", if decoded[index(x, y, 0)] == 0 { ' ' } else { '#' });
        }
        print!("\n");
    }
}

/// width: 4, height: 2, layers: 2
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_loading() {
        let img = "1234112312345678";
        assert_eq!(load_image(img), [1,2,3,4,1,1,2,3,1,2,3,4,5,6,7,8]);
    }

    #[test]
    fn counting_digits() {
        let img = load_image("1234112312345678");
        assert_eq!(count_digits_on_layer(&img, 0, 1), 3);
        assert_eq!(count_digits_on_layer(&img, 0, 2), 2);
        assert_eq!(count_digits_on_layer(&img, 1, 1), 1);
    }

    #[test]
    fn indexing() {
        let img = load_image("1234112312345678");
        assert_eq!(img[index(3, 1, 1)], 8);
        assert_eq!(img[index(1, 0, 0)], 2);
        assert_eq!(img[index(0, 1, 1)], 5);
    }

    #[test]
    fn decoding() {
        let img = load_image("0122102212112201");
        assert_eq!(decode(img), vec![0,1,1,1,1,0,0,1]);
    }
}
