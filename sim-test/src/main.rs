
use sim_macros::*;
use sim::*;
use sim;

pub trait Addable: Sized + std::ops::Add<Output=Self> + Copy {
}
impl Addable for usize {}

#[derive(Module)]
#[derive(Debug)]
pub struct MyAdder {
    #[input]
    x: Signal<usize>,
    #[input]
    y: Signal<usize>,
    #[output]
    output: Signal<usize>,
}
impl Combinational for MyAdder {
    fn sim_comb(&mut self) {
        let x = self.x.sample();
        let y = self.y.sample();
        let z = x + y;
        self.output.drive(z);
    }
}

#[derive(Module)]
#[derive(Debug)]
pub struct MyAdderReg {
    #[input]
    x: Signal<usize>,
    #[input]
    y: Signal<usize>,
    #[clocked]
    s: Register<usize>,
    #[output]
    output: Signal<usize>,
}
impl Combinational for MyAdderReg {
    fn sim_comb(&mut self) {
        let x = self.x.sample();
        let y = self.y.sample();
        let z = x + y;
        self.s.drive(z);

        let out = self.s.sample();
        self.output.drive(out);
    }
}



#[derive(Module)]
#[derive(Debug)]
pub struct MyParametrizedAdder<T: Addable> {
    #[input]
    x: Signal<T>,
    #[input]
    y: Signal<T>,
    #[output]
    output: Signal<T>,
}
impl <T: Addable> MyParametrizedAdder<T> {
    pub fn run(&mut self) {
        let x = self.x.sample();
        let y = self.y.sample();
        let z = x + y;
        self.output.drive(z);
    }
}




fn main() {
    let mut a = MyAdder {
        x: Signal::default(),
        y: Signal::default(),
        output: Signal::default(),
    };
    for i in 0..4 {
        a.drive_x(1);
        a.drive_y(i);
        a.sim_comb();
        println!("{} {:?}", a.sample_output(), a);
    }

    let mut a = MyAdderReg {
        x: Signal::default(),
        y: Signal::default(),
        output: Signal::default(),
        s: Register::init(0),
    };
    for i in 0..4 {
        a.drive_x(1);
        a.drive_y(i);
        a.sim_comb();
        a.sim_clock_edge();
        println!("{} {:?}", a.sample_output(), a);
    }



}




