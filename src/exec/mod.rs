mod driver;

pub struct ExecutionConfig {

}

impl ExecutionConfig {
    pub fn new() -> ExecutionConfig {
        ExecutionConfig {

        }
    }

    pub fn run(&self,args: Vec<String>) {
        let _exec = driver::run_executor(args);
    }
}
