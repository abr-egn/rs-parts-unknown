#[macro_export]
macro_rules! some_or {
    ($opt:expr, $els:expr) => {
        { if let Some(v) = $opt { v } else { $els } }
    };
}

pub trait Closure<Args, Ret>: Fn(Args) -> Ret {
    fn clone_box(&self) -> Box<dyn Closure<Args, Ret>>;
}

impl<Args, Ret, T: 'static + Clone + Fn(Args) -> Ret> Closure<Args, Ret> for T {
    fn clone_box(&self) -> Box<dyn Closure<Args, Ret>> { Box::new(self.clone()) }
}

impl<Args: 'static, Ret: 'static> Clone for Box<dyn Closure<Args, Ret>> {
    fn clone(&self) -> Self { self.clone_box() }
}