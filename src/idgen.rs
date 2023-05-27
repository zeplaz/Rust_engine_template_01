
pub type EntityId = u64;

#[derive(Debug)]
struct IdGenerator {
    next_id: u64,
}

impl IdGenerator {
    fn new() -> IdGenerator {
        IdGenerator { next_id: 0 }
    }

    fn generate_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}
