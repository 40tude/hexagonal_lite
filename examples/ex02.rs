// test only
// cargo test --example ex02

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

mod application {
    use crate::domain::{Order, OrderError};
    use crate::ports::OrderNotifier;

    pub struct OrderService<N: OrderNotifier> {
        notifier: N,
        next_id: u32,
    }

    impl<N: OrderNotifier> OrderService<N> {
        pub fn new(notifier: N) -> Self {
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

#[cfg(test)]
mod tests {
    use crate::application::OrderService;
    use crate::domain::{Order, OrderError};
    use crate::ports::OrderNotifier;

    struct TestNotifier;

    impl OrderNotifier for TestNotifier {
        fn process(&self, _order: &Order) -> Result<(), OrderError> {
            Ok(())
        }
    }

    #[test]
    fn process_order_successfully() {
        let mut service = OrderService::new(TestNotifier);

        let order = service.process_order(4999).unwrap();

        assert_eq!(order.id, 1);
        assert_eq!(order.total, 4999);
    }
}
