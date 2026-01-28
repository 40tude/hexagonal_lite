// cargo run --example ex03bis
// ex03bis.rs - ex03 rewritten with &dyn parameters (no lifetime needed)
// - OrderService no longer has a generic N or lifetime 'a
// - The notifier is passed as a parameter to process_order(&mut self, total: u32, notifier: &dyn OrderNotifier) instead of being stored in the struct.
// - The service keeps its state (next_id) but no longer depends on a specific adapter for construction.
// - main() passes &notifier to each call instead of giving it to the constructor.

mod domain {
    use std::fmt;

    #[derive(Debug, Clone)]
    pub struct Order {
        pub id: u32,
        pub total: u32,
    }

    #[derive(Debug)]
    pub enum OrderError {
        // Failed,
    }

    impl fmt::Display for OrderError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{self:?}")
        }
    }
}

mod ports {
    use crate::domain::{Order, OrderError};

    pub trait OrderNotifier {
        fn process(&self, order: &Order) -> Result<(), OrderError>;
    }
}

mod adapters {
    use crate::domain::{Order, OrderError};
    use crate::ports::OrderNotifier;

    pub struct ConsoleNotifier;

    impl OrderNotifier for ConsoleNotifier {
        fn process(&self, order: &Order) -> Result<(), OrderError> {
            println!(
                "[Console] Order #{} confirmed! Total: {}",
                order.id, order.total
            );
            Ok(())
        }
    }
}

mod application {
    use crate::domain::{Order, OrderError};
    use crate::ports::OrderNotifier;

    pub struct OrderService {
        next_id: u32,
    }

    impl OrderService {
        pub fn new() -> Self {
            Self { next_id: 1 }
        }

        pub fn process_order(
            &mut self,
            total: u32,
            notifier: &dyn OrderNotifier,
        ) -> Result<Order, OrderError> {
            let order = Order {
                id: self.next_id,
                total,
            };
            self.next_id += 1;
            notifier.process(&order)?;
            Ok(order)
        }
    }
}

fn main() {
    use adapters::ConsoleNotifier;
    use application::OrderService;

    let notifier = ConsoleNotifier;
    let mut service = OrderService::new();

    match service.process_order(4999, &notifier) {
        Ok(order) => println!("Success! Order #{} processed.", order.id),
        Err(e) => println!("Error: {e}"),
    }
}
