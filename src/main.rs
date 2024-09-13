use std::error::Error;

use midir::{Ignore, MidiInput, MidiOutput};

use piano_keyboard::{Keyboard2d, KeyboardBuilder, Rectangle, Element};

use macroquad::prelude::*;
use std::sync::{Arc, Mutex};

const PIANO_WIDTH: u16 = 1024;
const KEY_HEIGHT: u16 = 106;
// KEY_HEIGHT is 103 when adding the small and wide triangle heights of the white keys
// PIANO_WIDTH is in pixels
//
fn is_point_in_rect(x: f32, y: f32, rect: &Rectangle) -> bool {
    x >= rect.x as f32 && x <= (rect.x + rect.width) as f32 &&
    y >= rect.y as f32 && y <= (rect.y + rect.height) as f32
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Piano".to_owned(),
        window_width: 1280,
        window_height: 300,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut midi_in = MidiInput::new("midir reading input")?;
    midi_in.ignore(Ignore::None);

    let in_ports = midi_in.ports();
    let in_port = in_ports.get(0).ok_or("no input port found")?;

    println!("\nOpening connection");
    let in_port_name = midi_in.port_name(in_port)?;

    let keyboard = KeyboardBuilder::new().standard_piano(88).unwrap().set_width(PIANO_WIDTH).unwrap().build2d();
    let pressed_keys = Arc::new(Mutex::new(vec![false; 128]));
    let pressed_keys_clone = Arc::clone(&pressed_keys);

    let _conn_in = midi_in.connect(
        in_port,
        "midir-read-input",
        move |_stamp, message, _| {
            if message.len() >= 3 {
                let status = message[0];
                let note = message[1] as usize;
                let velocity = message[2];

                let mut keys = pressed_keys_clone.lock().unwrap();

                if status == 0x90 && velocity > 0 {
                    keys[note] = true;
                } else if status == 0x80 || (status == 0x90 && velocity == 0) {
                    keys[note] = false;
                }
            }
        },
        (),
    )?;

    println!(
        "Connection open, reading input from '{}'...",
        in_port_name
    );

    let screen_width = screen_width();
    let screen_height = screen_height();
    let x_offset = (screen_width - PIANO_WIDTH as f32) / 2.0;
    let y_offset = screen_height - KEY_HEIGHT as f32; 

    loop {
        clear_background(DARKGRAY);

        draw_text("Piano", 20.0, 20.0, 20.0, GRAY);


        let mut pressed_keys = pressed_keys.lock().unwrap();

        if is_mouse_button_pressed(MouseButton::Left) {
            let (mouse_x, mouse_y) = mouse_position();
            for (index, key) in keyboard.iter().enumerate() {
                let midi_note = index as u8 + 21; // Assuming the lowest key is MIDI note 21 (A0)
                match key {
                    Element::BlackKey(rect) => {
                        if is_point_in_rect(mouse_x - x_offset, mouse_y - y_offset, rect) {
                            pressed_keys[midi_note as usize] = true;
                        }
                    },
                    Element::WhiteKey{wide, small, blind} => {
                        if is_point_in_rect(mouse_x - x_offset, mouse_y - y_offset, wide) ||
                           is_point_in_rect(mouse_x - x_offset, mouse_y - y_offset, small) ||
                           blind.as_ref().map_or(false, |edge| is_point_in_rect(mouse_x - x_offset, mouse_y - y_offset, edge)) {
                           pressed_keys[midi_note as usize] = true;
                        }
                    },
                }
            }
        }

        // Release keys when mouse button is released
        if is_mouse_button_released(MouseButton::Left) {
            for pressed in pressed_keys.iter_mut() {
                *pressed = false;
            }
        }

        for (index, key) in keyboard.iter().enumerate() {
            let midi_note = index as u8 + 21; // Assuming the lowest key is MIDI note 21 (A0)
            let is_pressed = pressed_keys[midi_note as usize];

            match key {
                Element::BlackKey(rect) => {
                    let color = if is_pressed { RED } else { BLACK };
                    draw_rectangle(
                        rect.x as f32 + x_offset,
                        rect.y as f32 + y_offset,
                        rect.width as f32,
                        rect.height as f32,
                        color);
                },
                Element::WhiteKey{wide, small, blind} => {
                    let color = if is_pressed { RED } else { WHITE };
                    draw_rectangle(
                        wide.x as f32 + x_offset,
                        wide.y as f32 + y_offset,
                        wide.width as f32,
                        wide.height as f32,
                        color);

                    draw_rectangle(
                        small.x as f32 + x_offset,
                        small.y as f32 + y_offset,
                        small.width as f32,
                        small.height as f32,
                        color);

                    if let Some(edge_key) = blind {
                        draw_rectangle(
                            edge_key.x as f32 + x_offset,
                            edge_key.y as f32 + y_offset,
                            edge_key.width as f32,
                            edge_key.height as f32,
                            color);
                    }
                },
            }
        }
        
        next_frame().await
    }
}
