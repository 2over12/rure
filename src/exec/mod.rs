mod driver;


pub struct Executor {

}

impl Executor {
    pub fn new() -> Executor {
        Executor {}
    }
}

pub struct ExecutionConfig {

}

impl ExecutionConfig {
    pub fn new() -> ExecutionConfig {
        ExecutionConfig {

        }
    }

    pub fn run(&self,args: Vec<String>) {
        let exec = driver::run_executor(args);
    }
}
