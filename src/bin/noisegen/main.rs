#![allow(unused)]
use std::num::ParseFloatError;

use bevy::{prelude::*, window::PresentMode};
use bevy_egui::EguiContexts;
use bevy_egui::egui::{self, *};


fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(bevy::window::Window {
                title: "Noise Generator".into(),
                resolution: (1280.0, 720.0).into(),
                present_mode: PresentMode::AutoVsync,
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(bevy_egui::EguiPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, update_gui)
        .run();
}

fn setup() {

}

struct NoiseGenGuiInterval {
    knots: Option<Vec<String>>,
    persistence: String,
    scale: String,
    x_mult: String,
    y_mult: String,
}
use unvoga::core::error::Result as UnvResult;
impl TryInto<NoiseGenInterval> for NoiseGenGuiInterval {
    type Error = ParseFloatError;
    fn try_into(self) -> Result<NoiseGenInterval, Self::Error> {
        let knots = if let Some(knots) = self.knots {
            let knots: Result<Box<[f64]>, _> = knots.into_iter().map(|knot| knot.parse::<f64>()).collect()?;
            Some(knots?)
        } else {
            None
        };
        
        Ok(NoiseGenInterval {
            knots,
            persistence: self.persistence.parse()?,
            scale: self.scale.parse()?,
            x_mult: self.x_mult.parse()?,
            y_mult: self.y_mult.parse()?,
        })
    }
}

struct NoiseGenInterval {
    knots: Option<Box<[f64]>>,
    persistence: f64,
    scale: f64,
    x_mult: f64,
    y_mult: f64,
}

pub struct NoiseGenGui {
    intervals: Vec<NoiseGenGuiInterval>,
    
}

#[derive(Resource)]
struct GuiResources {
    texhand: Option<TextureHandle>,
    knot_strings: Vec<String>,
}

fn update_gui(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut gui: ResMut<GuiResources>,
) {
    let image = egui::ColorImage::from_rgba_unmultiplied([2, 2], &[0]);
    egui::CentralPanel::default().show(contexts.ctx_mut(), |ui| {
        let texture = ui.ctx().load_texture("my_texture", ColorImage::new([2, 2], Color32::BLACK), TextureOptions::LINEAR);
        // knots, persistence, scale, xmul, ymul
        
        
    });
}