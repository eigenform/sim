

pub trait Clocked {
    fn sim_clock_edge(&mut self);
}
pub trait Combinational {
    fn sim_comb(&mut self);
}


#[derive(Clone, Copy, Debug)]
pub struct Signal<T: Sized + Copy>(Option<T>);
impl <T: Sized + Copy> Default for Signal<T> {
    fn default() -> Self { Self(None) }
}
impl <T: Sized + Copy> Signal<T> {
    pub fn new() -> Self { Self(None) }
    pub fn has_value(&self) -> bool { self.0.is_some() }
    pub fn sample(&self) -> T { self.0.unwrap() }
    pub fn drive(&mut self, value: T) {
        self.0 = Some(value);
    }
}

//#[derive(Clone, Copy, Debug)]
//pub struct VecSignal<T: Sized + Copy, const SIZE: usize>([Signal<T>; SIZE]);
//impl <T: Sized + Copy, const SIZE: usize> Default for VecSignal<T, SIZE> {
//    fn default() -> Self { Self([Signal::default(); SIZE]) }
//}
//impl <T: Sized + Copy, const SIZE: usize> 
//std::ops::Index<usize> for VecSignal<T, SIZE> {
//    type Output = Signal<T>;
//    fn index(&self, idx: usize) -> &Self::Output {
//        &self.0.index(idx)
//    }
//}
//impl <T: Sized + Copy, const SIZE: usize> 
//std::ops::IndexMut<usize> for VecSignal<T, SIZE> {
//    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
//        self.0.index_mut(idx)
//    }
//}
//impl <T: Sized + Copy, const SIZE: usize> VecSignal<T, SIZE> {
//    pub fn sample(&self) -> [Signal<T>; SIZE] { self.0 }
//    pub fn drive(&mut self, value: [Signal<T>; SIZE]) {
//        self.0 = value;
//    }
//}





#[derive(Debug)]
pub struct Register<T: Sized + Copy> {
    input: Signal<T>,
    data: T
}
impl <T: Sized + Copy> Register<T> {
    pub fn sample(&self) -> T { self.data }
    pub fn drive(&mut self, value: T) {
        self.input.drive(value);
    }
    pub fn init(data: T) -> Self { 
        Self { 
            input: Signal::default(),
            data: data,
        }
    }
    pub fn clock_tick(&mut self) {
        self.data = self.input.sample();
    }
}
impl <T: Sized + Copy> Combinational for Register<T> {
    fn sim_comb(&mut self) {
    }
}

impl <T: Sized + Copy> Clocked for Register<T> {
    fn sim_clock_edge(&mut self) {
        self.data = self.input.sample();
    }
}


#[cfg(test)]
mod test {
    use super::*;
}



