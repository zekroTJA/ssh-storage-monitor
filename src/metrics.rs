use std::fmt::{self};

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
        super::write_metric(&mut res, "foo", 123, [
            &("key1", "value1"),
            &("key2", "value2"),
        ])
        .unwrap();
        assert_eq!("foo{key1=\"value1\",key2=\"value2\"} 123\n", res);
    }
}
