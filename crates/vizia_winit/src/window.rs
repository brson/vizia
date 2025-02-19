use crate::convert::cursor_icon_to_cursor_icon;
use femtovg::{renderer::OpenGl, Canvas, Color};
#[cfg(not(target_arch = "wasm32"))]
use glutin::ContextBuilder;
use vizia_core::prelude::*;
use winit::event_loop::EventLoop;
use winit::window::{CursorGrabMode, WindowBuilder};
use winit::{dpi::*, window::WindowId};

pub struct Window {
    pub id: WindowId,
    #[cfg(not(target_arch = "wasm32"))]
    handle: glutin::WindowedContext<glutin::PossiblyCurrent>,
    #[cfg(target_arch = "wasm32")]
    handle: winit::window::Window,
    pub should_close: bool,
}

#[cfg(target_arch = "wasm32")]
impl Window {
    pub fn new(
        events_loop: &EventLoop<Event>,
        window_description: &WindowDescription,
    ) -> (Self, Canvas<OpenGl>) {
        let window_builder = WindowBuilder::new();

        // For wasm, create or look up the canvas element we're drawing on
        let canvas_element = {
            use wasm_bindgen::JsCast;

            let document = web_sys::window().unwrap().document().unwrap();

            if let Some(canvas_id) = &window_description.target_canvas {
                document.get_element_by_id(canvas_id).unwrap()
            } else {
                let element = document.create_element("canvas").unwrap();
                document.body().unwrap().insert_adjacent_element("afterbegin", &element).unwrap();
                element
            }
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap()
        };

        // Build the femtovg renderer
        let renderer = OpenGl::new_from_html_canvas(&canvas_element).unwrap();

        // tell winit about the above canvas
        let window_builder = {
            use winit::platform::web::WindowBuilderExtWebSys;
            window_builder.with_canvas(Some(canvas_element))
        };

        // Apply generic WindowBuilder properties
        let window_builder = apply_window_description(window_builder, &window_description);

        // Get the window handle. this is a winit::window::Window
        let handle = window_builder.build(&events_loop).unwrap();

        // Build our window
        let window = Window {
            id: handle.id(),
            handle,
            should_close: false,
            //canvas: Canvas::new(renderer).expect("Cannot create canvas"),
        };

        let mut canvas = Canvas::new(renderer).expect("Failed to create canvas");

        let size = window.window().inner_size();
        canvas.set_size(size.width as u32, size.height as u32, 1.0);
        canvas.clear_rect(0, 0, size.width as u32, size.height as u32, Color::rgb(255, 80, 80));

        (window, canvas)
    }

    pub fn window(&self) -> &winit::window::Window {
        &self.handle
    }

    pub fn resize(&self, _size: PhysicalSize<u32>) {
        // TODO?
    }

    pub fn swap_buffers(&self) {
        // Intentional no-op
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Window {
    pub fn new(
        events_loop: &EventLoop<Event>,
        window_description: &WindowDescription,
    ) -> (Self, Canvas<OpenGl>) {
        let window_builder = WindowBuilder::new();

        //Windows COM doesn't play nicely with winit's drag and drop right now
        #[cfg(target_os = "windows")]
        let window_builder = {
            use winit::platform::windows::WindowBuilderExtWindows;
            window_builder.with_drag_and_drop(false)
        };

        // Apply generic WindowBuilder properties
        let window_builder = apply_window_description(window_builder, &window_description);

        // Get the window handle. this is a ContextWrapper
        let handle = {
            let handle = ContextBuilder::new()
                .with_vsync(window_description.vsync)
                // .with_srgb(true)
                .build_windowed(window_builder, &events_loop)
                .expect("Window context creation failed!");

            unsafe { handle.make_current().unwrap() }
        };

        // Build the femtovg renderer
        let renderer = OpenGl::new_from_glutin_context(&handle).expect("Cannot create renderer");

        let mut canvas = Canvas::new(renderer).expect("Failed to create canvas");

        let size = handle.window().inner_size();
        canvas.set_size(size.width as u32, size.height as u32, 1.0);
        canvas.clear_rect(0, 0, size.width as u32, size.height as u32, Color::rgb(255, 80, 80));

        //cx.canvases.insert(Entity::root(), canvas);

        // Build our window
        let window = Window { id: handle.window().id(), handle, should_close: false };

        (window, canvas)
    }

    pub fn window(&self) -> &winit::window::Window {
        self.handle.window()
    }

    pub fn resize(&self, size: PhysicalSize<u32>) {
        self.handle.resize(size);
    }

    pub fn swap_buffers(&self) {
        self.handle.swap_buffers().expect("Failed to swap buffers");
    }
}

impl View for Window {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|window_event, _| match window_event {
            WindowEvent::GrabCursor(flag) => {
                let grab_mode = if *flag { CursorGrabMode::Locked } else { CursorGrabMode::None };
                self.window().set_cursor_grab(grab_mode).expect("Failed to set cursor grab");
            }

            WindowEvent::SetCursorPosition(x, y) => {
                self.window()
                    .set_cursor_position(winit::dpi::Position::Physical(PhysicalPosition::new(
                        *x as i32, *y as i32,
                    )))
                    .expect("Failed to set cursor position");
            }

            WindowEvent::SetCursor(cursor) => {
                if let Some(icon) = cursor_icon_to_cursor_icon(*cursor) {
                    self.window().set_cursor_visible(true);
                    self.window().set_cursor_icon(icon);
                } else {
                    self.window().set_cursor_visible(false);
                }
            }

            WindowEvent::SetTitle(title) => {
                self.window().set_title(title);
            }

            WindowEvent::SetSize(size) => {
                self.window().set_inner_size(LogicalSize::new(size.width, size.height));
            }

            WindowEvent::SetMinSize(size) => {
                self.window()
                    .set_min_inner_size(size.map(|size| LogicalSize::new(size.width, size.height)));
            }

            WindowEvent::SetMaxSize(size) => {
                self.window()
                    .set_max_inner_size(size.map(|size| LogicalSize::new(size.width, size.height)));
            }

            WindowEvent::SetPosition(pos) => {
                self.window().set_outer_position(LogicalPosition::new(pos.x, pos.y));
            }

            WindowEvent::SetResizable(flag) => {
                self.window().set_resizable(*flag);
            }

            WindowEvent::SetMinimized(flag) => {
                self.window().set_minimized(*flag);
            }

            WindowEvent::SetMaximized(flag) => {
                self.window().set_maximized(*flag);
            }

            WindowEvent::SetVisible(flag) => {
                self.window().set_visible(*flag);
            }

            WindowEvent::SetDecorations(flag) => {
                self.window().set_decorations(*flag);
            }

            WindowEvent::SetAlwaysOnTop(flag) => {
                self.window().set_always_on_top(*flag);
            }

            WindowEvent::ReloadStyles => {
                cx.reload_styles().unwrap();
            }

            WindowEvent::WindowClose => {
                self.should_close = true;
            }

            _ => {}
        })
    }
}

fn apply_window_description(
    mut builder: WindowBuilder,
    description: &WindowDescription,
) -> WindowBuilder {
    builder = builder.with_title(&description.title).with_inner_size(LogicalSize::new(
        description.inner_size.width,
        description.inner_size.height,
    ));

    if let Some(min_inner_size) = description.min_inner_size {
        builder = builder
            .with_min_inner_size(LogicalSize::new(min_inner_size.width, min_inner_size.height))
    }

    if let Some(max_inner_size) = description.max_inner_size {
        builder = builder
            .with_max_inner_size(LogicalSize::new(max_inner_size.width, max_inner_size.height));
    }

    if let Some(position) = description.position {
        builder = builder.with_position(LogicalPosition::new(position.x, position.y));
    }

    builder
        .with_resizable(description.resizable)
        .with_maximized(description.maximized)
        .with_visible(description.visible)
        .with_transparent(description.transparent)
        .with_decorations(description.decorations)
        .with_always_on_top(description.always_on_top)
        .with_window_icon(if let Some(icon) = &description.icon {
            Some(
                winit::window::Icon::from_rgba(
                    icon.clone(),
                    description.icon_width,
                    description.icon_height,
                )
                .unwrap(),
            )
        } else {
            None
        })
}
