#[derive(Debug, Clone)]
pub enum Mod {
    Add(i32),
    Mul(f32),
}

impl Mod {
    fn index(&self) -> u32 {
        match self {
            Mod::Add(_) => 0,
            Mod::Mul(_) => 1,
        }
    }
    fn apply(&self, val: i32) -> i32 {
        match self {
            Mod::Add(n) => val + *n,
            Mod::Mul(n) => ((val as f32) * *n) as i32,
        }
    }
}

pub fn eval<'a, I: IntoIterator<Item=&'a Mod>>(base: i32, mods: I) -> i32 {
    let mut sorted: Vec<_> = mods.into_iter().collect();
    sorted.sort_by(|a, b| { a.index().cmp(&b.index()) });
    let mut out = base;
    for m in sorted {
        out = m.apply(out);
    }
    out
}