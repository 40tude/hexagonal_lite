// 2 adapters on the same port
// cargo run --example ex04

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
    // Added
    use std::cell::RefCell;

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

    // Added
    pub struct InMemoryNotifier {
        messages: RefCell<Vec<String>>,
    }

    impl InMemoryNotifier {
        pub fn new() -> Self {
            Self {
                messages: RefCell::new(Vec::new()),
            }
        }

        pub fn messages(&self) -> Vec<String> {
            self.messages.borrow().clone()
        }
    }

    impl OrderNotifier for InMemoryNotifier {
        fn process(&self, order: &Order) -> Result<(), OrderError> {
            self.messages.borrow_mut().push(format!(
                "Order #{} stored, total = {}",
                order.id, order.total
            ));
            Ok(())
        }
    }
}

// The application does NOT own its notifier. It has a reference to the notifier (see `notifier: &'a N`)
mod application {
    use crate::domain::{Order, OrderError};
    use crate::ports::OrderNotifier;

    pub struct OrderService<'a, N: OrderNotifier> {
        notifier: &'a N,
        next_id: u32,
    }

    impl<'a, N: OrderNotifier> OrderService<'a, N> {
        pub fn new(notifier: &'a N) -> Self {
            Self {
                notifier,
                next_id: 1,
            }
        }

        pub fn process_order(&mut self, total: u32) -> Result<Order, OrderError> {
            let order = Order {
                id: self.next_id,
                total,
            };
            self.next_id += 1;
            self.notifier.process(&order)?;
            Ok(order)
        }
    }
}

fn main() {
    // use adapters::ConsoleNotifier;
    use adapters::{ConsoleNotifier, InMemoryNotifier};
    use application::OrderService;

    let notifier = ConsoleNotifier;
    let mut service = OrderService::new(&notifier);

    match service.process_order(4999) {
        Ok(order) => println!("Success! Order #{} processed.", order.id),
        Err(e) => println!("Error: {e}"),
    }

    //Added
    let memory_notifier = InMemoryNotifier::new();
    let mut memory_service = OrderService::new(&memory_notifier);
    memory_service.process_order(42).unwrap();

    for message in memory_notifier.messages() {
        println!("[Memory] {message}");
    }
}
