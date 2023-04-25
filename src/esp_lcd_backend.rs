use std::{cell::RefCell, rc::Rc, time::Instant};

use embedded_graphics::prelude::Dimensions;
use log::debug;
use slint::platform::WindowEvent;

use crate::{
    gt911::read_touch,
    hx8369::HX8369,
};

pub struct EspBackend {
    start: Instant,
    window: RefCell<Option<Rc<slint::platform::software_renderer::MinimalSoftwareWindow>>>,
}

impl Default for EspBackend {
    fn default() -> Self {
        Self {
            start: Instant::now(),
            window: RefCell::new(None),
        }
    }
}

impl slint::platform::Platform for EspBackend {
    fn create_window_adapter(
        &self,
    ) -> Result<Rc<dyn slint::platform::WindowAdapter>, slint::PlatformError> {
        let window = slint::platform::software_renderer::MinimalSoftwareWindow::new(
            slint::platform::software_renderer::RepaintBufferType::ReusedBuffer,
        );
        self.window.replace(Some(window.clone()));
        Ok(window)
    }

    fn duration_since_start(&self) -> core::time::Duration {
        Instant::now().duration_since(self.start)
    }

    fn run_event_loop(&self) -> Result<(), slint::PlatformError> {
        let mut display = HX8369::new(800, 480);
        display.flush();
        let size = display.bounding_box().size;
        let size = slint::PhysicalSize::new(size.width, size.height);

        self.window.borrow().as_ref().unwrap().set_size(size);

        let mut last_touch = None;

        loop {
            slint::platform::update_timers_and_animations();

            if let Some(window) = self.window.borrow().clone() {
                if let Some(touch) = read_touch() {
                    // There is a new touch event
                    let position = slint::PhysicalPosition::new(touch.x as i32, touch.y as i32)
                        .to_logical(window.scale_factor());
                    let button = slint::platform::PointerEventButton::Left;
                    let event = if touch.pressed {
                        Some(match last_touch.replace(position) {
                            Some(_) => WindowEvent::PointerMoved { position },
                            None => WindowEvent::PointerPressed { position, button },
                        })
                    } else {
                        last_touch
                            .take()
                            .map(|position| WindowEvent::PointerReleased { position, button })
                    };

                    if let Some(event) = event {
                        let is_pointer_release_event =
                            matches!(event, WindowEvent::PointerReleased { .. });

                        window.dispatch_event(event);

                        // removes hover state on widgets
                        if is_pointer_release_event {
                            window.dispatch_event(WindowEvent::PointerExited);
                        }
                    }
                }
                let stride = display.bounding_box().size.width as usize;
                let buffer = display.get_raw_buffer();
                let dirty = window.draw_if_needed(|renderer| {
                    // renderer.render_by_line(&mut buffer_provider);
                    renderer.render(buffer, stride);
                });
                if dirty {
                    display.flush();
                }
                if window.has_active_animations() {
                    continue;
                }
            }
            // thread::sleep(Duration::from_millis(1));
        }
    }

    fn debug_log(&self, arguments: core::fmt::Arguments) {
        debug!("{}", arguments);
    }
}
