use std::time::Duration;
use engine::Handle;
use crate::{EngineResources, State, StateAction, Text};

const UPDATE_INTERVAL_SECONDS: f32 = 0.5;
const MAX_SAMPLED_FRAMES: usize = 100;

pub struct FrameMetricsState {
	label: Handle<Text>,
	update_interval: Duration,
	duration: Duration,
	fps_sampled_frames: usize,
	frame_times: [u32; MAX_SAMPLED_FRAMES],
	frame_time_sampled_frames: usize,
	current_frame: usize
}

impl FrameMetricsState {
	pub fn new(resources: &mut EngineResources) -> Self {
		let mut label_text = Text::new(resources.game_resources.roboto_14, String::new());
		label_text.transform.position.set(10.0, 20.0);
		let label = resources.scene.text.add(label_text);

		Self {
			label,
			update_interval: Duration::from_secs_f32(UPDATE_INTERVAL_SECONDS),
			duration: Duration::new(0, 0),
			fps_sampled_frames: 0,
			frame_times: [0; MAX_SAMPLED_FRAMES],
			frame_time_sampled_frames: 0,
			current_frame: 0
		}
	}
}

impl State for FrameMetricsState {
	fn update(&mut self, resources: &mut EngineResources, frame_time: &Duration) -> StateAction {
		self.fps_sampled_frames += 1;

		self.frame_times[self.current_frame] = frame_time.as_micros() as u32;
		self.current_frame = (self.current_frame + 1) % MAX_SAMPLED_FRAMES;
		self.frame_time_sampled_frames = (self.frame_time_sampled_frames + 1).min(MAX_SAMPLED_FRAMES);

		self.duration += *frame_time;

		if self.duration >= self.update_interval && self.frame_time_sampled_frames == MAX_SAMPLED_FRAMES {
			let mut total = 0;
			let mut max = 0;

			for frame in &self.frame_times {
				total += *frame;

				if frame > &max {
					max = *frame;
				}
			}

			let fps = self.fps_sampled_frames as f32 / self.duration.as_secs_f32();
			let average = total as f32 / MAX_SAMPLED_FRAMES as f32 / 1000.0;
			let max = max as f32 / 1000.0;

			let string = format!("{:.1}fps {:.1}ms avg {:.1}ms max", fps, average, max);
			resources.scene.text.get_mut(&self.label).unwrap().set_string(string);
			
			self.duration = Duration::new(0, 0);
			self.fps_sampled_frames = 0;
		}

		StateAction::None
	}
}