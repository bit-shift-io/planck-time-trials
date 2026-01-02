use std::{env, time::Instant};

use crate::{
    core::math::vec2::Vec2,
    engine::{
        app::{
            camera::{Camera, CameraController},
            context::Context,
            game_loop::GameLoop,
        },
        renderer::{
            instance_renderer::{Instance, InstanceRaw, InstanceRenderer, QUAD_INDICES, QUAD_VERTICES, Vertex},
            model::{Material, Mesh},
            shader::{Shader, ShaderBuilder},
        },
    },
    game::{
        entity::{entities::car_entity::CarEntity, entity_system::EntitySystem},
        level::level_builder::LevelBuilder,
        irc::irc_manager::{IrcManager, IrcEvent},
        leaderboard::Leaderboard,
        game_state::GameState,
        settings::Settings,
    },
    simulation::particles::{particle_vec::ParticleVec, simulation::Simulation, simulation_demos::SimulationDemos},
};
use crate::engine::app::event_system::{GameEvent, ElementStateType, KeyCodeType};
use cgmath::Rotation3;

pub struct Game {
    camera: Camera,
    camera_controller: CameraController,
    particle_vec: ParticleVec,
    particle_instance_renderer: InstanceRenderer,
    quad_mesh: Mesh,
    material: Material,
    particle_shader: Shader,
    line_shader: Shader,
    frame_idx: u128,
    entity_system: EntitySystem,
    simulation: Simulation,
    total_time: f32,
    game_state: GameState,
    irc_manager: Option<IrcManager>,
    current_nickname: String,
    leaderboard: Leaderboard,
    ui: crate::game::ui::game_ui::GameUI,
}

impl Game {
    fn update_particle_instances(&mut self, queue: &wgpu::Queue, device: &wgpu::Device) {
        let mut instances: Vec<Instance> = vec![]; 
        let particles = &self.simulation.particles;

        for i in 0..particles.len() {
            let position = cgmath::Vector3 {
                x: particles[i].pos[0],
                y: particles[i].pos[1],
                z: 0.0,
            };

            let rotation = cgmath::Quaternion::from_axis_angle(
                cgmath::Vector3::unit_z(),
                cgmath::Deg(0.0),
            );

            let colour = particles[i].colour;
            let radius = particles[i].radius;

            instances.push(Instance { position, rotation, colour, radius });
        }
        self.particle_instance_renderer.update_instances(&instances, queue, device);
    }
    pub fn reset(&mut self, ctx: &mut Context) {
        self.total_time = 0.0;
        self.game_state = GameState::Playing;
        self.frame_idx = 0;
        
        // Re-initialize systems
        self.entity_system = EntitySystem::new();
        self.particle_vec = ParticleVec::new();
        
        let rng = crate::core::math::random::Random::seed_from_beginning_of_day();
        self.simulation = Simulation::new(rng);
        
        // Re-generate level
        LevelBuilder::default().generate_level_based_on_date(&mut self.entity_system, &mut self.particle_vec, &mut self.simulation);
        let car = CarEntity::new(&mut self.particle_vec, &mut self.simulation, Vec2::new(0.0, 1.0));
        self.entity_system.car_entity_system.push(car);
        
        // Update UI
        self.ui.update(crate::game::ui::game_ui::Message::UpdateGameState(GameState::Playing));
        self.ui.update(crate::game::ui::game_ui::Message::UpdateTime(0.0));
        
        // Reset recording if necessary
        let args: Vec<String> = env::args().collect();
        let scene = if args.len() >= 2 { args[1].clone() } else { String::from("") };
        let is_demo_scene = matches!(scene.as_str(), "friction" | "granular" | "sdf" | "boxes" | "wall" | "pendulum" | "rope" | "fluid" | "fluid_solid" | "gas" | "water_balloon" | "newtons_cradle" | "smoke_open" | "smoke_closed" | "rope_gas" | "volcano" | "wrecking_ball");
        
        if !is_demo_scene {
            ctx.event_system.start_recording();
        }
        
        self.update_particle_instances(&ctx.graphics.queue, &ctx.graphics.device);
    }

    pub fn step_simulation(&mut self, time_delta: f32) -> f32 {
        let start = Instant::now();
        
        self.simulation.pre_solve(time_delta);
        self.entity_system.elevator_entity_system.update_counts(&mut self.simulation);

        for i in 0..3 {
            self.simulation.solve(time_delta, 3, i);
            self.entity_system.elevator_entity_system.solve_constraints(&mut self.simulation, time_delta);
        }
        self.simulation.post_solve(time_delta);

        start.elapsed().as_secs_f32() * 1000.0
    }
}

impl GameLoop for Game {
    fn new(ctx: &mut Context) -> Self {
        let camera_controller = CameraController::new(0.2);
        let mut entity_system = EntitySystem::new();
        let mut particle_vec = ParticleVec::new();
        
        let rng = crate::core::math::random::Random::seed_from_beginning_of_day();
        let mut simulation = Simulation::new(rng);

        let particle_instance_renderer = InstanceRenderer::new(&ctx.graphics.device, &ctx.graphics.queue, &ctx.graphics.config);
        let quad_mesh = Mesh::from_verticies_and_indicies("Quad".to_owned(), &ctx.graphics.device, QUAD_VERTICES, QUAD_INDICES);
        let material = Material::from_file("marble.png".to_owned(), &ctx.graphics.device, &ctx.graphics.queue);
        let camera = Camera::new(&ctx.graphics.device, ctx.graphics.config.width as f32 / ctx.graphics.config.height as f32);
        
        let diffuse_texture = &material.diffuse_texture;

        let particle_shader = ShaderBuilder::from_file("particle_shader.wgsl".to_owned(), &ctx.graphics.device)
            .camera(&camera)
            .diffuse_texture(diffuse_texture)
            .build(&[Vertex::desc(), InstanceRaw::desc()], ctx.graphics.config.format);
        
        let line_shader = ShaderBuilder::from_file("line_shader.wgsl".to_owned(), &ctx.graphics.device)
            .camera(&camera)
            .build(&[Vertex::desc(), InstanceRaw::desc()], ctx.graphics.config.format);

        let args: Vec<String> = env::args().collect();
        let scene = if args.len() >= 2 { args[1].clone() } else { String::from("") };
        
        let replay_file = if args.len() >= 3 && args[1] == "replay" {
            Some(args[2].clone())
        } else {
            None
        };
        
        let is_demo_scene = match scene.as_str() {
            "friction" => { SimulationDemos::init_friction(&mut simulation); true }
            "granular" => { SimulationDemos::init_granular(&mut simulation); true }
            "sdf" => { SimulationDemos::init_sdf(&mut simulation); true }
            "boxes" => { SimulationDemos::init_boxes(&mut simulation); true }
            "wall" => { SimulationDemos::init_wall(&mut simulation); true }
            "pendulum" => { SimulationDemos::init_pendulum(&mut simulation); true }
            "rope" => { SimulationDemos::init_rope(&mut simulation); true }
            "fluid" => { SimulationDemos::init_fluid(&mut simulation); true }
            "fluid_solid" => { SimulationDemos::init_fluid_solid(&mut simulation); true }
            "gas" => { SimulationDemos::init_gas(&mut simulation); true }
            "water_balloon" => { SimulationDemos::init_water_balloon(&mut simulation); true }
            "newtons_cradle" => { SimulationDemos::init_newtons_cradle(&mut simulation); true }
            "smoke_open" => { SimulationDemos::init_smoke_open(&mut simulation); true }
            "smoke_closed" => { SimulationDemos::init_smoke_closed(&mut simulation); true }
            "rope_gas" => { SimulationDemos::init_rope_gas(&mut simulation); true }
            "volcano" => { SimulationDemos::init_volcano(&mut simulation); true }
            "wrecking_ball" => { SimulationDemos::init_wrecking_ball(&mut simulation); true }
            "replay" | _ => {
                LevelBuilder::default().generate_level_based_on_date(&mut entity_system, &mut particle_vec, &mut simulation);
                let car = CarEntity::new(&mut particle_vec, &mut simulation, Vec2::new(0.0, 1.0));
                entity_system.car_entity_system.push(car);
                false
            }
        };

        if let Some(replay_path) = replay_file {
            if let Err(e) = ctx.event_system.load_replay(&replay_path) {
                eprintln!("Failed to load replay file '{}': {}", replay_path, e);
            } else {
                ctx.event_system.start_replay();
            }
        } else if !is_demo_scene {
            ctx.event_system.start_recording();
        }

        let settings = Settings::load();
        let (game_state, nickname) = if let Some(name) = settings.player_name {
            (GameState::Playing, name)
        } else {
            (GameState::NameEntry, format!("Player{}", chrono::Utc::now().timestamp_subsec_micros()))
        };

        let irc_manager = if game_state == GameState::Playing {
            Some(IrcManager::new(
                 "irc.libera.chat".to_owned(),
                 nickname.clone(),
                 vec!["#planck-global".to_owned(), "#planck-leaderboard".to_owned()]
            ))
        } else {
            None
        };

        let mut ui = crate::game::ui::game_ui::GameUI::new();
        ui.update(crate::game::ui::game_ui::Message::UpdateGameState(game_state));
        ui.update(crate::game::ui::game_ui::Message::UpdateShowDebugInfo(settings.show_debug_info.unwrap_or(true)));

        let mut game = Self {
            camera,
            camera_controller,
            particle_vec,
            particle_instance_renderer,
            quad_mesh,
            material,
            particle_shader,
            line_shader,
            frame_idx: 0,
            entity_system,
            simulation,
            total_time: 0.0,
            game_state,
            irc_manager,
            current_nickname: nickname,
            leaderboard: Leaderboard::new(),
            ui,
        };

        game.update_particle_instances(&ctx.graphics.queue, &ctx.graphics.device);
        game
    }

    fn update(&mut self, ctx: &mut Context) {
        let start = Instant::now();
        let dt = if ctx.dt <= 0.0 { 1.0 / 60.0 } else { ctx.dt };
        let fps = (1.0 / dt).round() as i32;
        self.ui.update(crate::game::ui::game_ui::Message::UpdateFps(fps));

        self.frame_idx += 1;
        ctx.event_system.set_frame(self.frame_idx);
        ctx.event_system.process_events();

        let mut should_reset = false;
        for event in ctx.event_system.events.iter() {
            match event {
                GameEvent::KeyboardInput { key_code, state } => {
                    let is_pressed = matches!(state, ElementStateType::Pressed);
                    self.camera_controller.handle_key(*key_code, is_pressed);
                    self.entity_system.handle_key(*key_code, is_pressed);
                    
                    if *key_code == KeyCodeType::KeyR && is_pressed && self.game_state == GameState::Finished {
                        should_reset = true;
                    }
                }
                _ => {}
            }
        }
        
        if should_reset {
            self.reset(ctx);
        }
        ctx.event_system.clear_events();

        if self.game_state == GameState::NameEntry {
            let elapsed = start.elapsed().as_secs_f32() * 1000.0;
            self.ui.update(crate::game::ui::game_ui::Message::UpdateUpdateTime(elapsed));
            return;
        }

        let time_delta: f32 = 0.005;
        let sim_time = self.step_simulation(time_delta);
        self.ui.update(crate::game::ui::game_ui::Message::UpdateSimulationTime(sim_time));
        
        self.camera_controller.update_camera(&mut self.camera);

        if self.game_state == GameState::Playing {
            self.total_time += time_delta;
            self.ui.update(crate::game::ui::game_ui::Message::UpdateTime(self.total_time));
        }
        self.entity_system.update(&mut self.particle_vec, &mut self.simulation, &mut self.camera, time_delta, self.total_time);

        if self.game_state == GameState::Playing {
            let game_finished = self.entity_system.car_entity_system.0.iter().any(|car| car.game_ended);
            if game_finished {
                self.game_state = GameState::Finished;
                self.ui.update(crate::game::ui::game_ui::Message::UpdateGameState(GameState::Finished));
                
                if ctx.event_system.is_recording() {
                    ctx.event_system.stop_recording();
                    let filename = "recording.json";
                    let _ = ctx.event_system.export_recording(&filename);
                }
                
                let seed = chrono::Utc::now().format("%Y-%m-%d").to_string();
                let msg = format!("BEST_TIME seed={} time={:.3} user={}", seed, self.total_time, self.current_nickname);
                if let Some(irc) = &self.irc_manager {
                    irc.send_message("#planck-leaderboard".to_owned(), msg);
                }
                
                self.leaderboard.add_score(seed.clone(), self.current_nickname.clone(), self.total_time);

                let entries = self.leaderboard.get_leaderboard_entries(&seed, &self.current_nickname, Some(self.total_time));
                self.ui.update(crate::game::ui::game_ui::Message::UpdateLeaderboardResults(entries));

                if let Some(top10) = self.leaderboard.get_top_10(&seed) {
                    if let Some(irc) = &self.irc_manager {
                        irc.send_message("#planck-global".to_owned(), top10);
                    }
                }
            }
        }

        self.camera.update_camera_uniform(&ctx.graphics.queue);
        self.update_particle_instances(&ctx.graphics.queue, &ctx.graphics.device);

        if let Some(irc) = &self.irc_manager {
            for event in irc.process_events() {
                match event {
                    IrcEvent::MessageReceived { target, message, .. } => {
                        if target == "#planck-leaderboard" {
                            let seed = chrono::Utc::now().format("%Y-%m-%d").to_string();
                            if message.starts_with("BEST_TIME") {
                                self.leaderboard.parse_message(&message);
                                if let Some(sync_msg) = self.leaderboard.serialize_sync(&seed) {
                                    irc.send_message("#planck-leaderboard".to_owned(), sync_msg);
                                }
                            } else if message.starts_with("LEADERBOARD_SYNC") {
                                self.leaderboard.parse_sync_message(&message);
                            }
                            let current_run_time = if self.game_state == GameState::Finished { Some(self.total_time) } else { None };
                            let entries = self.leaderboard.get_leaderboard_entries(&seed, &self.current_nickname, current_run_time);
                            self.ui.update(crate::game::ui::game_ui::Message::UpdateLeaderboardResults(entries));
                        }
                    },
                    _ => {}
                }
            }
        }

        let elapsed = start.elapsed().as_secs_f32() * 1000.0;
        self.ui.update(crate::game::ui::game_ui::Message::UpdateUpdateTime(elapsed));
    }

    fn render(&mut self, ctx: &mut Context) {
        let start = Instant::now();
        let output = match ctx.graphics.surface.get_current_texture() {
            Ok(output) => output,
            Err(_) => return,
        };
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = ctx.graphics.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.1, g: 0.2, b: 0.3, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &ctx.graphics.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            self.particle_shader.bind(&mut render_pass);
            self.material.bind(&mut render_pass, 0);
            self.particle_instance_renderer.render(&mut render_pass);
            self.quad_mesh.render(&mut render_pass, 0..1);
        }
        
        ctx.graphics.queue.submit(std::iter::once(encoder.finish()));

        // Use UI Helper for rendering
        let ui_messages = ctx.ui.draw(self.ui.view(), &ctx.graphics, &view);

        for msg in ui_messages {
            match msg {
                crate::game::ui::game_ui::Message::SubmitName => {
                    if !self.ui.name_input.trim().is_empty() {
                        self.current_nickname = self.ui.name_input.trim().to_string();
                        let settings = Settings {
                            player_name: Some(self.current_nickname.clone()),
                            show_debug_info: Some(self.ui.show_debug_info),
                        };
                        let _ = settings.save();

                        self.irc_manager = Some(IrcManager::new(
                             "irc.libera.chat".to_owned(),
                             self.current_nickname.clone(),
                             vec!["#planck-global".to_owned(), "#planck-leaderboard".to_owned()]
                        ));

                        self.game_state = GameState::Playing;
                        self.ui.update(crate::game::ui::game_ui::Message::UpdateGameState(GameState::Playing));
                    }
                }
                _ => self.ui.update(msg),
            }
        }

        let elapsed = start.elapsed().as_secs_f32() * 1000.0;
        self.ui.update(crate::game::ui::game_ui::Message::UpdateRenderTime(elapsed));

        output.present();
    }
}