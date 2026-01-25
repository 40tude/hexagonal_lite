// Multiple ports, multiple adapters (application borrows adapters)

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
    pub trait OrderRepository {
        fn save(&mut self, order: &Order) -> Result<(), OrderError>;
        fn find(&self, id: u32) -> Result<Option<Order>, OrderError>;
    }
}

mod adapters {
    use crate::domain::{Order, OrderError};
    use crate::ports::{OrderNotifier, OrderRepository};
    use std::collections::HashMap;

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

    pub struct InMemoryOrderRepository {
        orders: HashMap<u32, Order>,
    }

    impl InMemoryOrderRepository {
        pub fn new() -> Self {
            Self {
                orders: HashMap::new(),
            }
        }
    }

    impl OrderRepository for InMemoryOrderRepository {
        fn save(&mut self, order: &Order) -> Result<(), OrderError> {
            println!("[InMemory] Saving order #{}", order.id);
            self.orders.insert(order.id, order.clone());
            Ok(())
        }

        fn find(&self, id: u32) -> Result<Option<Order>, OrderError> {
            println!("[InMemory] Finding order #{id}");
            Ok(self.orders.get(&id).cloned())
        }
    }
}

mod application {
    use crate::domain::{Order, OrderError};
    use crate::ports::{OrderNotifier, OrderRepository};

    pub struct OrderService<'a, R: OrderRepository, N: OrderNotifier> {
        repository: &'a mut R,
        notifier: &'a N,
        next_id: u32,
    }

    impl<'a, R: OrderRepository, N: OrderNotifier> OrderService<'a, R, N> {
        pub fn new(repository: &'a mut R, notifier: &'a N) -> Self {
            Self {
                repository,
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

            self.repository.save(&order)?;
            self.notifier.process(&order)?;

            Ok(order)
        }

        pub fn get_order(&self, id: u32) -> Result<Option<Order>, OrderError> {
            self.repository.find(id)
        }
    }
}

fn main() {
    use adapters::{ConsoleNotifier, InMemoryOrderRepository};
    use application::OrderService;

    let mut repo = InMemoryOrderRepository::new();
    let notifier = ConsoleNotifier;

    let mut service = OrderService::new(&mut repo, &notifier);

    match service.process_order(4999) {
        Ok(order) => println!("Success! Order #{} processed.\n", order.id),
        Err(e) => println!("Error: {e}\n"),
    }

    println!("Retrieving order #1...");
    match service.get_order(1) {
        Ok(Some(order)) => println!("Found: Order #{}, total: {}", order.id, order.total),
        Ok(None) => println!("Order not found"),
        Err(e) => println!("Error: {e}"),
    }
}
