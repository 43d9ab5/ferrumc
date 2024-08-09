use ferrumc_macros::AutoGenName;
use crate::net::systems::System;
use crate::state::GlobalState;

#[derive(AutoGenName)]
pub struct CommandSystem {

}


impl CommandSystem {
    pub fn new() -> Self {
        CommandSystem {

        }
    }
}

impl System for CommandSystem {
    async fn run(&self, _state: GlobalState) {
        todo!()
    }

    fn name(&self) -> &'static str {
        Self::type_name()
    }
}