use crate::Scene;

pub trait State<T> {
	fn enter(&self, scene: &mut Scene);
	fn leave(&self, scene: &mut Scene);
	fn update(&self, scene: &mut Scene, data: &mut T) -> StateAction<T>;
}

pub enum StateAction<T> {
	None,
	Push(Box<dyn State<T>>),
	Pop
}

pub struct StateManager<T> {
	states: Vec<Box<dyn State<T>>>
}

impl<T> StateManager<T> {
	pub fn new(scene: &mut Scene, initial_state: Box<dyn State<T>>) -> Self {
		initial_state.enter(scene);
		
		Self { states: vec![initial_state] }
	}

	pub fn update(&mut self, scene: &mut Scene, data: &mut T) {
		let mut actions = Vec::with_capacity(self.states.len());

		for state in &self.states {
			actions.push(state.update(scene, data));
		}

		for action in actions {
			match action {
				StateAction::Push(state) => {
					state.enter(scene);
					self.states.push(state);
				},
				StateAction::Pop => {
					self.states.last().unwrap().leave(scene);
					self.states.pop();
				},
				_ => ()
			}
		}
	}
}