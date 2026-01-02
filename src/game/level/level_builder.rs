use rand_pcg::Pcg64;
use rand::Rng;

use crate::{core::math::{random::Random, unit_conversions::cm_to_m, vec2::Vec2}, game::{entity::entity_system::EntitySystem, level::{level_blocks::{cliff_operation::CliffOperation, drop_direction_reverse::DropDirectionReverse, elevator::ElevatorOperation, finish_operation::FinishOperation, fluid_funnel::FluidFunnel, hill_operation::HillOperation, saggy_bridge_operation::SaggyBridgeOperation, spawn_operation::SpawnOperation, straight_level_block::StraightLevelBlock, water_balloon_drop::WaterBalloonDrop}, level_builder_operation::LevelBuilderOperation, level_builder_operation_registry::LevelBuilderOperationRegistry}}, simulation::particles::{particle::Particle, particle_vec::ParticleVec, simulation::Simulation}};

pub struct LevelBuilder {
    level_builder_operations_registry: LevelBuilderOperationRegistry,
}

impl LevelBuilder {
    pub fn new(level_builder_operations_registry: LevelBuilderOperationRegistry) -> Self {
        Self {
            level_builder_operations_registry,
        }
    }
}

pub struct LevelBuilderContext<'a> {
    pub particle_vec: &'a mut ParticleVec, //pub particle_sim: &'a mut ParticleSim,
    pub cursor: Vec2,
    pub x_direction: f32, // which way the cursor is pointing
    pub x_direction_changed: bool,
    pub particle_template: Particle,
    pub operations: Vec<Box<dyn LevelBuilderOperation + Send + Sync>>,
    pub is_first: bool,
    pub is_last: bool,
    pub rng: &'a mut Pcg64,
    pub entity_system: &'a mut EntitySystem,
    pub sim: &'a mut Simulation,
}

impl<'a> LevelBuilderContext<'a> {
    pub fn new(entity_system: &'a mut EntitySystem, particle_vec: &'a mut ParticleVec, sim: &'a mut Simulation, rng: &'a mut Pcg64) -> Self {
        let particle_radius = cm_to_m(10.0); // was 4.0

        Self {
            particle_vec, //particle_sim,
            cursor: Vec2::new(0.0, 0.0),
            x_direction: 1.0,
            x_direction_changed: false,
            particle_template: Particle::default().set_radius(particle_radius).clone(),
            operations: vec![],
            is_first: true,
            is_last: false,
            rng,
            entity_system,
            sim
        }
    }
}

impl LevelBuilder {
    pub fn generate_level_based_on_date(&mut self, entity_system: &mut EntitySystem, particle_vec: &mut ParticleVec, sim: &mut Simulation) {
        // set a random seed used for level generation based on todays date. Each day we get a new map to try
        let mut rng = Random::seed_from_beginning_of_day(); //seed_from_beginning_of_week(); //car_scene.rng;
        
        let mut level_builder_context = LevelBuilderContext::new(entity_system, particle_vec, sim, &mut rng);
        self.generate(&mut level_builder_context, 10); //10); //10);

        // todo: we should push the seed and # level blocks into the event system
    }

    pub fn generate(&mut self, level_builder_context: &mut LevelBuilderContext, num_blocks: i32) -> &mut Self {
        // Algorithm to generate a level
        // 1. Set cursor to origin. This is where the car will spawn (well, a bit behind)
        // 2. Generate a block, which will adjust the cursor

        // currently I spawn an amount of blocks. It might be better to keep spawning blocks till we get a certain distance? or a combination? 
        for bi in 0..num_blocks {
            level_builder_context.is_first = bi == 0;
            level_builder_context.is_last = bi == (num_blocks - 1);

            // 1. Create a pair of "spawn change" and a operation.
            let mut spawn_chance_operations = vec![];
            for op in self.level_builder_operations_registry.iter() {
                spawn_chance_operations.push((op.as_ref().default_spawn_chance(), op.as_ref().box_clone()))
            }

            // 2. Give each operation a chance to mutate "spawn_chance_operations".
            for op in self.level_builder_operations_registry.iter() {
                op.as_ref().prepare(level_builder_context, &mut spawn_chance_operations);
            }

            // 3. Select an operation
            let mut spawn_chance_total = 0.0;
            for (chance, _) in &spawn_chance_operations {
                spawn_chance_total += chance;
            }
            if spawn_chance_total <= 0.0 {
                // nothing to spawn!
                continue;
            }

            // 4. Find the selected operation and execute it
            let mut spawn_value = level_builder_context.rng.random_range(0.0..spawn_chance_total);
            for (chance, operation) in &spawn_chance_operations {
                spawn_value -= chance;
                if spawn_value <= 0.0 {
                    // pick this item!
                    level_builder_context.operations.push(operation.box_clone());
                    operation.execute(level_builder_context);
                    break;
                }
            }
        }

        // let particle system know all static particles have been built - can we move this into create_in_particle_sim?
        //level_builder_context.particle_sim.notify_particle_container_changed();

        self
    }
}


impl Default for LevelBuilder {
    fn default() -> Self {
        let mut registry = LevelBuilderOperationRegistry::new();

        // here is our registry
        //
        // things to try:
        // - a jelly draw bridge you drive into and it falls over
        // - a flexible curved pipe that changes direction and flips the car over at the same time
        // - a big ball you drive onto and keep it rolling forwards to get to the other side
        // - an elevator
        // - a steep incline with toothed or flexible ground to give you grip to get up step. (or change the car tyres to be spiked)
        // - some cloth you need to drive under/tear through
        //
        // instead of picking random numbers in a range, pick a random integer and just quantize the number eg. pick a number and then * by 0.5 to get 0.5, 1.0, 1.5, 2.0 as random distances. this might provide more "variety" through less choice.
        // we should keep a bounding box for each operation applied to help work out if a block can be used instead of using x_direction_changed for example
        registry.register(SpawnOperation {});
        registry.register(FinishOperation {});
        registry.register(HillOperation {});


        registry.register(WaterBalloonDrop {});
        registry.register(SaggyBridgeOperation {});
        registry.register(StraightLevelBlock {});
        registry.register(CliffOperation {});
        registry.register(FluidFunnel {});
        registry.register(DropDirectionReverse {});
        registry.register(ElevatorOperation {});
        

        //registry.register(JellyCube {});
 
        LevelBuilder::new(registry)
    }
}