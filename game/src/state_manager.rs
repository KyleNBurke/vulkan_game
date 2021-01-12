use glfw::Window;
use engine::{Handle, Renderer, Scene};

pub struct GameResources {
	pub roboto_32: Handle,
	pub roboto_14: Handle
}

pub struct EngineResources {
	pub window: Window,
	pub renderer: Renderer,
	pub game_resources: GameResources,
	pub scene: Scene
}

pub trait State {
	fn enter(&mut self, resources: &mut EngineResources);
	fn leave(&mut self, resources: &mut EngineResources);
	fn handle_event(&mut self, event: &glfw::WindowEvent, resources: &mut EngineResources);
	fn update(&mut self, resources: &mut EngineResources) -> StateAction;
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
	pub fn new(resources: &mut EngineResources, mut initial_state: Box<dyn State>) -> Self {
		initial_state.enter(resources);

		Self { states: vec![initial_state] }
	}

	pub fn handle_event(&mut self, event: &glfw::WindowEvent, resources: &mut EngineResources) {
		for state in &mut self.states {
			state.handle_event(event, resources);
		}
	}

	pub fn update(&mut self, resources: &mut EngineResources) {
		let mut actions = Vec::with_capacity(self.states.len());

		for state in &mut self.states {
			actions.push(state.update(resources));
		}

		for action in actions {
			match action {
				StateAction::Push(mut state) => {
					state.enter(resources);
					self.states.push(state);
				},
				StateAction::Pop => {
					self.states.last_mut().unwrap().leave(resources);
					self.states.pop();
				},
				_ => ()
			}
		}
	}
}