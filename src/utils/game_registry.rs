use std::{collections::HashMap, sync::OnceLock};

use crate::{games::*, prelude::*};

static GAME_REGISTRY: OnceLock<GameRegistry> = OnceLock::new();

pub struct GameRegistry {
    factories: HashMap<String, GameFactory>,
}

impl GameRegistry {
    pub fn new() -> Self {
        let mut registry: GameRegistry = Self {
            factories: HashMap::new(),
        };
        registry.register("Qafoon", || Box::new(Qafoon::new()));
        registry
    }

    pub fn register(&mut self, name: &str, factory: GameFactory) {
        self.factories.insert(name.to_string(), factory);
    }

    pub fn create_game(&self, name: &str) -> Result<BoxGame> {
        let factory: &GameFactory = self
            .factories
            .get(name)
            .ok_or_else(|| Error::Other(format!("Game {} is not supported", name)))?;
        Ok(factory())
    }
}

pub fn get_game_registry() -> &'static GameRegistry {
    GAME_REGISTRY.get_or_init(GameRegistry::new)
}

pub fn create_game(name: &str) -> Result<BoxGame> {
    get_game_registry().create_game(name)
}
