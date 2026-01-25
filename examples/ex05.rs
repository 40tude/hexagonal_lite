// Architectural template: one port, one adapter, one application service

mod domain {
    #[derive(Debug, Clone, PartialEq)]
    pub struct Stuff {
        pub value: u32,
    }

    #[derive(Debug)]
    pub enum StuffError {}
}

mod ports {
    use crate::domain::{Stuff, StuffError};

    pub trait StuffHandler {
        fn handle(&self, stuff: &Stuff) -> Result<(), StuffError>;
    }
}

mod adapters {
    use crate::domain::{Stuff, StuffError};
    use crate::ports::StuffHandler;

    pub struct MyAdapter;

    impl StuffHandler for MyAdapter {
        fn handle(&self, _stuff: &Stuff) -> Result<(), StuffError> {
            todo!("Adapter implementation goes here");
        }
    }
}

mod application {
    use crate::domain::{Stuff, StuffError};
    use crate::ports::StuffHandler;

    pub struct StuffService<'a, H: StuffHandler> {
        handler: &'a H,
    }

    impl<'a, H: StuffHandler> StuffService<'a, H> {
        pub fn new(handler: &'a H) -> Self {
            Self { handler }
        }

        pub fn process(&self, value: u32) -> Result<Stuff, StuffError> {
            let stuff = Stuff { value };
            self.handler.handle(&stuff)?;
            Ok(stuff)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::application::StuffService;
    use crate::domain::{Stuff, StuffError};
    use crate::ports::StuffHandler;

    struct TestHandler;

    impl StuffHandler for TestHandler {
        fn handle(&self, _stuff: &Stuff) -> Result<(), StuffError> {
            Ok(())
        }
    }

    #[test]
    fn process_stuff_successfully() {
        let service = StuffService::new(&TestHandler);

        let stuff = service.process(42).unwrap();

        assert_eq!(stuff.value, 42);
    }
}
