use std::time::Duration;

pub trait DurationExtension {
    fn one_second() -> Duration {
        Duration::from_secs(1)
    }
    fn hhmmss(&self) -> String;
}

fn pad_zero(n: u64) -> String {
    if n < 10 {
        format!("0{}", n)
    } else {
        n.to_string()
    }
}

fn pad_millis(n: u64) -> String {
    if n < 10 {
        format!("00{}", n)
    } else if n < 100 {
        format!("0{}", n)
    } else {
        n.to_string()
    }
}

impl DurationExtension for Duration {
    fn hhmmss(&self) -> String {
        let d = self.as_secs() / 86400;
        let h = (self.as_secs() % 86400) / 3600;
        let m = (self.as_secs() % 3600) / 60;
        let s = self.as_secs() % 60;
        let ml = (self.as_millis() % 1000) as u64;
        let d_str = if d == 0 {
            String::from("")
        } else {
            format!("+{}.", d)
        };
        format!(
            "{}{}:{}:{}.{}",
            d_str,
            h,
            pad_zero(m),
            pad_zero(s),
            pad_millis(ml)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_duration_hhmmss() {
        // some simple times within a day
        assert_eq!(Duration::from_secs(28800).hhmmss(), "8:00:00.000");
        assert_eq!(Duration::from_secs(28800 + 543).hhmmss(), "8:09:03.000");
        // a test output in milliseconds taking longer than a day
        assert_eq!(Duration::from_millis(108208019).hhmmss(), "+1.6:03:28.019");
    }
}
