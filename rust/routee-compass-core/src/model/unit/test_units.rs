use uom::si::f64::{Length, Time};
use uom::si::velocity::{kilometer_per_hour, mile_per_hour};
use uom::si::{length::meter, time::second};

#[derive(Debug, Clone, Copy)]
pub enum StateFeature {
    Distance(Length),
    Time(Time),
}

impl StateFeature {
    pub fn value(&self) -> f64 {
        match self {
            StateFeature::Distance(q) => q.value, // meters
            StateFeature::Time(q) => q.value,     // seconds
        }
    }

    pub fn unit_name(&self) -> &'static str {
        match self {
            StateFeature::Distance(_) => "m",
            StateFeature::Time(_) => "s",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub name: String,
    pub qty: StateFeature,
}

#[derive(Debug, Default)]
pub struct Quantities {
    entries: Vec<Entry>,
}

impl Quantities {
    pub fn new() -> Self {
        Quantities {
            entries: Vec::new(),
        }
    }

    pub fn push_distance(&mut self, name: impl Into<String>, q: Length) {
        self.entries.push(Entry {
            name: name.into(),
            qty: StateFeature::Distance(q),
        });
    }

    pub fn push_time(&mut self, name: impl Into<String>, q: Time) {
        self.entries.push(Entry {
            name: name.into(),
            qty: StateFeature::Time(q),
        });
    }

    pub fn get_by_name(&self, name: &str) -> Option<StateFeature> {
        self.entries.iter().find(|e| e.name == name).map(|e| e.qty)
    }

    pub fn get_distance(&self, name: &str) -> Option<Length> {
        self.get_by_name(name).and_then(|q| match q {
            StateFeature::Distance(d) => Some(d),
            _ => None,
        })
    }

    pub fn get_time(&self, name: &str) -> Option<Time> {
        self.get_by_name(name).and_then(|q| match q {
            StateFeature::Time(t) => Some(t),
            _ => None,
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &StateFeature)> {
        self.entries.iter().map(|e| (&e.name, &e.qty))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use uom::si::f64::{Length, Time};
    use uom::si::velocity::{kilometer_per_hour, mile_per_hour};
    use uom::si::{length::meter, time::second};

    #[test]
    fn test() {
        let mut qs = Quantities::new();
        qs.push_distance("distance", Length::new::<meter>(2.0));
        qs.push_time("time", Time::new::<second>(1.5));

        for (name, any) in qs.iter() {
            println!("{} = {} {}", name, any.value(), any.unit_name());
        }

        let distance = qs.get_distance("distance").unwrap();
        let time = qs.get_time("time").unwrap();

        let speed = distance / time;
        let speed_mph = speed.get::<mile_per_hour>();
        let speed_kph = speed.get::<kilometer_per_hour>();

        println!("Speed: {:.2} mph, {:.2} kph", speed_mph, speed_kph);
    }
}
