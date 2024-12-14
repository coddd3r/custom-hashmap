extern crate my_hashmap;
use my_hashmap::HashMap;
#[derive(Hash, Eq, PartialEq, Debug)]
struct Viking {
    name: String,
    country: String,
}

impl Viking {
    /// Creates a new Viking.
    fn new(name: &str, country: &str) -> Viking {
        Viking {
            name: name.to_string(),
            country: country.to_string(),
        }
    }
}

fn main() {
    // Use a HashMap to store the vikings' health points.
    let vikings = HashMap::from([
        (Viking::new("Einar", "Norway"), 25),
        (Viking::new("Olaf", "Denmark"), 24),
        (Viking::new("Harald", "Iceland"), 12),
    ]);
    // Use derived implementation to print the status of the vikings.
    for (viking, health) in &vikings {
        println!("{viking:?} has {health} hp");
    }
    let solar_distance = HashMap::from([
        ("Mercury", 0.4),
        ("Venus", 0.7),
        ("Earth", 1.0),
        ("Mars", 1.5),
    ]);
    for (p, dist) in &solar_distance {
        println!("{p:?} has {dist} dist");
    }
}
