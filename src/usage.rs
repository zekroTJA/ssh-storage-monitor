use std::str::FromStr;

#[derive(Debug)]
pub struct DiskUsage {
    pub entries: Vec<DiskUsageEntry>,
}

impl FromStr for DiskUsage {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut lines = s.lines();

        // skip header line
        lines.next();

        let entries = lines.map(|l| l.parse()).collect::<Result<Vec<_>, _>>()?;

        Ok(Self { entries })
    }
}

#[derive(Debug)]
pub struct DiskUsageEntry {
    pub filesystem: String,
    pub mount: String,
    pub blocks: usize,
    pub used: usize,
    pub available: usize,
}

impl FromStr for DiskUsageEntry {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut split = s.split_whitespace();

        let filesystem = split
            .next()
            .ok_or_else(|| anyhow::anyhow!("no filesystem entry"))?
            .to_string();

        let blocks = split
            .next()
            .ok_or_else(|| anyhow::anyhow!("no blocks entry"))?
            .parse()?;

        let used = split
            .next()
            .ok_or_else(|| anyhow::anyhow!("no used entry"))?
            .parse()?;

        let available = split
            .next()
            .ok_or_else(|| anyhow::anyhow!("no available entry"))?
            .parse()?;

        // skip usage percentage value
        split.next();

        let mount = split.collect::<Vec<_>>().join(" ");
        if mount.is_empty() {
            return Err(anyhow::anyhow!("no mount entry"));
        }

        Ok(Self {
            filesystem,
            blocks,
            used,
            available,
            mount,
        })
    }
}
