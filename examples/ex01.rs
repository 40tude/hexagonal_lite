// circus
// cargo run --example ex01

mod domain {
    use std::fmt;

    #[derive(Debug, Clone)]
    pub struct ClownAct {
        pub act_number: u32,
        pub silliness_level: u32,
    }

    #[derive(Debug)]
    pub enum CircusError {
        // ClownTrippedOnBanana,
    }

    impl fmt::Display for CircusError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{self:?}")
        }
    }
}

mod ports {
    use crate::domain::{CircusError, ClownAct};

    pub trait Announcer {
        fn announce(&self, act: &ClownAct) -> Result<(), CircusError>;
    }
}

mod adapters {
    use crate::domain::{CircusError, ClownAct};
    use crate::ports::Announcer;

    pub struct MegaphoneAnnouncer;

    impl Announcer for MegaphoneAnnouncer {
        fn announce(&self, act: &ClownAct) -> Result<(), CircusError> {
            println!(
                "[Megaphone] 🎪 Act #{} is ON! Silliness level: {}",
                act.act_number, act.silliness_level
            );
            Ok(())
        }
    }
}

mod application {
    use crate::domain::{CircusError, ClownAct};
    use crate::ports::Announcer;

    pub struct CircusService<A: Announcer> {
        announcer: A,
        next_act: u32,
    }

    impl<A: Announcer> CircusService<A> {
        pub fn new(announcer: A) -> Self {
            Self {
                announcer,
                next_act: 1,
            }
        }

        pub fn schedule_act(&mut self, silliness: u32) -> Result<ClownAct, CircusError> {
            let act = ClownAct {
                act_number: self.next_act,
                silliness_level: silliness,
            };
            self.next_act += 1;
            self.announcer.announce(&act)?;
            Ok(act)
        }
    }
}

fn main() {
    use adapters::MegaphoneAnnouncer;
    use application::CircusService;

    let mut circus = CircusService::new(MegaphoneAnnouncer);

    match circus.schedule_act(9001) {
        Ok(act) => println!("🤡 Success! Clown act #{} scheduled.", act.act_number),
        Err(e) => println!("Error: {e}"),
    }
}
