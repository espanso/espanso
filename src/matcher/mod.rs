use std::sync::mpsc::Receiver;

pub struct Match {
    pub trigger: String,
    pub result: String
}



pub struct Matcher {
    pub receiver: Receiver<char>
}

impl Matcher {
    pub fn watch(&self) {
        loop {
            match self.receiver.recv() {
                Ok(c) => {
                    println!("Yeah {}",c);
                },
                Err(_) => panic!("Worker threads disconnected before the solution was found!"),
            }
        }
    }
}