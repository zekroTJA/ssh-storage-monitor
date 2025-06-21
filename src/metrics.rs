use std::fmt::{self};

// pub struct Metric<'a, V> {
//     name: &'a str,
//     value: V,
//     labels: Vec<(String, String)>,
// }

// impl<'a, V> Metric<'a, V>
// where
//     V: fmt::Display,
// {
//     pub fn new(name: &'a str, value: V) -> Self {
//         Self { name, value }
//     }

//     pub fn write<W: fmt::Write>(&self, mut w: W) -> fmt::Result {
//         // write name
//         w.write_str(self.name)?;

//         // write labels
//         w.write_char('{')?;
//         let mut labels = labels.into_iter();
//         if let Some((k, v)) = labels.next() {
//             write!(w, r#"{k}="{v}""#)?;
//         }
//         for (k, v) in labels {
//             write!(w, r#",{k}="{v}""#)?;
//         }
//         w.write_char('}')?;

//         // write value
//         write!(w, " {value}")
//     }
// }

pub fn write_metric<'a, W, L, V>(mut w: W, name: &str, value: V, labels: L) -> fmt::Result
where
    W: fmt::Write,
    L: IntoIterator<Item = &'a (&'a str, &'a str)>,
    V: fmt::Display,
{
    // write name
    w.write_str(name)?;

    // write labels
    w.write_char('{')?;
    let mut labels = labels.into_iter();
    if let Some((k, v)) = labels.next() {
        write!(w, r#"{k}="{v}""#)?;
    }
    for (k, v) in labels {
        write!(w, r#",{k}="{v}""#)?;
    }
    w.write_char('}')?;

    // write value
    writeln!(w, " {value}")
}

#[cfg(test)]
mod test {
    #[test]
    fn write_metrics() {
        let mut res = String::new();
        super::write_metric(
            &mut res,
            "foo",
            123,
            [&("key1", "value1"), &("key2", "value2")],
        )
        .unwrap();
        assert_eq!("foo{key1=\"value1\",key2=\"value2\"} 123\n", res);
    }
}
