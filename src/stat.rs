#[derive(Debug, Clone)]
pub enum StatMod {
    Add(i32),
    Mul(f32),
}

impl StatMod {
    pub fn eval<'a, I: IntoIterator<Item=&'a StatMod>>(base: i32, mods: I) -> i32 {
        let mut sorted: Vec<_> = mods.into_iter().collect();
        sorted.sort_by(|a, b| { a.index().cmp(&b.index()) });
        let mut out = base;
        for m in sorted {
            out = m.apply(out);
        }
        out
    }

    fn index(&self) -> u32 {
        match self {
            StatMod::Add(_) => 0,
            StatMod::Mul(_) => 1,
        }
    }
    fn apply(&self, val: i32) -> i32 {
        match self {
            StatMod::Add(n) => val + *n,
            StatMod::Mul(n) => ((val as f32) * *n) as i32,
        }
    }
}