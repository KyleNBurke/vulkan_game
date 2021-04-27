use std::time::Duration;
use engine::{glfw, pool::Handle, Renderer, Scene, Font};

pub struct GameResources {
	pub roboto_14: Handle<Font>
}

pub struct EngineResources {
	pub window: glfw::Window,
	pub renderer: Renderer,
	pub game_resources: GameResources,
	pub scene: Scene
}

pub trait State {
	fn enter(&mut self, _resources: &mut EngineResources) {}
	fn leave(&mut self, _resources: &mut EngineResources) {}
	fn handle_event(&mut self, _event: &glfw::WindowEvent, _resources: &mut EngineResources) {}
	fn update(&mut self, resources: &mut EngineResources, frame_time: &Duration) -> StateAction;
}

#[allow(dead_code)]
pub enum StateAction {
	None,
	Push(Box<dyn State>),
	Pop
}

pub struct StateManager {
	states: Vec<Box<dyn State>>
}

impl StateManager {
	pub fn new() -> Self {
		Self { states: vec![] }
	}

	pub fn push(&mut self, resources: &mut EngineResources, mut state: Box<dyn State>) {
		state.enter(resources);
		self.states.push(state);
	}

	pub fn pop(&mut self, resources: &mut EngineResources) {
		self.states.last_mut().unwrap().leave(resources);
		self.states.pop();
	}

	pub fn handle_event(&mut self, event: &glfw::WindowEvent, resources: &mut EngineResources) {
		for state in &mut self.states {
			state.handle_event(event, resources);
		}
	}

	pub fn update(&mut self, resources: &mut EngineResources, frame_time: &Duration) {
		let mut actions = Vec::with_capacity(self.states.len());

		for state in &mut self.states {
			actions.push(state.update(resources, frame_time));
		}

		for action in actions {
			match action {
				StateAction::Push(state) => {
					self.push(resources, state);
				},
				StateAction::Pop => {
					self.pop(resources);
				},
				_ => ()
			}
		}
	}
}