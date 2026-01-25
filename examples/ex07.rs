// cargo run --example ex07

// If you read this page : https://www.40tude.fr/docs/06_programmation/rust/022_solid/solid_04.html
// In the example ex_02_dip we invert dependencies using a single trait (Sender)
// and a single adapter (Email). That was enough to understand the core idea.
//
// But real applications talk to MANY external systems:
// databases, payment providers, notification services, caches...
//
// This example shows how DIP scales naturally using
// Hexagonal Architecture (a.k.a. Ports & Adapters).
//
// Nothing magical here. Just clear boundaries and Rust doing what it does best.
//
//
//
//
//
//
//
//
// =============================================================================
// DOMAIN Layer - Pure Business Concepts
// =============================================================================
// The domain is the heart of the application.
// It contains business vocabulary and business rules.
// No traits. No infrastructure. No frameworks.
mod domain {
    use std::fmt;

    // Strongly-typed identifiers make illegal states harder to represent.
    // These are "Value Objects": they represent business concepts.
    // OrderId isn't just a u32, it's a meaningful business identifier.
    // This makes our code speak the language of the business.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct OrderId(pub u32);

    #[derive(Debug, Clone, Copy)]
    pub struct Money(pub u32); // stored in cents

    #[derive(Debug, Clone)]
    pub struct LineItem {
        pub name: String,
        pub price: Money,
    }

    // The Order entity is pure business data + invariants.
    // Notice: no database stuff, no HTTP, no external dependencies.
    // Just what is needed to explain "What IS an order?"
    #[derive(Debug, Clone)]
    pub struct Order {
        pub id: OrderId,
        pub items: Vec<LineItem>,
        pub total: Money,
    }

    // Domain-level errors describe business failures,
    // not technical ones (no SQL errors, no HTTP codes).
    #[derive(Debug)]
    pub enum OrderError {
        InvalidOrder,
        PaymentFailed,
        StorageFailed,
        NotificationFailed,
    }

    impl fmt::Display for OrderError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{:?}", self)
        }
    }

    // Business rule:
    // An order must contain at least one item.
    // This validation lives in the domain: it's a business rule,
    // not a database constraint or an API validation.
    impl Order {
        pub fn new(id: OrderId, items: Vec<LineItem>) -> Result<Self, OrderError> {
            if items.is_empty() {
                return Err(OrderError::InvalidOrder);
            }

            let total = Money(items.iter().map(|item| item.price.0).sum());

            Ok(Order { id, items, total })
        }
    }
}

// =============================================================================
// PORTS - What the Domain Needs From the Outside World
// =============================================================================
// Ports are abstractions defined by the application/domain.
// They describe required capabilities, not implementations.
mod ports {
    use crate::domain::*;

    // Output port: persistence because "I need to store orders somewhere"
    // Could be PostgreSQL, MongoDB, a file, Redis... domain doesn't care.
    pub trait OrderRepository {
        fn save(&mut self, order: &Order) -> Result<(), OrderError>;
        fn find(&self, id: OrderId) -> Result<Option<Order>, OrderError>;
    }

    // Output port: payment processing because "I need to charge customers"
    // Could be Stripe, PayPal, a mock for testing... domain doesn't care.
    pub trait PaymentGateway {
        fn charge(&self, amount: Money) -> Result<(), OrderError>;
    }

    // Output port: notifications
    pub trait Sender {
        fn send(&self, order: &Order) -> Result<(), OrderError>;
    }
}

// =============================================================================
// APPLICATION Layer - Use Cases and Orchestration
// =============================================================================
// The application layer coordinates the business flow.
// It does NOT implement business rules and does NOT know adapters.
//
mod application {
    use crate::domain::*;
    use crate::ports::*;

    // OrderService is generic over its ports,
    // and it holds *references* to implementations.
    //
    // This means:
    // - adapters live elsewhere
    // - the service only temporarily borrows capabilities
    // - multiple services could share the same adapters
    pub struct OrderService<'a, R, P, N>
    where
        R: OrderRepository,
        P: PaymentGateway,
        N: Sender,
    {
        repository: &'a mut R,
        payment: &'a P,
        sender: &'a N,
        next_id: u32,
    }

    impl<'a, R, P, N> OrderService<'a, R, P, N>
    where
        R: OrderRepository,
        P: PaymentGateway,
        N: Sender,
    {
        // Dependency injection via references.
        // The application does not decide *what* implementations are used.
        // It only states *what it needs*.
        pub fn new(repository: &'a mut R, payment: &'a P, sender: &'a N) -> Self {
            Self {
                repository,
                payment,
                sender,
                next_id: 1,
            }
        }

        // This is the main use case:
        // "A customer places an order"
        pub fn place_order(&mut self, items: Vec<LineItem>) -> Result<Order, OrderError> {
            let order_id = OrderId(self.next_id);
            self.next_id += 1;

            // Step 1: pure business logic
            let order = Order::new(order_id, items)?;

            // Step 2: orchestrate external interactions
            // Notice how everything goes through ports.
            self.payment.charge(order.total)?;
            self.repository.save(&order)?;
            self.sender.send(&order)?;

            Ok(order)
        }

        pub fn get_order(&self, id: OrderId) -> Result<Option<Order>, OrderError> {
            self.repository.find(id)
        }
    }
}

// =============================================================================
// ADAPTERS - Concrete Implementations
// =============================================================================
// Adapters live at the edge of the system.
// They depend on ports, never the other way around.

// --- In-memory adapters (testing / development) ---
mod in_memory_adapters {
    use crate::domain::*;
    use crate::ports::*;
    use std::collections::HashMap;

    // A simple HashMap-based repository.
    // Perfect for unit tests: no database needed!
    pub struct InMemoryOrderRepository {
        orders: HashMap<OrderId, Order>,
    }

    impl InMemoryOrderRepository {
        pub fn new() -> Self {
            Self {
                orders: HashMap::new(),
            }
        }
    }

    // It implements the OrderRepository port.
    // The application doesn't know (or care) that this is a HashMap.
    impl OrderRepository for InMemoryOrderRepository {
        fn save(&mut self, order: &Order) -> Result<(), OrderError> {
            println!("  [InMemory] Saving order {:?}", order.id);
            self.orders.insert(order.id, order.clone());
            Ok(())
        }

        fn find(&self, id: OrderId) -> Result<Option<Order>, OrderError> {
            println!("  [InMemory] Finding order {:?}", id);
            Ok(self.orders.get(&id).cloned())
        }
    }

    // A mock payment gateway: always succeeds.
    // Great for testing the happy path!
    pub struct MockPaymentGateway;

    impl PaymentGateway for MockPaymentGateway {
        fn charge(&self, amount: Money) -> Result<(), OrderError> {
            println!(
                "  [MockPayment] Charging ${}.{:02}",
                amount.0 / 100,
                amount.0 % 100
            );
            Ok(())
        }
    }

    // Console-based notification: just prints to stdout.
    pub struct ConsoleSender;

    impl Sender for ConsoleSender {
        fn send(&self, order: &Order) -> Result<(), OrderError> {
            println!(
                "  [Console] Order {:?} confirmed, total ${}.{:02}",
                order.id,
                order.total.0 / 100,
                order.total.0 % 100
            );
            Ok(())
        }
    }
}

// --- Adapter Set #2: External Services (for production) ---
// Same ports, completely different implementations.
// If we swap these and our application works with real services!
mod external_adapters {
    use crate::domain::*;
    use crate::ports::*;
    use std::collections::HashMap;

    // A "simulated" PostgreSQL adapter.
    // In real life, this would use sqlx, diesel, or similar.
    pub struct PostgresOrderRepository {
        simulated_db: HashMap<OrderId, Order>,
    }

    impl PostgresOrderRepository {
        pub fn new() -> Self {
            Self {
                simulated_db: HashMap::new(),
            }
        }
    }

    impl OrderRepository for PostgresOrderRepository {
        fn save(&mut self, order: &Order) -> Result<(), OrderError> {
            println!("  [Postgres] INSERT order {:?}", order.id);
            self.simulated_db.insert(order.id, order.clone());
            Ok(())
        }

        fn find(&self, id: OrderId) -> Result<Option<Order>, OrderError> {
            println!("  [Postgres] SELECT order {:?}", id);
            Ok(self.simulated_db.get(&id).cloned())
        }
    }

    // A "simulated" Stripe adapter.
    // In real life, this would call the Stripe API.
    pub struct StripePaymentGateway;

    impl PaymentGateway for StripePaymentGateway {
        fn charge(&self, amount: Money) -> Result<(), OrderError> {
            println!(
                "  [Stripe] Charging ${}.{:02}",
                amount.0 / 100,
                amount.0 % 100
            );
            Ok(())
        }
    }

    // A "simulated" SendGrid adapter for sending emails.
    // Same Sender trait as ConsoleSender, but talks to an email API.
    pub struct SendGridSender;

    impl Sender for SendGridSender {
        fn send(&self, order: &Order) -> Result<(), OrderError> {
            println!("  [SendGrid] Sending confirmation for order {:?}", order.id);
            Ok(())
        }
    }
}

// =============================================================================
// MAIN - Composition Root
// =============================================================================
// This is the ONLY place where concrete implementations are chosen.
// The application never sees this wiring.
// We can switch from "test mode" to "production mode" just by swapping adapters.
// No changes to business logic. No changes to the application layer.
// That's the power of Hexagonal Architecture!
fn main() {
    use application::OrderService;
    use domain::{LineItem, Money};
    use external_adapters::*;
    use in_memory_adapters::*;

    let items = vec![
        LineItem {
            name: "Rust Book".to_string(),
            price: Money(4999),
        },
        LineItem {
            name: "Keyboard".to_string(),
            price: Money(12999),
        },
    ];

    // --- Configuration #1: In-Memory Adapters ---
    // Perfect for testing! No external dependencies needed.
    // In ex_02_dip, this is like injecting a MockSender.
    // Here, we inject mocks for ALL our ports.
    println!("--- In-memory configuration ---\n");
    {
        let mut repo = InMemoryOrderRepository::new();
        let payment = MockPaymentGateway;
        let sender = ConsoleSender;

        let mut service = OrderService::new(&mut repo, &payment, &sender);
        // let _ = service.place_order(items.clone());
        match service.place_order(items.clone()) {
            Ok(order) => println!("\n  Success! Order {:?} placed.\n", order.id),
            Err(e) => println!("\n  Error: {}\n", e),
        }
    }

    // --- Configuration #2: External Services ---
    // Ready for production! Real database, real payment, real emails.
    // Notice: we didn't change a single line in OrderService or domain.
    // We just plugged in different adapters. That's DIP at scale!
    println!("\n--- External services configuration ---\n");
    {
        let mut repo = PostgresOrderRepository::new();
        let payment = StripePaymentGateway;
        let sender = SendGridSender;

        let mut service = OrderService::new(&mut repo, &payment, &sender);
        // let _ = service.place_order(items);
        match service.place_order(items.clone()) {
            Ok(order) => {
                println!("\n  Success! Order {:?} placed.", order.id);

                // Let's also test retrieval
                println!();
                if let Ok(Some(retrieved)) = service.get_order(order.id) {
                    println!(
                        "  Retrieved: {} items, total ${}.{:02}\n",
                        retrieved.items.len(),
                        retrieved.total.0 / 100,
                        retrieved.total.0 % 100
                    );
                }
            }
            Err(e) => println!("\n  Error: {}\n", e),
        }
    }
}
